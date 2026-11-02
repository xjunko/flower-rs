use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::AtomicU64;

use spin::Mutex;

use crate::vfs::file::OpenFlags;
use crate::vfs::traits::{FileOps, Inode};
use crate::vfs::{FileType, Metadata, VfsError, VfsResult};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum TarInodeType {
    File = 0,
    HardLink = 1,
    Symlink = 2,
    CharacterDevice = 3,
    BlockDevice = 4,
    Directory = 5,
    Fifo = 6,
    Unknown = 7,
}

impl From<u8> for TarInodeType {
    fn from(value: u8) -> Self {
        match value {
            b'0' | 0 => TarInodeType::File,
            b'1' => TarInodeType::HardLink,
            b'2' => TarInodeType::Symlink,
            b'3' => TarInodeType::CharacterDevice,
            b'4' => TarInodeType::BlockDevice,
            b'5' => TarInodeType::Directory,
            b'6' => TarInodeType::Fifo,
            _ => TarInodeType::Unknown,
        }
    }
}

pub(crate) struct TarInode {
    pub(crate) inode: u64,
    pub(crate) kind: TarInodeType,
    pub(crate) ids: Arc<AtomicU64>,
    pub(crate) data: Arc<Vec<u8>>,
    pub(crate) children: Arc<Mutex<BTreeMap<String, Arc<TarInode>>>>,
    pub(crate) link: Option<String>,
}

impl Inode for TarInode {
    fn open(&self, _flags: OpenFlags) -> VfsResult<Arc<dyn FileOps>> {
        if !matches!(self.kind, TarInodeType::File | TarInodeType::HardLink) {
            return Err(VfsError::NotAFile);
        }

        Ok(Arc::new(TarFileOps { data: self.data.clone() }))
    }

    fn close(&self) -> VfsResult<()> { Ok(()) }

    fn metadata(&self) -> VfsResult<Metadata> {
        let size = match self.kind {
            TarInodeType::Directory => self.children.lock().len() as u64,
            TarInodeType::Symlink => {
                self.link.as_ref().map_or(0, |link| link.len() as u64)
            },
            _ => self.data.len() as u64,
        };
        Ok(Metadata { inode: self.inode, size })
    }

    fn get_type(&self) -> VfsResult<FileType> {
        Ok(match self.kind {
            TarInodeType::File | TarInodeType::HardLink => FileType::File,
            TarInodeType::Directory => FileType::Directory,
            TarInodeType::Symlink => FileType::Symlink,
            _ => FileType::Unknown,
        })
    }

    fn lookup(&self, name: &str) -> VfsResult<Arc<dyn Inode>> {
        if self.kind != TarInodeType::Directory {
            return Err(VfsError::NotADirectory);
        }
        if name.is_empty() || name.contains('/') {
            return Err(VfsError::InvalidPath);
        }

        self.children
            .lock()
            .get(name)
            .cloned()
            .map(|inode| inode as Arc<dyn Inode>)
            .ok_or(VfsError::NotFound)
    }

    fn create(
        &self,
        _name: &str,
        _flags: OpenFlags,
    ) -> VfsResult<Arc<dyn Inode>> {
        if self.kind != TarInodeType::Directory {
            return Err(VfsError::NotADirectory);
        }

        Err(VfsError::ReadOnly)
    }

    fn readlink(&self) -> VfsResult<String> {
        if self.kind != TarInodeType::Symlink {
            return Err(VfsError::InvalidArgument);
        }
        self.link.clone().ok_or(VfsError::InvalidArgument)
    }
}

struct TarFileOps {
    data: Arc<Vec<u8>>,
}

impl FileOps for TarFileOps {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let start =
            usize::try_from(offset).map_err(|_| VfsError::InvalidArgument)?;
        if start >= self.data.len() {
            return Ok(0);
        }

        let count = buf.len().min(self.data.len() - start);
        buf[..count].copy_from_slice(&self.data[start..start + count]);
        Ok(count)
    }

    fn write(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::ReadOnly)
    }

    fn truncate(&self, _size: u64) -> VfsResult<()> { Err(VfsError::ReadOnly) }
}
