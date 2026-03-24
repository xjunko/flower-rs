use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::system::vfs::{
    VFSFile, VFSFileType, VFSMetadata, VFSPermissions, VFSResult, VFSSeek,
};

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

pub struct TarFile {
    pub _data_position: usize,
    pub _position: AtomicUsize,
    pub _data: Arc<Vec<u8>>,

    pub name: String,
    pub path: String,
    pub mode: usize,
    pub owner_id: usize,
    pub group_id: usize,
    pub size: usize,
    pub last_modified: usize,
    pub checksum: usize,
    pub file_type: TarFSFileType,
    pub owner_name: String,
    pub group_name: String,
    pub device_major: usize,
    pub device_minor: usize,
    pub prefix: String,
}

impl Clone for TarFile {
    fn clone(&self) -> Self {
        Self {
            _data_position: self._data_position,
            _position: AtomicUsize::new(self._position.load(Ordering::Relaxed)),
            _data: Arc::clone(&self._data),
            name: self.name.clone(),
            path: self.path.clone(),
            mode: self.mode,
            owner_id: self.owner_id,
            group_id: self.group_id,
            size: self.size,
            last_modified: self.last_modified,
            checksum: self.checksum,
            file_type: self.file_type,
            owner_name: self.owner_name.clone(),
            group_name: self.group_name.clone(),
            device_major: self.device_major,
            device_minor: self.device_minor,
            prefix: self.prefix.clone(),
        }
    }
}

impl VFSFile for TarFile {
    fn read(&self, buf: &mut [u8]) -> VFSResult<usize> {
        let (position, bytes_to_read) = loop {
            let position = self._position.load(Ordering::Acquire);

            if position >= self.size {
                return Ok(0);
            }

            let bytes_to_read = core::cmp::min(buf.len(), self.size - position);
            let new_position = position + bytes_to_read;

            if self
                ._position
                .compare_exchange(
                    position,
                    new_position,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                break (position, bytes_to_read);
            }
        };

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
            VFSSeek::Current(n) => {
                self._position.load(Ordering::Acquire).saturating_add(n)
            },
            VFSSeek::End(n) => self.size.saturating_add(n),
        };

        new_pos = core::cmp::min(new_pos, self.size);

        self._position.store(new_pos, Ordering::Release);
        Ok(new_pos)
    }

    fn mmap(
        &self,
        _len: usize,
        _prot: core::ffi::c_int,
        _flags: core::ffi::c_int,
    ) -> VFSResult<*mut u8> {
        unimplemented!()
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
            typ,
            size: self.size,
            last_modified: self.last_modified,
            owner_id: self.owner_id,
            group_id: self.group_id,
            permissions: VFSPermissions::from_unix(self.mode),
        })
    }
}
