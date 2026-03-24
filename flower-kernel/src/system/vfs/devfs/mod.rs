mod audio;
mod keyboard;
mod proc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ffi::c_int;
use core::sync::atomic::{AtomicUsize, Ordering};

pub use proc::create_procfs;

use crate::system::vfs::{
    VFSError, VFSFile, VFSFileType, VFSImplementation, VFSMetadata,
    VFSPermissions, VFSResult, VFSSeek,
};

pub struct DevFile {
    path: String,
    position: AtomicUsize,

    fn_read: Option<fn(usize, &mut [u8]) -> usize>,
    fn_write: Option<fn(&[u8]) -> usize>,
    fn_mmap: Option<fn(usize, c_int, c_int) -> *mut u8>,
}

impl DevFile {
    pub fn new(
        path: String,
        read: Option<fn(usize, &mut [u8]) -> usize>,
        write: Option<fn(&[u8]) -> usize>,
        mmap: Option<fn(usize, c_int, c_int) -> *mut u8>,
    ) -> Self {
        Self {
            path,
            position: AtomicUsize::new(0),
            fn_read: read,
            fn_write: write,
            fn_mmap: mmap,
        }
    }
}

impl Clone for DevFile {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            position: AtomicUsize::new(0),
            fn_read: self.fn_read,
            fn_write: self.fn_write,
            fn_mmap: self.fn_mmap,
        }
    }
}

impl VFSFile for DevFile {
    fn read(&self, buf: &mut [u8]) -> VFSResult<usize> {
        if let Some(read_fn) = self.fn_read {
            let position = self.position.load(Ordering::Acquire);
            let read = read_fn(position, buf);
            if read > 0 {
                self.position.fetch_add(read, Ordering::AcqRel);
            }
            Ok(read)
        } else {
            Err(VFSError::Unsupported)
        }
    }

    fn write(&self, buf: &mut [u8]) -> VFSResult<usize> {
        if let Some(write_fn) = self.fn_write {
            Ok(write_fn(buf))
        } else {
            Err(VFSError::Unsupported)
        }
    }

    fn seek(&mut self, pos: VFSSeek) -> VFSResult<usize> {
        let current = self.position.load(Ordering::Acquire);
        let new_pos = match pos {
            VFSSeek::Start(n) => n,
            VFSSeek::Current(n) => current.saturating_add(n),
            VFSSeek::End(n) => n,
        };

        self.position.store(new_pos, Ordering::Release);
        Ok(new_pos)
    }

    fn mmap(
        &self,
        len: usize,
        prot: core::ffi::c_int,
        flags: core::ffi::c_int,
    ) -> VFSResult<*mut u8> {
        if let Some(mmap_fn) = self.fn_mmap {
            Ok(mmap_fn(len, prot, flags))
        } else {
            Err(VFSError::Unsupported)
        }
    }

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
    pub fn new() -> Self { Self { files: Vec::new() } }

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

pub fn create_devfs() -> DevFS {
    let mut mnt = DevFS::new();
    keyboard::install(&mut mnt);
    audio::install(&mut mnt);
    mnt
}
