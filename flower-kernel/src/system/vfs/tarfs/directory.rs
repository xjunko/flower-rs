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

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::system::vfs::tarfs::file::{TarFSFileType, TarFile};
use crate::system::vfs::{VFSDirectory, VFSError, VFSFilelike, VFSResult};

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

impl VFSDirectory for TarFSDirectory {
    fn name(&self) -> VFSResult<String> { Ok(self.name.clone()) }

    fn contents(&self) -> VFSResult<Vec<VFSFilelike>> {
        let prefix = self.child_prefix();

        let children = self.files.iter().filter(|f| {
            f.path != self.path
                && f.path.starts_with(prefix.as_str())
                // only direct children, not grandchildren
                && !f.path[prefix.len()..].contains('/')
        });

        Ok(children
            .map(|file| match file.file_type {
                TarFSFileType::Directory => {
                    VFSFilelike::Directory(Box::new(TarFSDirectory {
                        name: file.name.clone(),
                        path: file.path.clone(),
                        files: Arc::clone(&self.files),
                    }))
                },
                _ => VFSFilelike::File(Box::new(file.clone())),
            })
            .collect())
    }

    fn delete(&self, _name: &str) -> VFSResult<()> {
        // read-only fs, consistent with TarFile::write
        Err(VFSError::Unsupported)
    }
}
