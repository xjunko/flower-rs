#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct FileStat {
    pub st_mode: u16,
    pub st_dev: u64,
    pub st_size: u64,
}
