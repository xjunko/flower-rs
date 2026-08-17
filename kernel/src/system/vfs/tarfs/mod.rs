/*
 * ISC License
 *
 * Copyright (c) 2025-2026 xjunko
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
 * REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
 * AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
 * INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
 * LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
 * OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
 * PERFORMANCE OF THIS SOFTWARE.
 */

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;

use crate::boot::limine::MODULE_REQUESTS;
use crate::system::vfs::fs::FileSystem;
use crate::system::vfs::inode::Inode;
use crate::system::vfs::tarfs::directory::TarFsDirectory;
use crate::system::vfs::tarfs::file::{TarFile, TarFsFileType};

mod directory;
mod file;

pub(crate) type TarEntries = BTreeMap<String, Arc<TarFile>>;

pub struct TarFs {
    pub entries: Arc<TarEntries>,
}

impl TarFs {
    pub fn create(module_name: &str) -> Self {
        let mut entries: TarEntries = BTreeMap::new();
        let mut next_inode = 1;
        let mut probable_ramfs: Option<&&limine::file::File> = None;

        if let Some(resp) = MODULE_REQUESTS.response() {
            for module in resp.modules() {
                if module.path().to_string().eq(module_name) {
                    probable_ramfs = Some(module);
                    break;
                }
            }
        }

        if probable_ramfs.is_none() {
            log::warn!(
                "no {} found in modules, tarfs will be empty",
                module_name
            );
            return Self { entries: Arc::new(entries) };
        }

        if let Some(file) = probable_ramfs {
            let size = file.data().len();
            let mut data = alloc::vec![0u8; size];
            unsafe {
                core::ptr::copy_nonoverlapping(
                    file.data().as_ptr(),
                    data.as_mut_ptr(),
                    size,
                );
            }

            let data = Arc::new(data);
            let mut offset = 0;

            while offset + 512 <= data.len() {
                let header = &data.as_ref()[offset..offset + 512];
                if header.iter().all(|&b| b == 0) {
                    break;
                }

                if header[257..257 + 5] != *b"ustar" {
                    log::warn!("invalid tar header at offset {}", offset);
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
                let file_type = TarFsFileType::from(header[156]);
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

                // ustar does weird stuff with long paths
                let full_name = if file_prefix.is_empty() {
                    file_name.clone()
                } else {
                    alloc::format!("{}/{}", file_prefix, file_name)
                };
                let path = normalize_tar_path(&full_name);

                // symlinks
                let should_track = matches!(
                    file_type,
                    TarFsFileType::Directory
                        | TarFsFileType::Symlink
                        | TarFsFileType::HardLink
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

                    let inode = next_inode;
                    next_inode += 1;

                    entries.insert(
                        path.clone(),
                        Arc::new(TarFile {
                            _data_position: data_position,
                            _data: Arc::clone(&data),
                            inode,
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
                        }),
                    );
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
        }

        Self { entries: Arc::new(entries) }
    }
}

impl FileSystem for TarFs {
    fn name(&self) -> &str { "tarfs" }

    fn root(&self) -> Arc<dyn Inode> {
        let entry = self.entries.get("/").cloned();
        Arc::new(TarFsDirectory::new(
            "/".to_string(),
            entry,
            self.entries.clone(),
        ))
    }

    fn sync(&self) -> super::error::VfsResult<()> { todo!() }
}

impl Default for TarFs {
    fn default() -> Self { Self::create("/boot/init.tar") }
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

fn ustar_to_unix(mode: usize, typ: TarFsFileType) -> usize {
    const S_IFMT: usize = 0o170000; // bitmask for file type
    const S_IFREG: usize = 0o100000; // regular file
    const S_IFDIR: usize = 0o040000; // directory
    const S_IFCHR: usize = 0o020000; // char device
    const S_IFBLK: usize = 0o060000; // block device
    const S_IFIFO: usize = 0o010000; // fifo/pipe
    const S_IFLNK: usize = 0o120000; // symlink

    let ftype = match typ {
        TarFsFileType::File => S_IFREG,
        TarFsFileType::Directory => S_IFDIR,
        TarFsFileType::CharacterDevice => S_IFCHR,
        TarFsFileType::BlockDevice => S_IFBLK,
        TarFsFileType::Fifo => S_IFIFO,
        TarFsFileType::Symlink => S_IFLNK,
        _ => 0,
    };

    let perms = mode & 0o777;
    ftype | perms
}
