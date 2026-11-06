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
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use spin::{Lazy, Mutex};

mod devfs;
mod fds;
mod tarfs;
mod types;

pub use self::fds::*;
pub use self::types::*;
use crate::system::posix::errno::{KError, KResult};
use crate::system::vfs::tarfs::TarFS;

pub struct Mount {
    path: String,
    fs: Box<dyn FileSystem>,
}

pub struct Vfs {
    mounts: Vec<Mount>,
}

// internals
impl Vfs {
    /// creates a new VFS instance
    pub fn new() -> Self { Self { mounts: Vec::new() } }

    /// start up all the filesystems
    pub fn initialize(&mut self) -> KResult<()> {
        for mount in &mut self.mounts {
            mount.fs.initialize()?;
        }
        Ok(())
    }

    /// mounts the given filesystem at the given path
    pub fn mount(
        &mut self,
        path: &str,
        fs: Box<dyn FileSystem>,
    ) -> KResult<()> {
        let path = path.to_string();

        if self.mounts.iter().any(|m| m.path == path) {
            return Err(KError::EEXIST);
        }

        self.mounts.push(Mount { path, fs });

        Ok(())
    }

    /// unmounts the filesystem at the given path
    pub fn unmount(&mut self, path: &str) -> KResult<()> {
        let path = path.to_string();

        let idx = self
            .mounts
            .iter()
            .position(|m| m.path == path)
            .ok_or(KError::EEXIST)?;
        self.mounts.remove(idx);
        Ok(())
    }

    /// resolves the given path to a filesystem and relative path
    pub fn resolve(&self, path: &str) -> KResult<(&dyn FileSystem, String)> {
        let path = path.to_string();
        let mut best: Option<(&dyn FileSystem, usize, String)> = None;

        for mount in &self.mounts {
            let matches = if mount.path == "/" {
                true
            } else {
                path == mount.path
                    || path.starts_with(&format!("{}/", mount.path))
            };

            if !matches {
                continue;
            }

            let relative = if mount.path == "/" {
                path.to_string()
            } else if path == mount.path {
                "/".to_string()
            } else {
                path[mount.path.len()..].to_string()
            };

            let length = mount.path.len();

            if best.as_ref().is_none_or(|(_, l, _)| length > *l) {
                best = Some((mount.fs.as_ref(), length, relative));
            }
        }

        match best {
            Some((fs, _, relative)) => Ok((fs, relative)),
            None => Err(KError::EEXIST),
        }
    }
}

// public api
impl Vfs {
    /// opens the file at the given path with the given flags
    pub fn open(&self, path: &str, flags: u32) -> KResult<FileLike> {
        let (fs, relative) = self.resolve(path)?;
        fs.open(&relative, flags)
    }
}

// global instance
static ROOT_VFS: Lazy<Mutex<Vfs>> = Lazy::new(|| Mutex::new(Vfs::new()));

pub fn install() {
    let tarfs = TarFS::new();
    ROOT_VFS.lock().mount("/", Box::new(tarfs)).expect("failed to mount tarfs");

    let devfs = devfs::create_devfs();
    ROOT_VFS
        .lock()
        .mount("/dev", Box::new(devfs))
        .expect("failed to mount devfs");

    let procfs = devfs::create_procfs();
    ROOT_VFS
        .lock()
        .mount("/proc", Box::new(procfs))
        .expect("failed to mount procfs");

    log::info!("mounted {} filesystems", ROOT_VFS.lock().mounts.len());
    ROOT_VFS.lock().initialize().expect("failed to initialize vfs");
}

// public methods
pub fn open(path: &str, flags: u32) -> KResult<FileLike> {
    ROOT_VFS.lock().open(path, flags)
}

/// reads the entire contents of the file then returns it as a vector of bytes.
/// only for internal use
pub fn __read(path: &str) -> Result<Vec<u8>, &'static str> {
    let file = match open(path, 0).map_err(|_| "failed to read file")? {
        FileLike::File(f) => f,
        _ => return Err("expected a regular file"),
    };

    let metadata = file.stat().map_err(|_| "failed to stat file")?;

    let mut file_data = Vec::with_capacity(metadata.size.max(1));
    let mut tmp = alloc::vec![0u8; 4096];

    loop {
        let read = file.read(&mut tmp).map_err(|_| "failed to read file")?;
        if read == 0 {
            break;
        }
        file_data.extend_from_slice(&tmp[..read]);
    }

    if file_data.is_empty() {
        return Err("empty file");
    }

    Ok(file_data)
}
