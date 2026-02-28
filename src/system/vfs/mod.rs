use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

use spin::{Lazy, Mutex};

mod tarfs;
mod types;

use self::{tarfs::TarFS, types::*};

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
    pub fn new() -> Self {
        Self { mounts: Vec::new() }
    }

    /// mounts the given filesystem at the given path
    pub fn mount(&mut self, path: &str, fs: Box<dyn VFSImplementation>) -> VFSResult<()> {
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
    pub fn resolve(&self, path: &str) -> VFSResult<(&dyn VFSImplementation, String)> {
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
}

// public methods
pub fn open(path: &str, flags: u32) -> VFSResult<Box<dyn VFSFile>> {
    ROOT_VFS.lock().open(path, flags)
}
