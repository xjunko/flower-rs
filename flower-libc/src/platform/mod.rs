use alloc::vec::Vec;

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut argv: Vec<&str> = Vec::new();
