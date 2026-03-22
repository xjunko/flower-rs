mod consts;
mod file;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::AtomicUsize;

use crate::boot::limine::MODULE_REQUESTS;
use crate::error;
use crate::system::vfs::tarfs::consts::*;
use crate::system::vfs::tarfs::file::{TarFSFileType, TarFile};
use crate::system::vfs::types::*;

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
                let file_type = TarFSFileType::from(header[156]);
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

                let sum: usize = header
                    .iter()
                    .enumerate()
                    .map(|(i, &b)| {
                        if (148..156).contains(&i) { 0x20 } else { b as usize }
                    })
                    .sum();

                if sum != file_checksum {
                    error!(
                        "tarfs: checksum mismatch for file {}, skipping...",
                        file_name
                    );
                    offset += 512;
                    continue;
                }

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
                        _position: AtomicUsize::new(0),
                        _data: Arc::clone(&data),
                        name: file_name
                            .split("/")
                            .last()
                            .unwrap_or(file_name.as_str())
                            .to_string(),
                        path,
                        mode: ustar_to_unix(file_mode, file_type),
                        owner_id: file_owner_id,
                        group_id: file_group_id,
                        size: file_size,
                        last_modified: file_last_modified,
                        checksum: file_checksum,
                        file_type,
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
    let s =
        core::str::from_utf8(bytes).unwrap_or("").trim_end_matches(['\0', ' ']);

    if s.is_empty() {
        return 0;
    }
    usize::from_str_radix(s, 8).unwrap_or(0)
}

fn ustar_to_unix(mode: usize, typ: TarFSFileType) -> usize {
    let ftype = match typ {
        TarFSFileType::File => S_IFREG,
        TarFSFileType::Directory => S_IFDIR,
        TarFSFileType::CharacterDevice => S_IFCHR,
        TarFSFileType::BlockDevice => S_IFBLK,
        TarFSFileType::Fifo => S_IFIFO,
        TarFSFileType::Symlink => S_IFLNK,
        _ => 0,
    };

    let perms = mode & 0o777;
    ftype | perms
}
