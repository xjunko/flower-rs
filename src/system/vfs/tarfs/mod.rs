use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

use crate::{boot::limine::MODULE_REQUESTS, error, system::vfs::types::*};

unsafe impl Send for TarFile {}
unsafe impl Sync for TarFile {}
struct TarFile {
    data_position: usize,
    position: usize,
    path: String,
    size: usize,
    fs: Option<*const TarFS>,
}

impl VFSFile for TarFile {
    fn read(&self, buf: &mut [u8]) -> VFSResult<usize> {
        self.fs.ok_or(VFSError::NotFound).map(|fs| {
            let fs = unsafe { &*fs };
            let position = self.position;

            let bytes_to_read = core::cmp::min(buf.len(), self.size);
            let source = unsafe { fs.data.as_ptr().add(position) };

            unsafe {
                core::ptr::copy_nonoverlapping(source, buf.as_mut_ptr(), bytes_to_read);
            }

            bytes_to_read
        })
    }

    fn write(&self, _buf: &mut [u8]) -> VFSResult<usize> {
        unimplemented!()
    }

    fn seek(&mut self, pos: VFSSeek) -> VFSResult<usize> {
        let mut new_pos = match pos {
            VFSSeek::Start(n) => self.data_position as isize + n as isize,
            VFSSeek::Current(n) => n as isize,
            VFSSeek::End(n) => self.data_position as isize + self.size as isize + n as isize,
        };

        if new_pos < 0 {
            return Err(VFSError::InvalidSeek);
        }

        // clamp inside the data_position and data_position + size
        new_pos = core::cmp::min(
            core::cmp::max(self.data_position, new_pos as usize),
            self.data_position + self.size,
        ) as isize;

        self.position = new_pos as usize;
        Ok(self.position)
    }

    fn metadata(&self) -> VFSResult<VFSMetadata> {
        Ok(VFSMetadata {
            file_type: VFSFileType::File,
            size: self.size,
        })
    }
}

unsafe impl Send for TarFS {}
unsafe impl Sync for TarFS {}
pub struct TarFS {
    data: Vec<u8>,
    files: Vec<TarFile>,
}

impl TarFS {
    pub fn new() -> Self {
        let mut files = Vec::new();
        let file = {
            MODULE_REQUESTS
                .get_response()
                .expect("no modules provider")
                .modules()
                .iter()
                .find(|m| {
                    m.path()
                        .to_str()
                        .map(|path| path == "/boot/initramfs.tar")
                        .unwrap_or(false)
                })
        }
        .ok_or("failed to find initramfs");

        if let Err(e) = file {
            error!("tarfs: {}", e);
            panic!("failed to initialize tarfs");
        }

        if let Ok(file) = file {
            // stream the file into memory
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

                // copy the data right away
                if file_size > 0 {
                    files.push(TarFile {
                        data_position: offset + 512,
                        position: 0,
                        path: path.clone(),
                        size: file_size,
                        fs: None,
                    });
                }

                offset += (((file_size + 511) / 512) + 1) * 512;
            }

            return Self { data, files };
        }

        Self {
            data: Vec::new(),
            files: Vec::new(),
        }
    }

    fn get_file(&self, path: &str) -> VFSResult<&TarFile> {
        self.files
            .iter()
            .find(|f| f.path == path)
            .ok_or(VFSError::NotFound)
    }
}

impl VFSImplementation for TarFS {
    fn open(&self, path: &str, _flags: u32) -> VFSResult<alloc::boxed::Box<dyn VFSFile>> {
        let file = self.get_file(path)?;
        Ok(Box::new(TarFile {
            data_position: file.data_position,
            position: 0,
            path: file.path.clone(),
            size: file.size,
            fs: Some(self as *const TarFS),
        }))
    }

    fn metadata(&self, path: &str) -> VFSResult<VFSMetadata> {
        let file = self.get_file(path)?;
        file.metadata()
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
