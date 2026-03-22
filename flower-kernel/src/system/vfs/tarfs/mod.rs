use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::boot::limine::MODULE_REQUESTS;
use crate::error;
use crate::system::vfs::types::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TarFSFileType {
    File = 0,
    HardLink = 1,
    Symlink = 2,
    CharacterDevice = 3,
    BlockDevice = 4,
    Directory = 5,
    Fifo = 6,
    Unknown = 7,
}

impl From<u8> for TarFSFileType {
    fn from(value: u8) -> Self {
        match value {
            b'0' => TarFSFileType::File,
            b'1' => TarFSFileType::HardLink,
            b'2' => TarFSFileType::Symlink,
            b'3' => TarFSFileType::CharacterDevice,
            b'4' => TarFSFileType::BlockDevice,
            b'5' => TarFSFileType::Directory,
            b'6' => TarFSFileType::Fifo,
            _ => TarFSFileType::Unknown,
        }
    }
}

#[derive(Clone)]
struct TarFile {
    _data_position: usize,
    _position: usize,
    _data: Arc<Vec<u8>>,

    name: String,
    path: String,
    mode: usize,
    owner_id: usize,
    group_id: usize,
    size: usize,
    last_modified: usize,
    checksum: usize,
    file_type: TarFSFileType,
    owner_name: String,
    group_name: String,
    device_major: usize,
    device_minor: usize,
    prefix: String,
}

impl VFSFile for TarFile {
    fn read(&self, buf: &mut [u8]) -> VFSResult<usize> {
        let position = self._position;

        if position >= self.size {
            return Ok(0);
        }

        let bytes_to_read = core::cmp::min(buf.len(), self.size - position);
        let source =
            unsafe { self._data.as_ptr().add(self._data_position + position) };

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
            VFSSeek::Current(n) => self._position.saturating_add(n),
            VFSSeek::End(n) => self.size.saturating_add(n),
        };

        new_pos = core::cmp::min(new_pos, self.size);

        self._position = new_pos;
        Ok(self._position)
    }

    fn metadata(&self) -> VFSResult<VFSMetadata> {
        let typ = match self.file_type {
            TarFSFileType::File => VFSFileType::File,
            TarFSFileType::Directory => VFSFileType::Directory,
            TarFSFileType::CharacterDevice | TarFSFileType::BlockDevice => {
                VFSFileType::Device
            },
            TarFSFileType::Symlink => VFSFileType::Symlink,
            TarFSFileType::Fifo => VFSFileType::Pipe,
            _ => VFSFileType::Unknown,
        };

        Ok(VFSMetadata {
            name: self.name.clone(),
            mode: self.mode,
            typ,
            last_modified: self.last_modified,
            size: self.size,
        })
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

                if header[257..257 + 5] != *b"ustar" {
                    break;
                }

                // pretty much parse everything we can
                // based on https://wiki.osdev.org/USTAR
                let file_name = String::from_utf8_lossy(&header[..100])
                    .trim_matches(char::from(0))
                    .to_string();
                let file_mode = oct_to_bin(&header[100..100 + 8]);
                let file_owner_id = oct_to_bin(&header[108..108 + 8]);
                let file_group_id = oct_to_bin(&header[116..116 + 8]);
                let file_size = oct_to_bin(&header[124..124 + 12]);
                let file_last_modified = oct_to_bin(&header[136..136 + 12]);
                let file_checksum = oct_to_bin(&header[148..148 + 8]);
                let file_type = header[156];
                let file_owner_name =
                    String::from_utf8_lossy(&header[265..265 + 32])
                        .trim_matches(char::from(0))
                        .to_string();
                let file_group_name =
                    String::from_utf8_lossy(&header[297..297 + 32])
                        .trim_matches(char::from(0))
                        .to_string();
                let file_device_major = oct_to_bin(&header[329..329 + 8]);
                let file_device_minor = oct_to_bin(&header[337..337 + 8]);
                let file_prefix =
                    String::from_utf8_lossy(&header[345..345 + 155])
                        .trim_matches(char::from(0))
                        .to_string();

                let path = "/".to_string() + &file_name;

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
                        _data_position: data_position,
                        _position: 0,
                        _data: Arc::clone(&data),
                        name: file_name
                            .split("/")
                            .last()
                            .unwrap_or(file_name.as_str())
                            .to_string(),
                        path,
                        mode: file_mode,
                        owner_id: file_owner_id,
                        group_id: file_group_id,
                        size: file_size,
                        last_modified: file_last_modified,
                        checksum: file_checksum,
                        file_type: TarFSFileType::from(file_type),
                        owner_name: file_owner_name,
                        group_name: file_group_name,
                        device_major: file_device_major,
                        device_minor: file_device_minor,
                        prefix: file_prefix,
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
        Ok(Box::new(file.clone()))
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
