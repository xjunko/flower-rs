pub mod heap;
pub mod pmm;
pub mod vmm;

mod tests;

pub const PAGE_SIZE: usize = 4096;

pub fn self_test() {
    tests::start();
}
