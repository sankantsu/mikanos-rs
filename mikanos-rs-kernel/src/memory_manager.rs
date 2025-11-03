use uefi::mem::memory_map::{MemoryMap, MemoryMapOwned, MemoryType};
const PAGE_SIZE: usize = 4 * 1024;
const MAX_PHYSICAL_MEM_SIZE: usize = 64 * 1024 * 1024 * 1024;
const MAX_NUM_PAGE_FRAME: usize = MAX_PHYSICAL_MEM_SIZE / PAGE_SIZE;
const BITMAP_SIZE: usize = MAX_NUM_PAGE_FRAME / 8;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameID(usize);

impl FrameID {
    fn offset(&self, offset: usize) -> Self {
        Self(self.0 + offset)
    }
}

pub struct BitmapMemoryManager {
    alloc_map: [u8; BITMAP_SIZE],
    range_begin: FrameID,
    range_end: FrameID,
}

impl BitmapMemoryManager {
    pub const fn new() -> Self {
        Self {
            alloc_map: [0; BITMAP_SIZE],
            range_begin: FrameID(0),
            range_end: FrameID(MAX_NUM_PAGE_FRAME),
        }
    }
    fn set_memory_range(&mut self, range_begin: FrameID, range_end: FrameID) {
        self.range_begin = range_begin;
        self.range_end = range_end;
    }
    pub fn allocate(&mut self, num_frames: usize) -> Option<FrameID> {
        let mut start_frame_id = self.range_begin;
        loop {
            let mut i = 0;
            while i < num_frames {
                if start_frame_id.offset(i) >= self.range_end {
                    return None;
                }
                if self.get_bit(start_frame_id.offset(i)) {
                    break;
                }
                i += 1;
            }
            if i == num_frames {
                self.mark_allocated(start_frame_id, num_frames);
                return Some(start_frame_id);
            }
            start_frame_id = start_frame_id.offset(i + 1)
        }
    }
    // TODO:
    // Current BitmapMemoryManager implementation requires the user to explicitly pass the allocation size to free().
    // It differs from standard malloc()/free() interface.
    // How can we handle this problem when we integrate this memory allocator to Rust global_allocator?
    pub fn free(&mut self, start_frame: FrameID, num_frames: usize) {
        assert!(
            self.range_begin <= start_frame && start_frame.offset(num_frames) <= self.range_end
        );
        for i in 0..num_frames {
            self.set_bit(start_frame.offset(i), false);
        }
    }
    fn mark_allocated(&mut self, start_frame: FrameID, num_frames: usize) {
        assert!(
            self.range_begin <= start_frame && start_frame.offset(num_frames) <= self.range_end
        );
        for i in 0..num_frames {
            self.set_bit(start_frame.offset(i), true);
        }
    }
    fn set_bit(&mut self, frame_id: FrameID, allocated: bool) {
        let byte_idx = frame_id.0 / 8;
        let bit_idx = frame_id.0 % 8;

        if allocated {
            let mask = 1 << bit_idx;
            self.alloc_map[byte_idx] |= mask;
        } else {
            let mask = !(1 << bit_idx);
            self.alloc_map[byte_idx] &= mask;
        }
    }
    fn get_bit(&self, frame_id: FrameID) -> bool {
        let byte_idx = frame_id.0 / 8;
        let bit_idx = frame_id.0 % 8;
        let mask = 1 << bit_idx;

        (self.alloc_map[byte_idx] & mask) != 0
    }
}

pub static MEMORY_MANAGER: spin::Mutex<BitmapMemoryManager> =
    spin::Mutex::new(BitmapMemoryManager::new());

pub fn init(memory_map: &'static MemoryMapOwned) {
    let mut available_end = 0;
    for (i, desc) in memory_map.entries().enumerate() {
        if available_end < desc.phys_start {
            let start_frame_id = FrameID((available_end / PAGE_SIZE as u64) as usize);
            let num_pages = ((desc.phys_start - available_end) / PAGE_SIZE as u64) as usize;
            MEMORY_MANAGER
                .lock()
                .mark_allocated(start_frame_id, num_pages);
        }
        if desc.ty == MemoryType::CONVENTIONAL
            || desc.ty == MemoryType::BOOT_SERVICES_CODE
            || desc.ty == MemoryType::BOOT_SERVICES_DATA
        {
            available_end = desc.phys_start + desc.page_count * uefi::boot::PAGE_SIZE as u64;
        } else {
            let start_frame_id = FrameID((desc.phys_start / PAGE_SIZE as u64) as usize);
            let num_pages = desc.page_count as usize * uefi::boot::PAGE_SIZE / PAGE_SIZE;
            MEMORY_MANAGER
                .lock()
                .mark_allocated(start_frame_id, num_pages);
            available_end = desc.phys_start + desc.page_count * uefi::boot::PAGE_SIZE as u64;
        }
    }
    MEMORY_MANAGER.lock().set_memory_range(
        FrameID(1),
        FrameID((available_end / PAGE_SIZE as u64) as usize),
    );
}
