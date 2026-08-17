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

use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::system::vfs::error::{VfsError, VfsResult};
use crate::system::vfs::inode::{DirectoryEntry, FileType, Inode, Metadata};
use crate::system::vfs::perm::Permissions;
use crate::system::vfs::tarfs::TarEntries;
use crate::system::vfs::tarfs::file::{TarFile, TarFsFileType, to_file_type};

pub struct TarFsDirectory {
    path: String,
    inode_id: u64,
    entry: Option<Arc<TarFile>>,
    entries: Arc<TarEntries>,
}

impl TarFsDirectory {
    pub(crate) fn new(
        path: String,
        entry: Option<Arc<TarFile>>,
        entries: Arc<TarEntries>,
    ) -> Self {
        let inode_id = entry.as_ref().map(|f| f.inode).unwrap_or(0);
        Self { path, inode_id, entries, entry }
    }

    fn child_prefix(&self) -> String {
        if self.path == "/" {
            "/".to_string()
        } else {
            format!("{}/", self.path)
        }
    }
}

pub(crate) fn make_inode(
    path: String,
    file: Arc<TarFile>,
    entries: Arc<TarEntries>,
) -> Arc<dyn Inode> {
    if file.file_type == TarFsFileType::Directory {
        Arc::new(TarFsDirectory::new(path, Some(file), entries))
    } else {
        file
    }
}

impl Inode for TarFsDirectory {
    fn metadata(&self) -> VfsResult<Metadata> {
        match &self.entry {
            Some(file) => file.metadata(),
            // synthetic root: the archive had no explicit "/" header.
            None => Ok(Metadata {
                inode: self.inode_id,
                size: 0,
                file_type: FileType::Directory,
                permissions: Permissions::from_unix(0o755),
                owner: 0,
                group: 0,
                links: 2,
                last_modified: 0,
            }),
        }
    }

    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn Inode>> {
        let child_path = format!("{}{}", self.child_prefix(), name);
        let file =
            self.entries.get(&child_path).cloned().ok_or(VfsError::NotFound)?;
        Ok(make_inode(child_path, file, self.entries.clone()))
    }

    fn readdir(&self) -> VfsResult<Vec<DirectoryEntry>> {
        let prefix = self.child_prefix();
        let mut out = Vec::new();

        for (path, file) in self.entries.range(prefix.clone()..) {
            if !path.starts_with(prefix.as_str()) {
                break;
            }

            let rest = &path[prefix.len()..];
            if rest.is_empty() || rest.contains('/') {
                continue;
            }

            out.push(DirectoryEntry {
                name: rest.to_string(),
                inode: file.inode,
                file_type: to_file_type(file.file_type),
            });
        }

        Ok(out)
    }
}
