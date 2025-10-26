use crate::descriptor::{DescriptorTablePointer, SegmentDescriptorType};
use bitfield::bitfield;

#[inline]
unsafe fn set_cs_register(sel: usize) {
    // Ref: https://docs.rs/x86_64/0.15.2/x86_64/registers/segmentation/struct.CS.html#method.set_reg
    // SPDX-License-Identifier: Apache-2
    // SPDX-FileCopyrightText: rust-osdev community
    unsafe {
        core::arch::asm!(
            "push {sel}",
            "lea {tmp}, [55f + rip]",
            "push {tmp}",
            "retfq",
            "55:",
            sel = in(reg) sel as u64,
            tmp = lateout(reg) _,
            options(preserves_flags),
        );
    }
}

bitfield! {
  #[repr(transparent)]
  #[derive(Clone, Copy)]
  pub struct SegmentDescriptor(u64);
  limit_low, _ : 15, 0;
  base_low, _ : 31, 16;
  base_middle, _ : 39, 32;
  r#type, set_type : 43, 40;
  system_segment, set_system_segment : 44;
  descriptor_privilege_level, set_descriptor_privilege_level : 46, 45;
  present, set_present : 47;
  limit_high, _ : 51, 48;
  available, set_available : 52;
  long_mode, set_long_mode : 53;
  default_operation_size, set_default_operation_size : 54, 54;
  granularity, _ : 55;
  base_high, _ : 63, 56;
}

impl SegmentDescriptor {
    #[inline]
    const fn empty() -> Self {
        Self(0)
    }
    fn make_code_segment() -> Self {
        let mut segment = Self(0);
        segment.set_type(SegmentDescriptorType::CodeExecuteRead as u64);
        segment.set_system_segment(true);
        segment.set_descriptor_privilege_level(0);
        segment.set_present(true);
        segment.set_available(false);
        segment.set_long_mode(true);
        segment.set_default_operation_size(0);
        segment
    }
    fn make_data_segment() -> Self {
        let mut segment = Self(0);
        segment.set_type(SegmentDescriptorType::DataReadWrite as u64);
        segment.set_system_segment(true);
        segment.set_descriptor_privilege_level(0);
        segment.set_present(true);
        segment.set_available(false);
        segment
    }
}

struct GlobalDescriptorTable {
    data: [SegmentDescriptor; Self::TABLE_SIZE],
}

impl GlobalDescriptorTable {
    const TABLE_SIZE: usize = 3;
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: [SegmentDescriptor::empty(); Self::TABLE_SIZE],
        }
    }
    fn set_entry(&mut self, idx: usize, entry: SegmentDescriptor) {
        self.data[idx] = entry;
    }
    fn get_limit(&self) -> u16 {
        (Self::TABLE_SIZE * core::mem::size_of::<SegmentDescriptor>() - 1) as u16
    }
    #[inline]
    fn to_descriptor_pointer(&self) -> DescriptorTablePointer {
        let limit = self.get_limit();
        DescriptorTablePointer::new(limit, self as *const Self as u64)
    }
    unsafe fn load(&self) {
        let descriptor_pointer = self.to_descriptor_pointer();
        core::arch::asm!("lgdt [{}]", in(reg) &descriptor_pointer);
    }
}

static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

#[allow(static_mut_refs)]
pub fn init_gdt() {
    unsafe {
        GDT.set_entry(1, SegmentDescriptor::make_code_segment()); // code segment
        GDT.set_entry(2, SegmentDescriptor::make_data_segment()); // data segment
        crate::serial_println!("Setup segments done.");
        GDT.load();
        crate::serial_println!("Load GDT done.");
        core::arch::asm!(
            // Set ax = 0
            "xor ax, ax",
            // Set data/stack segment registers to null descriptor
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
        );
        // Set code segment register
        set_cs_register(0x8);
        crate::serial_println!("Set segment registers done.");
    }
}
