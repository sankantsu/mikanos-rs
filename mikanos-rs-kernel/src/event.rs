use crate::queue::Queue;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Invalid,
    XHCI,
    Timeout(u64, i64), // timeout, value
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
pub unsafe fn get_event_queue_raw() -> &'static spin::Mutex<Queue<Event, QUEUE_SIZE>> {
    unsafe { &EVENT_QUEUE }
}
