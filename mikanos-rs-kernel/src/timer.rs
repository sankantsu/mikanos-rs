const MAX_COUNT: u32 = 0x100000;
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
        start_local_apic_timer();
    }
}

fn start_local_apic_timer() {
    unsafe {
        core::ptr::write_volatile(INITIAL_COUNT, MAX_COUNT);
    }
}

pub struct Timer {
    timeout: u64,
    value: i64,
}

impl Timer {
    pub fn new(timeout: u64, value: i64) -> Self {
        Self { timeout, value }
    }
}

// Comparison based on timeout priority
impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.timeout == other.timeout
    }
}

impl Eq for Timer {}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.timeout.cmp(&other.timeout).reverse())
    }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.timeout.cmp(&other.timeout).reverse()
    }
}

pub struct TimerManager {
    tick: core::sync::atomic::AtomicU64,
    timers: alloc::collections::BinaryHeap<Timer>,
    task_timeout: bool,
}

impl TimerManager {
    pub const fn new() -> Self {
        TimerManager {
            tick: core::sync::atomic::AtomicU64::new(0),
            timers: alloc::collections::BinaryHeap::new(),
            task_timeout: false,
        }
    }
    fn add_timer(&mut self, timer: Timer) {
        self.timers.push(timer)
    }
    pub fn tick(&mut self) {
        self.tick
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        while !self.timers.is_empty() {
            let t = self.timers.peek().unwrap();
            if t.timeout > self.tick.load(core::sync::atomic::Ordering::Relaxed) {
                // No need to handle timeout
                break;
            }
            if t.value == crate::task::TASK_TIMEOUT_MESSAGE {
                // task timeout event
                self.task_timeout = true;
                crate::task::add_task_timeout_timer(
                    self.tick.load(core::sync::atomic::Ordering::Relaxed),
                );
            } else {
                // other timeout events
                let event = crate::event::Event::Timeout(t.timeout, t.value);
                unsafe {
                    crate::event::get_event_queue_raw()
                        .lock()
                        .push(event)
                        .unwrap()
                };
            }
            self.timers.pop().unwrap();
        }
    }
    pub fn get_tick(&self) -> u64 {
        self.tick.load(core::sync::atomic::Ordering::Relaxed)
    }
    pub fn check_task_timeout(&self) -> bool {
        self.task_timeout
    }
    pub fn reset_task_timeout(&mut self) {
        self.task_timeout = false;
    }
}

pub static mut TIMER_MANAGER: TimerManager = TimerManager::new();

#[allow(static_mut_refs)]
pub fn get_current_tick() -> u64 {
    unsafe { TIMER_MANAGER.get_tick() }
}

#[allow(static_mut_refs)]
pub fn add_timer(timer: Timer) {
    unsafe { TIMER_MANAGER.add_timer(timer) }
}
