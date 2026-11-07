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

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::system::posix::errno::{KError, KResult};
use crate::system::vfs::tarfs::file::{TarFile, TarFileType};
use crate::system::vfs::{Directory, KFile};

pub struct TarFSDirectory {
    pub name: String,
    pub path: String,
    /// Shared with the owning `TarFS` so directories don't need their own
    /// copy of the whole archive's file table.
    pub files: Arc<Vec<TarFile>>,
}

impl TarFSDirectory {
    fn child_prefix(&self) -> String {
        if self.path == "/" {
            "/".into()
        } else {
            alloc::format!("{}/", self.path)
        }
    }
}

impl Directory for TarFSDirectory {
    fn name(&self) -> KResult<String> { Ok(self.name.clone()) }

    fn contents(&self) -> KResult<Vec<KFile>> {
        let prefix = self.child_prefix();

        let children = self.files.iter().filter(|f| {
            f.path != self.path
                && f.path.starts_with(prefix.as_str())
                // only direct children, not grandchildren
                && !f.path[prefix.len()..].contains('/')
        });

        Ok(children
            .map(|file| match file.file_type {
                TarFileType::Directory => {
                    KFile::Directory(Arc::new(TarFSDirectory {
                        name: file.name.clone(),
                        path: file.path.clone(),
                        files: Arc::clone(&self.files),
                    }))
                },
                _ => KFile::File(Arc::new(file.clone())),
            })
            .collect())
    }

    fn delete(&self, _name: &str) -> KResult<()> {
        // read-only fs, consistent with TarFile::write
        Err(KError::EPERM)
    }
}
