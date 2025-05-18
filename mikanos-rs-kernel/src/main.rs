#![no_std]
#![no_main]

mod serial;

use core::panic::PanicInfo;
use mikanos_rs_frame_buffer::{FrameBuffer, PixelColor};
use uefi::mem::memory_map::{MemoryMap, MemoryMapOwned};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[repr(C, align(16))]
struct KernelStack([u8; 1024 * 1024]);

impl KernelStack {
    #[inline(always)]
    const fn new() -> Self {
        Self([0; 1024 * 1024])
    }

    #[inline(always)]
    pub fn end_addr(&self) -> u64 {
        self.0.as_ptr() as u64 + 1024 * 1024
    }
}

const _KERNEL_MAIN_STACK: KernelStack = KernelStack::new();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn kernel_main(frame_buffer: &FrameBuffer, memory_map: &MemoryMapOwned) {
    let stack_top = _KERNEL_MAIN_STACK.end_addr();
    core::arch::asm!(
        "mov rsp, {0}",
        "call kernel_main_new_stack",
        in(reg) stack_top,
        in("rdi") frame_buffer,
        in("rsi") memory_map,
        clobber_abi("C"),
    );
}

const K_FONT_A: [u8; 16] = [
    0b00000000, //
    0b00011000, //    **
    0b00011000, //    **
    0b00011000, //    **
    0b00011000, //    **
    0b00100100, //   *  *
    0b00100100, //   *  *
    0b00100100, //   *  *
    0b00100100, //   *  *
    0b01111110, //  ******
    0b01000010, //  *    *
    0b01000010, //  *    *
    0b01000010, //  *    *
    0b11100111, // ***  ***
    0b00000000, //
    0b00000000, //
];

fn write_ascii(frame_buffer: &FrameBuffer, x: usize, y: usize, c: char, color: &PixelColor) {
    if c != 'A' {
        return;
    }
    for dy in 0..16 {
        for dx in 0..8 {
            if ((K_FONT_A[dy] << dx) & 0x80) != 0 {
                frame_buffer.write_pixel(x + dx, y + dy, color);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main_new_stack(frame_buffer: &FrameBuffer, memory_map: &MemoryMapOwned) {
    frame_buffer.fill(&PixelColor::new(255, 255, 255));
    let rect_width = 200;
    let rect_height = 100;
    let offset = (100, 100);
    for x in 0..rect_width {
        for y in 0..rect_height {
            frame_buffer.write_pixel(x + offset.0, y + offset.1, &PixelColor::new(0, 255, 0));
        }
    }

    write_ascii(frame_buffer, 50, 50, 'A', &PixelColor::new(0, 0, 0));
    write_ascii(frame_buffer, 58, 50, 'A', &PixelColor::new(0, 0, 0));

    let header = "Index, Type, Type(name), PhysicalStart, NumberOfPages, Attribute";
    serial_println!("{}", header);
    for (i, desc) in memory_map.entries().enumerate() {
        serial_println!(
            "{}, {:#x}, {:?}, {:#08x}, {}, {:#x}",
            i,
            desc.ty.0,
            desc.ty,
            desc.phys_start,
            desc.page_count,
            desc.att.bits() & 0xfffff,
        );
    }
    loop {}
}
