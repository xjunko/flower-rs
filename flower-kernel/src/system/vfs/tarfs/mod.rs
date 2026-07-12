mod consts;
mod directory;
mod file;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::AtomicUsize;

use crate::boot::limine::MODULE_REQUESTS;
use crate::system::vfs::tarfs::consts::*;
use crate::system::vfs::tarfs::directory::TarFSDirectory;
use crate::system::vfs::tarfs::file::{TarFSFileType, TarFile};
use crate::system::vfs::types::*;

pub struct TarFS {
    files: Arc<Vec<TarFile>>,
    index: BTreeMap<String, usize>,
}

impl TarFS {
    pub fn new() -> Self {
        Self { files: Arc::new(Vec::new()), index: BTreeMap::new() }
    }

    fn get_file(&self, path: &str) -> VFSResult<&TarFile> {
        self.index
            .get(path)
            .and_then(|&i| self.files.get(i))
            .ok_or(VFSError::NotFound)
    }

    fn directory_filelike(&self, file: &TarFile) -> VFSFilelike {
        VFSFilelike::Directory(Box::new(TarFSDirectory {
            name: file.name.clone(),
            path: file.path.clone(),
            files: Arc::clone(&self.files),
        }))
    }
}

impl VFSImplementation for TarFS {
    fn initialize(&mut self) -> VFSResult<()> {
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
            log::error!("tarfs: {}", e);
            panic!("failed to initialize tarfs");
        }

        if let Ok(file) = file {
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

            let mut files: Vec<TarFile> = Vec::new();
            let mut offset = 0;

            while offset + 512 <= data.len() {
                let header = &data[offset..offset + 512];
                if header.iter().all(|&b| b == 0) {
                    break;
                }

                if header[257..257 + 5] != *b"ustar" {
                    break;
                }

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
                let file_linkname =
                    String::from_utf8_lossy(&header[157..157 + 100])
                        .trim_matches(char::from(0))
                        .to_string();
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
                    log::error!(
                        "tarfs: checksum mismatch for file {}, skipping...",
                        file_name
                    );
                    offset += 512;
                    continue;
                }

                // USTAR splits long paths across `name` and `prefix`
                // (prefix + "/" + name); ignoring `prefix` truncates any
                // path over 100 bytes.
                let full_name = if file_prefix.is_empty() {
                    file_name.clone()
                } else {
                    alloc::format!("{}/{}", file_prefix, file_name)
                };
                let path = normalize_tar_path(&full_name);

                // Symlinks/hardlinks carry no data blocks (size is 0), so
                // the old `file_size > 0` gate silently dropped them.
                let should_track = matches!(
                    file_type,
                    TarFSFileType::Directory
                        | TarFSFileType::Symlink
                        | TarFSFileType::HardLink
                ) || file_size > 0;

                if should_track {
                    let data_position = offset + 512;
                    if file_size > 0 && data_position + file_size > data.len() {
                        log::error!(
                            "tarfs: file {} exceeds archive bounds, stopping...",
                            path
                        );
                        break;
                    }

                    log::info!(
                        "tarfs: loaded type={:?} {} ({} bytes)",
                        file_type,
                        path,
                        file_size
                    );

                    files.push(TarFile {
                        _data_position: data_position,
                        _position: AtomicUsize::new(0),
                        _data: Arc::clone(&data),
                        name: file_name
                            .trim_end_matches('/')
                            .split('/')
                            .next_back()
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
                        linkname: file_linkname,
                    });
                }

                let next = (((file_size + 511) / 512) + 1) * 512;
                offset = match offset.checked_add(next) {
                    Some(value) => value,
                    None => {
                        log::error!(
                            "tarfs: archive offset overflow, stopping..."
                        );
                        break;
                    },
                };
            }

            self.index = files
                .iter()
                .enumerate()
                .map(|(i, f)| (f.path.clone(), i))
                .collect();
            self.files = Arc::new(files);
        }

        Ok(())
    }

    fn open(&self, path: &str, _flags: u32) -> VFSResult<VFSFilelike> {
        if path == "/" && !self.index.contains_key("/") {
            return Ok(VFSFilelike::Directory(Box::new(TarFSDirectory {
                name: String::new(),
                path: "/".to_string(),
                files: Arc::clone(&self.files),
            })));
        }

        let file = self.get_file(path)?;

        match file.file_type {
            TarFSFileType::Directory => Ok(self.directory_filelike(file)),
            _ => Ok(VFSFilelike::File(Box::new(file.clone()))),
        }
    }

    fn metadata(&self, path: &str) -> VFSResult<VFSMetadata> {
        let file = self.get_file(path)?;
        file.metadata()
    }
}

impl Default for TarFS {
    fn default() -> Self { Self::new() }
}

fn normalize_tar_path(name: &str) -> String {
    let trimmed = name.trim_end_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else if let Some(stripped) = trimmed.strip_prefix('/') {
        alloc::format!("/{}", stripped)
    } else {
        alloc::format!("/{}", trimmed)
    }
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
