use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use spin::Mutex;

use crate::vfs::path::components;
use crate::vfs::tarfs::node::{TarInode, TarInodeType};
use crate::vfs::traits::{FileSystem, Inode};
use crate::vfs::{Metadata, VfsResult};
mod node;

struct TarEntry {
    kind: TarInodeType,
    data: Arc<Vec<u8>>,
    link: Option<String>,
}

pub struct TarFs {
    root: Arc<TarInode>,
}

impl TarFs {
    pub fn new() -> Self {
        let ids = Arc::new(AtomicU64::new(2));
        Self {
            root: Arc::new(TarInode {
                inode: 1,
                kind: TarInodeType::Directory,
                ids,
                data: Arc::new(Vec::new()),
                children: Arc::new(Mutex::new(BTreeMap::new())),
                link: None,
            }),
        }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let ids = Arc::new(AtomicU64::new(2));
        let root = Arc::new(TarInode {
            inode: 1,
            kind: TarInodeType::Directory,
            ids: ids.clone(),
            data: Arc::new(Vec::new()),
            children: Arc::new(Mutex::new(BTreeMap::new())),
            link: None,
        });

        let mut entries = BTreeMap::new();
        let mut offset = 0usize;

        while offset + 512 <= data.len() {
            let header = &data[offset..offset + 512];
            if header.iter().all(|&b| b == 0) {
                break;
            }

            if header[257..257 + 5] != *b"ustar" {
                log::error!("Invalid tar header magic");
                break;
            }

            // based on https://wiki.osdev.org/USTAR
            let file_name = String::from_utf8_lossy(&header[..100])
                .trim_matches(char::from(0))
                .to_string();
            let file_size = Self::oct_to_bin(&header[124..124 + 12]);
            let file_checksum = Self::oct_to_bin(&header[148..148 + 8]);
            let file_type = TarInodeType::from(header[156]);
            let file_linkname =
                String::from_utf8_lossy(&header[157..157 + 100])
                    .trim_matches(char::from(0))
                    .to_string();
            let file_prefix = String::from_utf8_lossy(&header[345..345 + 155])
                .trim_matches(char::from(0))
                .to_string();

            let sum: usize =
                header
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

            let full_name = if file_prefix.is_empty() {
                file_name.clone()
            } else {
                alloc::format!("{}/{}", file_prefix, file_name)
            };
            let path = Self::normalize_tar_path(&full_name);

            let data_position = match offset.checked_add(512) {
                Some(position) => position,
                None => break,
            };
            let data_end = match data_position.checked_add(file_size) {
                Some(end) => end,
                None => {
                    log::error!(
                        "tarfs: file {} size causes overflow, skipping...",
                        path
                    );
                    break;
                },
            };

            if data_end > data.len() {
                log::error!(
                    "tarfs: file {} size exceeds archive size, skipping...",
                    path
                );
                break;
            }

            let blocks = match file_size.checked_add(511) {
                Some(size) => size / 512,
                None => {
                    log::error!(
                        "tarfs: file {} size causes overflow, skipping...",
                        path
                    );
                    break;
                },
            };
            let padded_size = match blocks.checked_mul(512) {
                Some(size) => size,
                None => {
                    log::error!(
                        "tarfs: file {} size causes overflow, skipping...",
                        path
                    );
                    break;
                },
            };
            let next = match offset
                .checked_add(512)
                .and_then(|next| next.checked_add(padded_size))
            {
                Some(next) => next,
                None => {
                    log::error!(
                        "tarfs: file {} size causes overflow, skipping...",
                        path
                    );
                    break;
                },
            };

            let should_track = matches!(
                file_type,
                TarInodeType::File
                    | TarInodeType::HardLink
                    | TarInodeType::Symlink
                    | TarInodeType::Directory
            );

            if should_track {
                log::trace!(
                    "tarfs: adding {} (type: {:?}, size: {}, offset: {})",
                    path,
                    file_type,
                    file_size,
                    data_position
                );

                let link = match file_type {
                    TarInodeType::Symlink => {
                        Some(Self::normalize_link_target(&file_linkname))
                    },
                    TarInodeType::HardLink => {
                        Some(Self::normalize_tar_path(&file_linkname))
                    },
                    _ => None,
                };

                let file_data = if file_type == TarInodeType::File {
                    Arc::new(data[data_position..data_end].to_vec())
                } else {
                    Arc::new(Vec::new())
                };

                entries.insert(
                    path,
                    TarEntry { kind: file_type, data: file_data, link },
                );
            }

            offset = next;
        }

        for (path, entry) in &entries {
            if path == "/" {
                continue;
            }

            let comps = components(path);
            let Some((name, parents)) = comps.split_last() else {
                continue;
            };

            if entry.kind == TarInodeType::Directory {
                Self::ensure_directory(&root, &comps, &ids);
                continue;
            }

            let file_data = if entry.kind == TarInodeType::HardLink {
                match Self::resolve_hardlink_data(&entries, path) {
                    Some(data) => data,
                    None => {
                        log::error!(
                            "tarfs: unresolved hard link {}, skipping...",
                            path
                        );
                        continue;
                    },
                }
            } else {
                entry.data.clone()
            };

            let parent = Self::ensure_directory(&root, parents, &ids);
            let node = Self::make_node(
                &ids,
                entry.kind,
                file_data,
                entry.link.clone(),
            );
            parent.children.lock().insert(String::from(*name), node);
        }

        Self { root }
    }

