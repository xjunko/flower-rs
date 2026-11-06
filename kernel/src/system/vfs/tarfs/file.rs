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
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::arch::addr::PhysAddr;
use crate::system::posix::errno::{KError, KResult};
use crate::system::vfs::{File, FileType, Stat, Whence};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TarFileType {
    File = 0,
    HardLink = 1,
    Symlink = 2,
    CharacterDevice = 3,
    BlockDevice = 4,
    Directory = 5,
    Fifo = 6,
    Unknown = 7,
}

impl From<u8> for TarFileType {
    fn from(value: u8) -> Self {
        match value {
            b'0' | 0 => TarFileType::File,
            b'1' => TarFileType::HardLink,
            b'2' => TarFileType::Symlink,
            b'3' => TarFileType::CharacterDevice,
            b'4' => TarFileType::BlockDevice,
            b'5' => TarFileType::Directory,
            b'6' => TarFileType::Fifo,
            _ => TarFileType::Unknown,
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
    pub file_type: TarFileType,
    pub owner_name: String,
    pub group_name: String,
    pub device_major: usize,
    pub device_minor: usize,
    pub prefix: String,
    /// symlink target path, if this is a symlink. empty otherwise.
    pub linkname: String,
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
            linkname: self.linkname.clone(),
        }
    }
}

impl File for TarFile {
    fn name(&self) -> KResult<String> { Ok(self.name.clone()) }

    fn read(&self, buf: &mut [u8]) -> KResult<usize> {
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

    fn write(&self, _buf: &mut [u8]) -> KResult<usize> {
        // tarfs is a read-only view over the initramfs archive.
        Err(KError::EPERM)
    }

    fn seek(&mut self, offset: i64, pos: Whence) -> KResult<usize> {
        let current = self._position.load(Ordering::Acquire) as i128;
        let base = match pos {
            Whence::Start => 0,
            Whence::Current => current,
            Whence::End => self.size as i128,
        };
        let new_pos = base
            .checked_add(offset as i128)
            .and_then(|position| usize::try_from(position).ok())
            .ok_or(KError::ESPIPE)?;

        self._position.store(new_pos, Ordering::Release);
        Ok(new_pos)
    }

    fn mmap(
        &self,
        _len: usize,
        _prot: core::ffi::c_int,
        _flags: core::ffi::c_int,
        _offset: u64,
    ) -> KResult<PhysAddr> {
        unimplemented!()
    }

    fn stat(&self) -> KResult<Stat> {
        let typ = match self.file_type {
            TarFileType::File | TarFileType::HardLink => FileType::File,
            TarFileType::Directory => FileType::Directory,
            TarFileType::CharacterDevice | TarFileType::BlockDevice => {
                FileType::Device
            },
            TarFileType::Symlink => FileType::Symlink,
            TarFileType::Fifo => FileType::Pipe,
            _ => FileType::Unknown,
        };

        Ok(Stat {
            name: self.name.clone(),
            typ,
            size: self.size,
            last_modified: self.last_modified,
        })
    }
}
