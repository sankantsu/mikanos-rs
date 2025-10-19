#[repr(u16)]
pub enum SegmentDescriptorType {
    DataReadWrite = 2,
    CodeExecuteRead = 10,
}

#[repr(u16)]
pub enum SystemDescriptorType {
    InterruptGate = 14,
}

#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    limit: u16,
    base: u64,
}

impl DescriptorTablePointer {
    pub fn new(limit: u16, base: u64) -> Self {
        Self { limit, base }
    }
    pub fn get_base(&self) -> u64 {
        self.base
    }
    pub fn get_limit(&self) -> u16 {
        self.limit
    }
    pub fn from_current_idt() -> Self {
        let mut descriptor_pointer = Self::new(0, 0);
        unsafe {
            core::arch::asm!("sidt [{}]", in(reg) &mut descriptor_pointer);
        }
        descriptor_pointer
    }
}
