pub const S_IFMT: usize = 0o170000; // bitmask for file type
pub const S_IFREG: usize = 0o100000; // regular file
pub const S_IFDIR: usize = 0o040000; // directory
pub const S_IFCHR: usize = 0o020000; // char device
pub const S_IFBLK: usize = 0o060000; // block device
pub const S_IFIFO: usize = 0o010000; // fifo/pipe
pub const S_IFLNK: usize = 0o120000; // symlink
