use alloc::boxed::Box;

use crate::system::vfs::types::{VFSError, VFSFile, VFSResult};

pub const MAX_FDS: usize = 256;

pub enum FdKind {
    File(Box<dyn VFSFile>),
    Stdin,
    Stdout,
    Stderr,
}

pub struct FdTable {
    fds: [Option<FdKind>; MAX_FDS],
}

impl FdTable {
    pub fn new() -> Self {
        let mut table = Self { fds: [const { None }; MAX_FDS] };
        table.fds[0] = Some(FdKind::Stdin);
        table.fds[1] = Some(FdKind::Stdout);
        table.fds[2] = Some(FdKind::Stderr);
        table
    }

    pub fn alloc(&mut self, kind: FdKind) -> VFSResult<usize> {
        for i in 0..MAX_FDS {
            if self.fds[i].is_none() {
                self.fds[i] = Some(kind);
                return Ok(i);
            }
        }
        Err(VFSError::NoSpace)
    }

    pub fn get(&self, fd: usize) -> VFSResult<&FdKind> {
        if fd >= MAX_FDS {
            return Err(VFSError::NotFound);
        }
        self.fds.get(fd).and_then(|opt| opt.as_ref()).ok_or(VFSError::NotFound)
    }

    pub fn get_mut(&mut self, fd: usize) -> VFSResult<&mut FdKind> {
        if fd >= MAX_FDS {
            return Err(VFSError::NotFound);
        }
        self.fds
            .get_mut(fd)
            .and_then(|opt| opt.as_mut())
            .ok_or(VFSError::NotFound)
    }

    pub fn close(&mut self, fd: usize) -> VFSResult<()> {
        if fd >= MAX_FDS {
            return Err(VFSError::NotFound);
        }
        if fd < 3 {
            return Err(VFSError::PermissionDenied);
        }
        if self.fds[fd].is_none() {
            return Err(VFSError::NotFound);
        }
        self.fds[fd] = None;
        Ok(())
    }
}
