#[derive(Debug)]
pub enum AuxType {
    Null = 0,
    Ignore = 1,
    ExecFileDescriptor = 2,
    Phdr = 3,
    Phent = 4,
    Phnum = 5,
    PageSize = 6,
    Base = 7,
    Flags = 8,
    Entry = 9,
    NotElf = 10,
    Uid = 11,
    Euid = 12,
    Gid = 13,
    Egid = 14,
    ClockTick = 17,
}
