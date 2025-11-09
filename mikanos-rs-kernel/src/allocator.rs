use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 128 * 1024 * 1024; // 128 MB
const NUM_HEAP_PAGES: usize = HEAP_SIZE / crate::memory_manager::PAGE_SIZE;

pub fn init_heap() {
    let heap_start = crate::memory_manager::MEMORY_MANAGER
        .lock()
        .allocate(NUM_HEAP_PAGES)
        .unwrap()
        .get_addr() as *mut u8;
    unsafe {
        ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
    }
    crate::serial_println!("Init heap done.");
}
