#![no_std]
#![no_main]

use core::panic::PanicInfo;
use mikanos_rs_frame_buffer::{FrameBuffer, PixelColor};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(frame_buffer: FrameBuffer) {
    frame_buffer.fill(&PixelColor::new(255, 255, 255));
    let rect_width = 200;
    let rect_height = 100;
    let offset = (100, 100);
    for x in 0..rect_width {
        for y in 0..rect_height {
            frame_buffer.write_pixel(x + offset.0, y + offset.1, &PixelColor::new(0, 255, 0));
        }
    }
    loop {}
}
