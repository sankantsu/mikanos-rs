use crate::descriptor::{DescriptorTablePointer, SystemDescriptorType};
use bitfield::bitfield;

#[repr(u8)]
pub enum InterruptVector {
    DivisionError = 0x00,
    DoubleFault = 0x08,
    XHCI = 0x40,
    Timer = 0x41,
}

bitfield! {
  #[repr(transparent)]
  #[derive(Clone, Copy)]
  pub struct IDTAttribute(u16);
  pub interrupt_stack_table, _ : 2, 0;
  // padding : 7, 3;
  pub r#type, set_type : 11, 8;
  // padding : 12;
  pub descriptor_privilege_level, set_descriptor_privilege_level : 14, 13;
  pub present, set_present : 15;
}

impl IDTAttribute {
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }
    #[inline]
    pub fn new(descriptor_type: SystemDescriptorType, descriptor_privilege_level: u16) -> Self {
        let mut attr = Self(0);
        // Assume IST (interrupt stack table) value is 0.
        attr.set_type(descriptor_type as u16);
        attr.set_descriptor_privilege_level(descriptor_privilege_level);
        attr.set_present(true);
        attr
    }
}

pub type HandlerFunc = extern "x86-interrupt" fn();

#[derive(Clone, Copy)]
#[repr(C)]
struct InterruptDescriptor {
    offset_low: u16,
    segment_selector: u16,
    attr: IDTAttribute,
    offset_middle: u16,
    offset_high: u32,
    _reserved: u32,
}

impl InterruptDescriptor {
    #[inline]
    pub const fn empty() -> Self {
        Self {
            offset_low: 0,
            segment_selector: 0,
            attr: IDTAttribute::empty(),
            offset_middle: 0,
            offset_high: 0,
            _reserved: 0,
        }
    }
    #[inline]
    pub fn new(attr: IDTAttribute, handler: HandlerFunc) -> Self {
        let mut cs: u16;
        let offset = handler as u64;
        // Get the current value of the code-segment register.
        unsafe { core::arch::asm!("mov ax, cs", out("ax") cs) }
        Self {
            offset_low: (offset & 0xffff) as u16,
            segment_selector: cs,
            attr,
            offset_middle: ((offset & 0xffff0000) >> 16) as u16,
            offset_high: (offset >> 32) as u32,
            _reserved: 0,
        }
    }
}

#[repr(align(16))]
struct InterruptDescriptorTable {
    data: [InterruptDescriptor; 256],
}

impl InterruptDescriptorTable {
    const TABLE_SIZE: usize = 256;
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: [InterruptDescriptor::empty(); Self::TABLE_SIZE],
        }
    }
    pub fn set_entry(&mut self, idx: usize, entry: InterruptDescriptor) {
        self.data[idx] = entry;
    }
    fn get_limit(&self) -> u16 {
        (Self::TABLE_SIZE * core::mem::size_of::<InterruptDescriptor>() - 1) as u16
    }
    #[inline]
    fn to_descriptor_pointer(&self) -> DescriptorTablePointer {
        let limit = self.get_limit();
        DescriptorTablePointer::new(limit, self as *const Self as u64)
    }
    pub unsafe fn load(&self) {
        let descriptor_pointer = self.to_descriptor_pointer();
        unsafe {
            core::arch::asm!("lidt [{}]", in(reg) &descriptor_pointer);
        }
    }
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init_idt() {
    unsafe {
        IDT.set_entry(
            InterruptVector::DivisionError as usize,
            InterruptDescriptor::new(
                IDTAttribute::new(SystemDescriptorType::InterruptGate, 0),
                handle_division_error,
            ),
        );
        IDT.set_entry(
            InterruptVector::DoubleFault as usize,
            InterruptDescriptor::new(
                IDTAttribute::new(SystemDescriptorType::InterruptGate, 0),
                handle_double_fault,
            ),
        );
        IDT.set_entry(
            InterruptVector::XHCI as usize,
            InterruptDescriptor::new(
                IDTAttribute::new(SystemDescriptorType::InterruptGate, 0),
                handle_xhci_event,
            ),
        );
        IDT.set_entry(
            InterruptVector::Timer as usize,
            InterruptDescriptor::new(
                IDTAttribute::new(SystemDescriptorType::InterruptGate, 0),
                handle_timer_event,
            ),
        );
        IDT.load();
    }
    // Check IDT configuration
    let current_idtp = DescriptorTablePointer::from_current_idt();
    let idt_base = unsafe { &IDT as *const InterruptDescriptorTable as u64 };
    let limit = unsafe { IDT.get_limit() };
    assert_eq!(current_idtp.get_base(), idt_base);
    assert_eq!(current_idtp.get_limit(), limit);
    crate::serial_println!("IDT initialization done.");
}

#[inline]
pub fn enable_maskable_interrupts() {
    unsafe {
        core::arch::asm!("sti");
    }
}

#[inline]
pub fn disable_maskable_interrupts() {
    unsafe {
        core::arch::asm!("cli");
    }
}

pub struct InterruptGuard<T> {
    ptr: *mut T,
}

impl<T> InterruptGuard<T> {
    pub fn new(val: &mut T) -> Self {
        disable_maskable_interrupts();
        Self { ptr: val as *mut T }
    }
}

impl<T> Drop for InterruptGuard<T> {
    fn drop(&mut self) {
        enable_maskable_interrupts();
    }
}

impl<T> core::ops::Deref for InterruptGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> core::ops::DerefMut for InterruptGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

fn notify_end_of_interrupt() {
    let eoi_reg = 0xfee000b0 as *mut u32;
    unsafe { *eoi_reg = 0 };
}

pub extern "x86-interrupt" fn handle_division_error() {
    crate::serial_println!("Zero division detected!");
    panic!();
}

pub extern "x86-interrupt" fn handle_double_fault() {
    crate::serial_println!("Double fault detected!");
    panic!();
}

pub extern "x86-interrupt" fn handle_xhci_event() {
    use crate::event::Event;
    if let Err(_event) = unsafe { crate::event::get_event_queue_raw().lock().push(Event::XHCI) } {
        crate::serial_println!("Warning: EVENT_QUEUE full, XHCI event dropped");
    }
    notify_end_of_interrupt();
}

extern "x86-interrupt" fn handle_timer_event() {
    unsafe {
        crate::timer::TIMER_MANAGER.tick();
    };
    notify_end_of_interrupt();
}
