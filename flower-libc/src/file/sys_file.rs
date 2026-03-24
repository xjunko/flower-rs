use alloc::string::String;

use crate::std;

pub struct File {
    fd: u64,
}

impl File {
    pub fn open(path: String) -> Result<Self, ()> {
        let fd = std::open(path.as_bytes(), 0, 0);
        if fd < 0 { Err(()) } else { Ok(Self { fd: fd as u64 }) }
    }

    pub fn close(&self) -> Result<(), ()> {
        if std::close(self.fd) < 0 { Err(()) } else { Ok(()) }
    }
}

impl File {
    pub fn read(&self, buf: &mut [u8]) -> Result<usize, ()> {
        let result = std::read(self.fd, buf);
        if result < 0 { Err(()) } else { Ok(result as usize) }
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, ()> {
        let result = std::write(self.fd, buf);
        if result < 0 { Err(()) } else { Ok(result as usize) }
    }
}
