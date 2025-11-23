const MAX_COUNT: u32 = 0xffffffff;
const LVT_TIMER: *mut u32 = 0xfee00320u64 as *mut u32;
const INITIAL_COUNT: *mut u32 = 0xfee00380u64 as *mut u32;
const CURRENT_COUNT: *mut u32 = 0xfee00390u64 as *mut u32;
const DIVIDE_CONFIG: *mut u32 = 0xfee003e0u64 as *mut u32;

pub unsafe fn init_local_apic_timer() {
    unsafe {
        core::ptr::write_volatile(DIVIDE_CONFIG, 0b1011); // divide 1:1
        core::ptr::write_volatile(
            LVT_TIMER,
            (0b010 << 16) | crate::interrupt::InterruptVector::Timer as u32, // not-masked, periodic
        );
    }
}

pub fn start_local_apic_timer() {
    unsafe {
        core::ptr::write_volatile(INITIAL_COUNT, MAX_COUNT);
    }
}

pub fn get_local_apic_timer_elapsed() -> u32 {
    unsafe {
        let current_count = core::ptr::read_volatile(CURRENT_COUNT);
        MAX_COUNT - current_count
    }
}

pub fn stop_local_apic_timer() {
    unsafe {
        core::ptr::write_volatile(INITIAL_COUNT, 0);
    }
}
