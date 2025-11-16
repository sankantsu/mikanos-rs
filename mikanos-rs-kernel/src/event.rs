use crate::queue::Queue;

use crate::interrupt::InterruptGuard;

#[derive(Clone, Copy)]
pub enum Event {
    Invalid = 0,
    XHCI,
}

impl Default for Event {
    fn default() -> Self {
        Self::Invalid
    }
}

const QUEUE_SIZE: usize = 32;
static mut EVENT_QUEUE: spin::Mutex<Queue<Event, QUEUE_SIZE>> =
    spin::Mutex::new(Queue::<Event, QUEUE_SIZE>::new(Event::Invalid));

#[allow(static_mut_refs)]
pub fn get_event_queue() -> InterruptGuard<spin::Mutex<Queue<Event, QUEUE_SIZE>>> {
    unsafe { InterruptGuard::new(&mut EVENT_QUEUE) }
}

#[allow(static_mut_refs)]
pub unsafe fn get_event_queue_raw() -> &'static spin::Mutex<Queue<Event, QUEUE_SIZE>> {
    unsafe { &EVENT_QUEUE }
}
