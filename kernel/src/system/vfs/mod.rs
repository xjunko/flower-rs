/*
 * ISC License
 *
 * Copyright (c) 2025-2026 xjunko
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
 * REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
 * AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
 * INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
 * LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
 * OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
 * PERFORMANCE OF THIS SOFTWARE.
 */

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use spin::{LazyLock, Mutex};

use crate::system::vfs::error::{VfsError, VfsResult};
use crate::system::vfs::file::{File, OpenFlags};
use crate::system::vfs::fs::FileSystem;
use crate::system::vfs::inode::{FileType, Inode, Metadata};
use crate::system::vfs::path::{components, join, normalize, parent_and_name};
use crate::system::vfs::perm::{Access, Credentials};

// internals
pub mod error;
pub mod fd;
pub mod file;
pub mod fs;
pub mod inode;
pub mod path;
pub mod perm;

// impls
mod devfs;
mod tarfs;

const MAX_SYMLINK_DEPTH: usize = 8;

struct MountEntry {
    path: String,
    fs: Box<dyn FileSystem>,
}

pub struct Vfs {
    mounts: Vec<MountEntry>,
}

impl Vfs {
    pub fn new() -> Self { Self { mounts: Vec::new() } }

    pub fn mount(
        &mut self,
        path: &str,
        fs: Box<dyn FileSystem>,
    ) -> VfsResult<()> {
        let path = normalize(path);

        if self.mounts.iter().any(|m| m.path == path) {
            return Err(VfsError::AlreadyExists);
        }

        self.mounts.push(MountEntry { path, fs });
        Ok(())
    }

    pub fn unmount(&mut self, path: &str) -> VfsResult<()> {
        let path = normalize(path);
        let idx = self
            .mounts
            .iter()
            .position(|m| m.path == path)
            .ok_or(VfsError::NotFound)?;
        self.mounts.remove(idx);
        Ok(())
    }

    pub fn get_mount_at<'a>(
        &self,
        path: &'a str,
    ) -> VfsResult<(&dyn FileSystem, Vec<&'a str>)> {
        let mut best: Option<&MountEntry> = None;

        for mount in &self.mounts {
            let matches = mount.path == "/"
                || path == mount.path
                || path.starts_with(&format!("{}/", mount.path));

            if matches && best.is_none_or(|b| mount.path.len() > b.path.len()) {
                best = Some(mount);
            }
        }

        let mount = best.ok_or(VfsError::NotFound)?;
        let relative =
            if mount.path == "/" { path } else { &path[mount.path.len()..] };

        Ok((mount.fs.as_ref(), components(relative)))
    }

    pub fn resolve(
        &self,
        abs_path: &str,
        follow_final_symlinks: bool,
    ) -> VfsResult<Arc<dyn Inode>> {
        self.__resolve_inner(abs_path, follow_final_symlinks, 0)
    }

    fn __resolve_inner(
        &self,
        abs_path: &str,
        follow_final_symlinks: bool,
        depth: usize,
    ) -> VfsResult<Arc<dyn Inode>> {
        if depth > MAX_SYMLINK_DEPTH {
            return Err(VfsError::TooManySymlinks);
        }

        let (fs, comps) = self.get_mount_at(abs_path)?;
        let mut current = fs.root();
        let mut walked = String::new();

        for (i, component) in comps.iter().enumerate() {
            let is_last = i == comps.len() - 1;

            current = current.lookup(component)?;
            walked = join(&walked, component);

            if current.file_type()? == FileType::Symlink
                && (!is_last || follow_final_symlinks)
            {
                let target = current.readlink()?;
                let target = if target.starts_with("/") {
                    target
                } else {
                    let (parent, _) = parent_and_name(&walked);
                    join(&parent, &target)
                };

                current = self.__resolve_inner(&target, true, depth + 1)?;
            }
        }

        Ok(current)
    }
}

/// public api
impl Vfs {
    pub fn open(
        &self,
        abs_path: &str,
        flags: OpenFlags,
        creds: Credentials,
    ) -> VfsResult<Arc<File>> {
        let inode = match self.resolve(abs_path, true) {
            Ok(inode) => {
                if flags.contains(OpenFlags::CREATE)
                    && flags.contains(OpenFlags::EXCLUSIVE)
                {
                    return Err(VfsError::AlreadyExists);
                }
                inode
            },
            Err(VfsError::NotFound) if flags.contains(OpenFlags::CREATE) => {
                let (parent_path, name) = parent_and_name(abs_path);
                let parent = self.resolve(&parent_path, true)?;
                let parent_meta = parent.metadata()?;

                if parent_meta.file_type != FileType::Directory {
                    return Err(VfsError::NotADirectory);
                }

                if !parent_meta.permissions.check(
                    creds,
                    parent_meta.owner,
                    parent_meta.group,
                    Access::Write,
                ) {
                    return Err(VfsError::PermissionDenied);
                }

                parent.create(&name, FileType::Regular)?
            },
            Err(err) => return Err(err),
        };

        let meta = inode.metadata()?;

        if flags.contains(OpenFlags::DIRECTORY)
            && meta.file_type != FileType::Directory
        {
            return Err(VfsError::NotADirectory);
        }

        let access =
            if flags.writable() { Access::Write } else { Access::Read };
        if !meta.permissions.check(creds, meta.owner, meta.group, access) {
            return Err(VfsError::PermissionDenied);
        }

        if flags.contains(OpenFlags::TRUNCATE) && flags.writable() {
            inode.truncate(0)?;
        }

        Ok(Arc::new(File::new(inode, flags)))
    }

    pub fn metadata(&self, abs_path: &str) -> VfsResult<Metadata> {
        self.resolve(abs_path, true)?.metadata()
    }

    pub fn exists(&self, abs_path: &str) -> bool {
        self.resolve(abs_path, true).is_ok()
    }
}

// public
static ROOT_VFS: LazyLock<Mutex<Vfs>> =
    LazyLock::new(|| Mutex::new(Vfs::new()));

pub fn install() {
    let mut vfs = ROOT_VFS.lock();

    vfs.mount("/dev", Box::new(devfs::unix::create()))
        .expect("failed to mount devfs");

    vfs.mount("/proc", Box::new(devfs::proc::create()))
        .expect("failed to mount procfs");

    vfs.mount("/init/", Box::new(tarfs::TarFs::create("/boot/init.tar")))
        .expect("failed to mount tarfs");

    log::info!("mounted {} filesystem", vfs.mounts.len());
}

pub fn open(
    path: &str,
    flags: OpenFlags,
    creds: Credentials,
) -> VfsResult<Arc<File>> {
    ROOT_VFS.lock().open(path, flags, creds)
}
