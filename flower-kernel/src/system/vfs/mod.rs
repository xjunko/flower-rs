use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use spin::{Lazy, Mutex};

mod devfs;
mod fds;
mod tarfs;
mod types;

pub use self::fds::*;
pub use self::types::*;
use crate::system::vfs::tarfs::TarFS;

pub struct Mount {
    path: String,
    fs: Box<dyn VFSImplementation>,
}

pub struct Vfs {
    mounts: Vec<Mount>,
}

// internals
impl Vfs {
    /// creates a new VFS instance
    pub fn new() -> Self { Self { mounts: Vec::new() } }

    /// mounts the given filesystem at the given path
    pub fn mount(
        &mut self,
        path: &str,
        fs: Box<dyn VFSImplementation>,
    ) -> VFSResult<()> {
        let path = path.to_string();

        if self.mounts.iter().any(|m| m.path == path) {
            return Err(VFSError::AlreadyExists);
        }

        self.mounts.push(Mount { path, fs });

        Ok(())
    }

    /// unmounts the filesystem at the given path
    pub fn unmount(&mut self, path: &str) -> VFSResult<()> {
        let path = path.to_string();

        let idx = self
            .mounts
            .iter()
            .position(|m| m.path == path)
            .ok_or(VFSError::NotFound)?;
        self.mounts.remove(idx);
        Ok(())
    }

    /// resolves the given path to a filesystem and relative path
    pub fn resolve(
        &self,
        path: &str,
    ) -> VFSResult<(&dyn VFSImplementation, String)> {
        let path = path.to_string();

        for mount in &self.mounts {
            if mount.path == "/" {
                return Ok((mount.fs.as_ref(), path.clone()));
            }

            if mount.path == path {
                return Ok((mount.fs.as_ref(), "/".to_string()));
            }

            if path.starts_with(&mount.path) {
                let rest = &path[mount.path.len()..];
                let relative = if rest.is_empty() || rest == "/" {
                    "/".to_string()
                } else if rest.starts_with("/") {
                    rest.to_string()
                } else {
                    continue;
                };
                return Ok((mount.fs.as_ref(), relative));
            }
        }

        Err(VFSError::NotFound)
    }
}

// public api
impl Vfs {
    /// opens the file at the given path with the given flags
    pub fn open(&self, path: &str, flags: u32) -> VFSResult<Box<dyn VFSFile>> {
        let (fs, relative) = self.resolve(path)?;
        fs.open(&relative, flags)
    }
}

// global instance
static ROOT_VFS: Lazy<Mutex<Vfs>> = Lazy::new(|| Mutex::new(Vfs::new()));

pub fn install() {
    let tarfs = TarFS::new();
    ROOT_VFS
        .lock()
        .mount("/init", Box::new(tarfs))
        .expect("failed to mount tarfs");

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
}

// public methods
pub fn open(path: &str, flags: u32) -> VFSResult<Box<dyn VFSFile>> {
    ROOT_VFS.lock().open(path, flags)
}

/// reads the entire contents of the file then returns it as a vector of bytes.
/// only for internal use
pub fn __read(path: &str) -> Result<Vec<u8>, &'static str> {
    let file = open(path, 0).map_err(|_| "failed to read file")?;
    let metadata = file.metadata().map_err(|_| "failed to stat file")?;

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
