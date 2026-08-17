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

use crate::system::vfs::error::{VfsError, VfsResult};
use crate::system::vfs::inode::{FileType, Inode, Metadata};
use crate::system::vfs::perm::Permissions;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TarFsFileType {
    File = 0,
    HardLink = 1,
    Symlink = 2,
    CharacterDevice = 3,
    BlockDevice = 4,
    Directory = 5,
    Fifo = 6,
    Unknown = 7,
}

impl From<u8> for TarFsFileType {
    fn from(value: u8) -> Self {
        match value {
            b'0' | 0 => TarFsFileType::File,
            b'1' => TarFsFileType::HardLink,
            b'2' => TarFsFileType::Symlink,
            b'3' => TarFsFileType::CharacterDevice,
            b'4' => TarFsFileType::BlockDevice,
            b'5' => TarFsFileType::Directory,
            b'6' => TarFsFileType::Fifo,
            _ => TarFsFileType::Unknown,
        }
    }
}

pub(crate) fn to_file_type(typ: TarFsFileType) -> FileType {
    match typ {
        TarFsFileType::File | TarFsFileType::HardLink => FileType::Regular,
        TarFsFileType::Directory => FileType::Directory,
        TarFsFileType::CharacterDevice => FileType::CharDevice,
        TarFsFileType::BlockDevice => FileType::BlockDevice,
        TarFsFileType::Symlink => FileType::Symlink,
        TarFsFileType::Fifo => FileType::Fifo,
        TarFsFileType::Unknown => FileType::Unknown,
    }
}

pub struct TarFile {
    pub _data_position: usize,
    pub _data: Arc<Vec<u8>>,

    pub inode: u64,
    pub name: String,
    pub path: String,
    pub mode: usize,
    pub owner_id: usize,
    pub group_id: usize,
    pub size: usize,
    pub last_modified: usize,
    pub checksum: usize,
    pub file_type: TarFsFileType,
    pub owner_name: String,
    pub group_name: String,
    pub device_major: usize,
    pub device_minor: usize,
    pub prefix: String,
    pub linkname: String,
}

impl Inode for TarFile {
    fn metadata(&self) -> VfsResult<Metadata> {
        Ok(Metadata {
            inode: self.inode,
            size: self.size,
            file_type: self::to_file_type(self.file_type),
            permissions: Permissions::from_unix(self.mode),
            owner: self.owner_id as u32,
            group: self.group_id as u32,
            links: 1,
            last_modified: self.last_modified as u64,
        })
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> VfsResult<usize> {
        if offset >= self.size {
            return Ok(0);
        }

        let bytes_to_read = core::cmp::min(buf.len(), self.size - offset);
        let source =
            unsafe { self._data.as_ptr().add(self._data_position + offset) };
        unsafe {
            core::ptr::copy_nonoverlapping(
                source,
                buf.as_mut_ptr(),
                bytes_to_read,
            );
        }
        Ok(bytes_to_read)
    }

    fn write_at(&self, _offset: usize, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::Unsupported)
    }

    fn mmap(
        &self,
        _len: usize,
        _prot: core::ffi::c_int,
        _flags: core::ffi::c_int,
        _offset: u64,
    ) -> VfsResult<*mut u8> {
        Err(VfsError::Unsupported)
    }

    fn readlink(&self) -> VfsResult<String> {
        if self.file_type == TarFsFileType::Symlink {
            Ok(self.linkname.clone())
        } else {
            Err(VfsError::Unsupported)
        }
    }
}
