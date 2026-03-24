pub mod heap;
pub mod pmm;
pub mod vmm;

mod tests;

pub fn self_test() { tests::start(); }
