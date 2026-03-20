use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::boot::limine::MODULE_REQUESTS;
use crate::error;
use crate::system::vfs::types::*;

struct TarFile {
    data_position: usize,
    position: usize,
    path: String,
    size: usize,
    data: Arc<Vec<u8>>,
}

impl VFSFile for TarFile {
    fn read(&self, buf: &mut [u8]) -> VFSResult<usize> {
        let position = self.position;

        if position >= self.size {
            return Ok(0);
        }

        let bytes_to_read = core::cmp::min(buf.len(), self.size - position);
        let source =
            unsafe { self.data.as_ptr().add(self.data_position + position) };

        unsafe {
            core::ptr::copy_nonoverlapping(
                source,
                buf.as_mut_ptr(),
                bytes_to_read,
            );
        }

        Ok(bytes_to_read)
    }

    fn write(&self, _buf: &mut [u8]) -> VFSResult<usize> { unimplemented!() }

    fn seek(&mut self, pos: VFSSeek) -> VFSResult<usize> {
        let mut new_pos = match pos {
            VFSSeek::Start(n) => n,
            VFSSeek::Current(n) => self.position.saturating_add(n),
            VFSSeek::End(n) => self.size.saturating_add(n),
        };

        new_pos = core::cmp::min(new_pos, self.size);

        self.position = new_pos;
        Ok(self.position)
    }

    fn metadata(&self) -> VFSResult<VFSMetadata> {
        Ok(VFSMetadata { file_type: VFSFileType::File, size: self.size })
    }
}

pub struct TarFS {
    data: Arc<Vec<u8>>,
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
                core::ptr::copy_nonoverlapping(
                    file.addr(),
                    data.as_mut_ptr(),
                    size,
                );
            }
            let data = Arc::new(data);

            // read all the files
            let mut offset = 0;

            while offset + 512 <= data.len() {
                let header = &data[offset..offset + 512];

                if header.iter().all(|&b| b == 0) {
                    break;
                }

                if data[offset + 257..offset + 257 + 5] != *b"ustar" {
                    error!(
                        "tarfs: invalid header at offset {}, stopping...",
                        offset
                    );
                    break;
                }

                let file_size = oct_to_bin(&header[0x7c..0x7c + 11]);
                let mut path = String::from_utf8_lossy(&header[..100])
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
                    let data_position = offset + 512;
                    if data_position + file_size > data.len() {
                        error!(
                            "tarfs: file {} exceeds archive bounds, stopping...",
                            path
                        );
                        break;
                    }

                    files.push(TarFile {
                        data_position,
                        position: 0,
                        path: path.clone(),
                        size: file_size,
                        data: Arc::clone(&data),
                    });
                }

                let next = (((file_size + 511) / 512) + 1) * 512;
                offset = match offset.checked_add(next) {
                    Some(value) => value,
                    None => {
                        error!("tarfs: archive offset overflow, stopping...");
                        break;
                    },
                };
            }

            return Self { data, files };
        }

        Self { data: Arc::new(Vec::new()), files: Vec::new() }
    }

    fn get_file(&self, path: &str) -> VFSResult<&TarFile> {
        self.files.iter().find(|f| f.path == path).ok_or(VFSError::NotFound)
    }
}

impl VFSImplementation for TarFS {
    fn open(
        &self,
        path: &str,
        _flags: u32,
    ) -> VFSResult<alloc::boxed::Box<dyn VFSFile>> {
        let file = self.get_file(path)?;
        Ok(Box::new(TarFile {
            data_position: file.data_position,
            position: 0,
            path: file.path.clone(),
            size: file.size,
            data: Arc::clone(&self.data),
        }))
    }

    fn metadata(&self, path: &str) -> VFSResult<VFSMetadata> {
        let file = self.get_file(path)?;
        file.metadata()
    }
}

impl Default for TarFS {
    fn default() -> Self { Self::new() }
}

fn oct_to_bin(bytes: &[u8]) -> usize {
    let mut n: usize = 0;
    for &b in bytes {
        if b == 0 || b == b' ' {
            break;
        }

        if !(b'0'..=b'7').contains(&b) {
            break;
        }

        n = n.saturating_mul(8).saturating_add((b - b'0') as usize);
    }
    n
}
