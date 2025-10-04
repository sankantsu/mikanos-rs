use crate::queue::Queue;

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
pub static EVENT_QUEUE: spin::Mutex<Queue<Event, QUEUE_SIZE>> =
    spin::Mutex::new(Queue::<Event, QUEUE_SIZE>::new(Event::Invalid));