    fn make_node(
        ids: &Arc<AtomicU64>,
        kind: TarInodeType,
        data: Arc<Vec<u8>>,
        link: Option<String>,
    ) -> Arc<TarInode> {
        Arc::new(TarInode {
            inode: ids.fetch_add(1, Ordering::Relaxed),
            kind,
            ids: ids.clone(),
            data,
            children: Arc::new(Mutex::new(BTreeMap::new())),
            link,
        })
    }

    fn ensure_directory(
        root: &Arc<TarInode>,
        components: &[&str],
        ids: &Arc<AtomicU64>,
    ) -> Arc<TarInode> {
        let mut current = root.clone();

        for component in components {
            let mut children = current.children.lock();
            let next = match children.get(*component) {
                Some(child) => child.clone(),
                None => {
                    let child = Self::make_node(
                        ids,
                        TarInodeType::Directory,
                        Arc::new(Vec::new()),
                        None,
                    );
                    children.insert(String::from(*component), child.clone());
                    child
                },
            };
            drop(children);

            current = next;
        }

        current
    }

    fn resolve_hardlink_data(
        entries: &BTreeMap<String, TarEntry>,
        path: &str,
    ) -> Option<Arc<Vec<u8>>> {
        let mut current = path;

        for _ in 0..8 {
            let entry = entries.get(current)?;
            match entry.kind {
                TarInodeType::File => return Some(entry.data.clone()),
                TarInodeType::HardLink => {
                    current = entry.link.as_deref()?;
                },
                _ => return None,
            }
        }

        None
    }

    pub fn oct_to_bin(bytes: &[u8]) -> usize {
        let s = core::str::from_utf8(bytes)
            .unwrap_or("")
            .trim_end_matches(['\0', ' ']);

        if s.is_empty() {
            return 0;
        }
        usize::from_str_radix(s, 8).unwrap_or(0)
    }

    fn normalize_tar_path(name: &str) -> String {
        let mut parts: Vec<&str> = Vec::new();

        for part in name.split('/') {
            match part {
                "" | "." => {},
                ".." => {
                    parts.pop();
                },
                other => parts.push(other),
            }
        }

        if parts.is_empty() {
            "/".to_string()
        } else {
            alloc::format!("/{}", parts.join("/"))
        }
    }

    fn normalize_link_target(name: &str) -> String {
        let absolute = name.starts_with('/');
        let mut parts: Vec<&str> = Vec::new();

        for part in name.split('/') {
            match part {
                "" | "." => {},
                ".." => {
                    if parts.last().is_some_and(|last| *last != "..") {
                        parts.pop();
                    } else if !absolute {
                        parts.push("..");
                    }
                },
                other => parts.push(other),
            }
        }

        let joined = parts.join("/");
        if absolute {
            alloc::format!("/{}", joined)
        } else if joined.is_empty() {
            String::from(".")
        } else {
            joined
        }
    }
}

impl Default for TarFs {
    fn default() -> Self { Self::new() }
}

impl FileSystem for TarFs {
    fn name(&self) -> &'static str { "tarfs" }

    fn root(&self) -> VfsResult<Arc<dyn Inode>> { Ok(self.root.clone()) }

    fn metadata(&self) -> VfsResult<Metadata> { self.root.metadata() }

    fn sync(&self) -> VfsResult<()> { Ok(()) }
}
