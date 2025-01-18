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

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(frame_buffer: FrameBuffer, memory_map: MemoryMapOwned) {
    frame_buffer.fill(&PixelColor::new(255, 255, 255));
    let rect_width = 200;
    let rect_height = 100;
    let offset = (100, 100);
    for x in 0..rect_width {
        for y in 0..rect_height {
            frame_buffer.write_pixel(x + offset.0, y + offset.1, &PixelColor::new(0, 255, 0));
        }
    }
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
