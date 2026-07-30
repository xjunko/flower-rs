mod heap;

pub fn start() {
    log::debug!("starting memory tests...");

    // heap
    log::debug!("testing heap...");
    {
        heap::test_heap_basic();
        heap::test_heap_vec();
        heap::test_heap_fragmentation();
        heap::test_heap_large();
        heap::test_heap_oom();
        heap::test_heap_stress();
    }
}
