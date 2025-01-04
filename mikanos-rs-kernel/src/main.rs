#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::slice;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(frame_buffer_base: *mut u8, frame_buffer_size: usize) {
    let mut cnt = 0;
    loop {
        let buf = unsafe { slice::from_raw_parts_mut(frame_buffer_base, frame_buffer_size) };
        if cnt % 2 == 0 {
            buf.fill(0);
        } else {
            buf.fill(255);
        }
        cnt += 1;
    }
}
