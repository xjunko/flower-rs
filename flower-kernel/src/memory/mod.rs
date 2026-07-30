pub mod fault;
pub mod heap;
pub mod pmm;
pub mod tests;
pub mod vmm;

pub fn self_test() { tests::start(); }
