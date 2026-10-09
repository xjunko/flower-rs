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
