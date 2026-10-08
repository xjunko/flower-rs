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

use spin::Mutex;

use crate::vfs::file::{OpenFile, OpenFlags};
use crate::vfs::path::{components, join, normalize, parent_and_name};
use crate::vfs::traits::{FileSystem, Inode};

pub mod file;
pub mod path;
pub mod ramfs;
pub mod traits;

/// generic vfs types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsError {
    NotFound,
    AlreadyExists,
    NotADirectory,
    NotAFile,
    PermissionDenied,
    InvalidArgument,
    InvalidPath,
    IOError,
    ReadOnly,
    Busy,
    Unknown,
}

pub type VfsResult<T> = Result<T, VfsError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    Directory,
    Symlink,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub inode: u64,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub enum SeekFrom {
    Start(i64),
    End(i64),
    Current(i64),
}

/// vfs impl
const MAX_SYMLINK_DEPTH: usize = 8;

struct MountEntry {
    path: String,
    fs: Box<dyn FileSystem + Send + Sync>,
}

#[derive(Default)]
pub struct Vfs {
    mounts: Vec<MountEntry>,
}

impl Vfs {
    pub const fn new() -> Self { Self { mounts: Vec::new() } }

    pub fn mount(
        &mut self,
        path: impl AsRef<str>,
        fs: Box<dyn FileSystem + Send + Sync>,
    ) -> VfsResult<()> {
        let path = normalize(path.as_ref());

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

    pub fn get_mount_at(
        &self,
        path: &str,
    ) -> VfsResult<(&dyn FileSystem, Vec<String>)> {
        let path = normalize(path);
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
            if mount.path == "/" { &path } else { &path[mount.path.len()..] };

        Ok((
            mount.fs.as_ref(),
            components(relative).into_iter().map(String::from).collect(),
        ))
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
            return Err(VfsError::IOError);
        }

        let (fs, comps) = self.get_mount_at(abs_path)?;
        let mut current = fs.root()?;
        let mut walked = String::new();

        for (i, component) in comps.iter().enumerate() {
            let is_last = i == comps.len() - 1;

            current = current.lookup(component)?;
            walked = join(&walked, component);

            if current.get_type()? == FileType::Symlink
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

// public api
impl Vfs {
    pub fn open(&self, path: &str, flags: OpenFlags) -> VfsResult<OpenFile> {
        let inode = match self.resolve(path, true) {
            Ok(inode) => {
                if flags.contains(OpenFlags::CREATE) {
                    return Err(VfsError::AlreadyExists);
                }

                inode
            },
            Err(VfsError::NotFound) => {
                if !flags.contains(OpenFlags::CREATE) {
                    return Err(VfsError::NotFound);
                }

                let (parent_path, name) = parent_and_name(path);
                if name.is_empty() {
                    return Err(VfsError::InvalidPath);
                }
                self.resolve(&parent_path, true)?.create(&name, flags)?
            },
            Err(err) => return Err(err),
        };

        let ops = inode.open(flags)?;
        if flags.contains(OpenFlags::TRUNCATE) {
            ops.truncate(0)?;
        }

        Ok(OpenFile::new(inode.clone(), ops.clone(), flags))
    }

    pub fn metadata(&self, path: &str) -> VfsResult<Metadata> {
        let inode = self.resolve(path, true)?;
        inode.metadata()
    }

    pub fn exists(&self, path: &str) -> bool {
        self.resolve(path, true).is_ok()
    }
}

//
static ROOT_VFS: Mutex<Vfs> = Mutex::new(Vfs::new());

pub fn install() -> VfsResult<()> {
    let mut vfs = ROOT_VFS.lock();

    vfs.mount("/tmp/", Box::new(ramfs::RamFs::new()))?;

    log::info!("mounted {} filesystem(s)", vfs.mounts.len());
    Ok(())
}

pub fn mount(
    path: impl AsRef<str>,
    fs: Box<dyn FileSystem + Send + Sync>,
) -> VfsResult<()> {
    ROOT_VFS.lock().mount(path, fs)
}

pub fn open(path: &str, flags: OpenFlags) -> VfsResult<OpenFile> {
    let file = ROOT_VFS.lock().open(path, flags)?;
    Ok(file)
}
