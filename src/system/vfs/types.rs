use alloc::boxed::Box;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VFSFileType {
    File,
    Directory,
    Device,
}

#[derive(Debug, Clone)]
pub struct VFSMetadata {
    pub file_type: VFSFileType,
    pub size: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum VFSSeek {
    Start(usize),
    Current(usize),
    End(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VFSError {
    NotFound,
    AlreadyExists,
    InvalidSeek,
    Unknown,
}

pub type VFSResult<T> = Result<T, VFSError>;

pub trait VFSFile: Send + Sync {
    /// reads data into the given buffer and returns the number of bytes read
    fn read(&self, buf: &mut [u8]) -> VFSResult<usize>;

    /// writes data from the given buffer and returns the number of bytes written
    fn write(&self, buf: &mut [u8]) -> VFSResult<usize>;

    /// seeks to the given position and returns the new position
    fn seek(&mut self, pos: VFSSeek) -> VFSResult<usize>;

    /// gets the info for the file
    fn metadata(&self) -> VFSResult<VFSMetadata>;
}

pub trait VFSImplementation: Send + Sync {
    /// opens the file
    fn open(&self, path: &str, flags: u32) -> VFSResult<Box<dyn VFSFile>>;

    /// gets the info for the file
    fn metadata(&self, path: &str) -> VFSResult<VFSMetadata>;

    /// checks if the file exists
    fn exists(&self, path: &str) -> bool {
        self.metadata(path).is_ok()
    }
}
