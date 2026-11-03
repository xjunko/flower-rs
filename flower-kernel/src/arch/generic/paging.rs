bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct MapFlags: u32 {
        const READ      = 1 << 0;
        const WRITE     = 1 << 1;
        const EXEC      = 1 << 2;
        const USER      = 1 << 3;
        const NO_CACHE  = 1 << 4;
        const WRITE_THROUGH = 1 << 5;
    }
}
