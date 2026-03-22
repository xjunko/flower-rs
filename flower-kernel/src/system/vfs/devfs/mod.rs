mod keyboard;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::system::vfs::{
    VFSError, VFSFile, VFSFileType, VFSImplementation, VFSMetadata,
    VFSPermissions, VFSResult, VFSSeek,
};

pub struct DevFile {
    path: String,

    _read: Option<fn(&mut [u8]) -> usize>,
    _write: Option<fn(&[u8]) -> usize>,
}

impl Clone for DevFile {
    fn clone(&self) -> Self {
        Self { path: self.path.clone(), _read: self._read, _write: self._write }
    }
}

impl VFSFile for DevFile {
    fn read(&self, buf: &mut [u8]) -> VFSResult<usize> {
        if let Some(read_fn) = self._read {
            Ok(read_fn(buf))
        } else {
            Err(VFSError::Unsupported)
        }
    }

    fn write(&self, buf: &mut [u8]) -> VFSResult<usize> {
        if let Some(write_fn) = self._write {
            Ok(write_fn(buf))
        } else {
            Err(VFSError::Unsupported)
        }
    }

    fn seek(&mut self, _pos: VFSSeek) -> VFSResult<usize> { Ok(1) }

    fn metadata(&self) -> VFSResult<VFSMetadata> {
        Ok(VFSMetadata {
            name: self
                .path
                .split("/")
                .last()
                .unwrap_or(self.path.as_str())
                .to_string(),
            typ: VFSFileType::Device,
            size: 0,
            last_modified: 0,
            owner_id: 0,
            group_id: 0,
            permissions: VFSPermissions::new(),
        })
    }
}

pub struct DevFS {
    files: Vec<DevFile>,
}

impl DevFS {
    pub fn new() -> Self {
        let mut dev = Self { files: Vec::new() };
        keyboard::install(&mut dev);
        dev
    }

    pub fn bind(&mut self, file: DevFile) { self.files.push(file); }
}

impl VFSImplementation for DevFS {
    fn open(&self, path: &str, _flags: u32) -> VFSResult<Box<dyn VFSFile>> {
        self.files
            .iter()
            .find(|f| f.path == path)
            .map(|f| Box::new(f.clone()) as Box<dyn VFSFile>)
            .ok_or(VFSError::NotFound)
    }

    fn metadata(&self, path: &str) -> VFSResult<VFSMetadata> {
        self.files
            .iter()
            .find(|f| f.path == path)
            .map(|f| f.metadata())
            .ok_or(VFSError::NotFound)?
    }
}
