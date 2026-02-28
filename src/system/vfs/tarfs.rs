use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use core::sync::atomic::{AtomicUsize, Ordering};

use limine::file::File;

use super::types::*;
use crate::{boot::limine::MODULE_REQUESTS, error};

unsafe impl Send for TarFile {}
unsafe impl Sync for TarFile {}
struct TarFile {
    path: String,
    position: usize,
    offset: AtomicUsize,
    size: usize,
    fs: Option<*const TarFS>,
}

impl VFSFile for TarFile {
    fn read(&self, buf: &mut [u8]) -> VFSResult<usize> {
        self.fs.ok_or(VFSError::NotFound).map(|fs| {
            let fs = unsafe { &*fs };
            let offset = self.offset.load(Ordering::Acquire);

            if offset >= self.size {
                return 0;
            }

            let bytes_to_read = core::cmp::min(buf.len(), self.size - offset);
            let src = unsafe { fs.addr.add(self.position + offset) };
            unsafe {
                core::ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), bytes_to_read);
            }

            self.offset
                .store(offset.saturating_add(bytes_to_read), Ordering::Release);

            bytes_to_read
        })
    }

    fn write(&self, _buf: &mut [u8]) -> VFSResult<usize> {
        unimplemented!()
    }

    fn seek(&self, pos: super::types::VFSSeek) -> VFSResult<usize> {
        let current = self.offset.load(Ordering::Acquire);
        let new_offset = match pos {
            VFSSeek::Start(offset) => offset,
            VFSSeek::Current(offset) => current.saturating_add(offset),
            VFSSeek::End(offset) => self.size.saturating_sub(offset),
        }
        .min(self.size);

        self.offset.store(new_offset, Ordering::Release);
        Ok(new_offset)
    }

    fn metadata(&self) -> VFSResult<VFSMetadata> {
        self.fs.ok_or(VFSError::NotFound).and_then(|fs| {
            let fs = unsafe { &*fs };
            fs.metadata(&self.path)
        })
    }
}

unsafe impl Send for TarFS {}
unsafe impl Sync for TarFS {}
pub struct TarFS {
    addr: *mut u8,
    files: Vec<TarFile>,
}

impl TarFS {
    pub fn new() -> Self {
        let mut files: Vec<TarFile> = Vec::new();
        let mut addr: Option<*mut u8> = None;

        {
            let mut initram_file: Option<&File> = None;

            for file in MODULE_REQUESTS
                .get_response()
                .expect("tarfs not found")
                .modules()
            {
                if file.path().to_str().expect("invalid file path") == "/boot/initramfs.tar" {
                    initram_file = Some(file);
                    addr = Some(file.addr());
                    break;
                }
            }

            if initram_file.is_none() {
                panic!("tarfs not found");
            }

            // stream the file into memory
            let file = initram_file.unwrap();
            let size = file.size() as usize;
            let mut data = alloc::vec![0u8; size];
            unsafe {
                core::ptr::copy_nonoverlapping(file.addr(), data.as_mut_ptr(), size);
            }

            // read all the files
            let mut offset = 0;

            while offset < data.len() {
                if data[offset + 257..offset + 257 + 5] != *b"ustar" {
                    error!("tarfs: invalid header at offset {}, stopping...", offset);
                    break;
                }

                let file_size = oct_to_bin(&data[offset + 0x7c..offset + 0x7c + 11]);
                let mut path = String::from_utf8_lossy(&data[offset..offset + 100])
                    .trim_matches(char::from(0))
                    .to_string();

                // resolve path
                if path.starts_with(".") {
                    path = path[1..].to_string();
                } else {
                    path = "/".to_string() + &path;
                }

                if file_size > 0 {
                    files.push(TarFile {
                        path,
                        position: offset + 512,
                        size: file_size,
                        offset: AtomicUsize::new(0),
                        fs: None,
                    })
                }

                offset += (((file_size + 511) / 512) + 1) * 512;
            }
        }

        Self {
            addr: addr.unwrap(),
            files,
        }
    }
}

impl VFSImplementation for TarFS {
    fn open(
        &self,
        path: &str,
        _flags: u32,
    ) -> super::VFSResult<alloc::boxed::Box<dyn super::VFSFile>> {
        if let Some(file) = self.files.iter().find(|f| f.path == path) {
            // NOTE: can't i just clone this ?
            Ok(Box::new(TarFile {
                path: file.path.clone(),
                position: file.position,
                size: file.size,
                offset: AtomicUsize::new(file.offset.load(Ordering::Acquire)),
                fs: Some(self),
            }))
        } else {
            Err(VFSError::NotFound)
        }
    }

    fn metadata(&self, path: &str) -> VFSResult<VFSMetadata> {
        if let Some(file) = self.files.iter().find(|f| f.path == path) {
            Ok(VFSMetadata {
                file_type: super::types::VFSFileType::File,
                size: file.size,
            })
        } else {
            Err(VFSError::NotFound)
        }
    }
}

impl Default for TarFS {
    fn default() -> Self {
        Self::new()
    }
}

fn oct_to_bin(bytes: &[u8]) -> usize {
    let mut n: usize = 0;
    for &b in bytes {
        n *= 8;
        n += (b - b'0') as usize;
    }
    n
}
