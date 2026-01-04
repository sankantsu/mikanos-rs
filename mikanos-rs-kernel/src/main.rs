#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, naked_functions)]
extern crate alloc;

mod allocator;
mod console;
mod descriptor;
mod event;
#[allow(static_mut_refs)]
mod interrupt;
mod memory_manager;
mod mouse;
mod paging;
mod pci;
mod queue;
mod segment;
mod serial;
mod task;
mod timer;
mod xhci;

use console::{Console, ShadowBuffer, copy_buffer};
use core::panic::PanicInfo;
use interrupt::enable_maskable_interrupts;
use mikanos_rs_frame_buffer::{FrameBuffer, FrameBufferWriter, PixelColor};
use mouse::{MouseEvent, init_mouse};
use uefi::mem::memory_map::MemoryMapOwned;
use xhci::{get_xhc, init_xhc};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("Panic!");
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
pub unsafe extern "C" fn kernel_main(
    frame_buffer: &'static FrameBuffer,
    memory_map: &'static MemoryMapOwned,
) {
    let stack_top = _KERNEL_MAIN_STACK.end_addr();
    unsafe {
        core::arch::asm!(
            "mov rsp, {0}",
            "call kernel_main_new_stack",
            in(reg) stack_top,
            in("rdi") frame_buffer,
            in("rsi") memory_map,
            clobber_abi("C"),
        );
    }
}

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
pub extern "C" fn kernel_main_new_stack(
    frame_buffer: &'static FrameBuffer,
    memory_map: &'static MemoryMapOwned,
) {
    unsafe {
        segment::init_gdt();
        paging::setup_identity_page_table();
        interrupt::init_idt();
        memory_manager::init(memory_map);
        allocator::init_heap();
        timer::init_local_apic_timer();
    }

    let mut console = Console::new(
        &frame_buffer,
        PixelColor::new(0, 0, 0),
        PixelColor::new(255, 255, 255),
    );

    let screen_width = frame_buffer.get_horizontal_resolution();
    let screen_height = frame_buffer.get_vertical_resolution();
    init_mouse((200, 300), (screen_width, screen_height));
    for _ in 0..100 {
        let dummy_event = MouseEvent::new(0, -10, 0);
        mouse::get_mouse().lock().move_mouse(&dummy_event);
    }

    // Scan PCI bus and find xHCI controller
    let mut pci_bus_scanner = pci::PCIBusScanner::new();
    pci_bus_scanner.scan_all();
    serial_println!("PCI Bus enumeration done.");
    let xhci_controller_addr = pci_bus_scanner.get_xhci_controller_address().unwrap();
    serial_println!("Found a xHCI controller.");

    // Read local APIC ID (see Intel SDM Vol 3, 12.4.6)
    let local_apic_id = unsafe { *(0xfee00020 as *const u32) >> 24 };
    crate::serial_println!("local_apic_id: {:x}", local_apic_id);
    // MSI message address and data (see Intel SDM Vol 3, 12.11)
    let msg_addr = 0xfee00000 | (local_apic_id << 12);
    let msg_data = 0xc000 | (interrupt::InterruptVector::XHCI as u32);
    crate::serial_println!("msg_addr: {:x}", msg_addr);
    crate::serial_println!("msg_data: {:x}", msg_data);
    xhci_controller_addr
        .configure_msi(msg_addr, msg_data)
        .unwrap();

    // Initialize USB driver
    let mmio_base = xhci_controller_addr.read_bar_64(0).unwrap();
    crate::serial_println!("mmio_base: {:x}", mmio_base);

    init_xhc(mmio_base);
    serial_println!("xHCI initialization done.");
    get_xhc().lock().run();
    serial_println!("Started running xHCI.");

    xhci::initialize_mouse();
    xhci::initialize_keyboard();

    for i in 1..=16 {
        get_xhc().lock().configure_port(i);
    }

    // Timer usage example
    timer::add_timer(timer::Timer::new(200, 2));
    task::initialize_task_switch();
    task::add_task(task::Task::new(task::TaskDescriptor::Func(task::task_b)));
    task::add_task(task::Task::new(task::TaskDescriptor::Func(task::task_c)));

    let mut shadow_buffer = ShadowBuffer::new(
        frame_buffer.get_pixels_per_scan_line(),
        frame_buffer.get_horizontal_resolution(),
        frame_buffer.get_vertical_resolution(),
        frame_buffer.get_pixel_format(),
    );

    // Start responding hardware and timer interrupts.
    enable_maskable_interrupts();

    serial_println!("Checking for a xhc event...");

    console.put_string("Started!\n");

    // main event loop
    let mut cnt = 0;
    loop {
        cnt += 1;
        if cnt % 10000 == 0 {
            let msg = alloc::format!("(Task A) count={}\n", cnt);
            serial_print!("{}", msg);
        }

        // Draw screen
        copy_buffer(console.get_buffer(), &shadow_buffer);
        mouse::get_mouse().lock().draw_mouse(&mut shadow_buffer);
        copy_buffer(&shadow_buffer, frame_buffer);

        if event::get_event_queue().lock().is_empty() {
            continue;
        }
        let event = event::get_event_queue().lock().pop().unwrap();

        match event {
            event::Event::XHCI => {
                while get_xhc().lock().has_event() {
                    get_xhc().lock().process_event();
                }
            }
            event::Event::Timeout(timeout, value) => {
                let current_tick = timer::get_current_tick();
                let s = alloc::format!(
                    "Timeout: timeout={}, value={} (current_tick={})\n",
                    timeout,
                    value,
                    current_tick
                );
                console.put_string(&s);
                if value > 0 {
                    let next_timeout = timeout + 100;
                    let next_value = value + 1;
                    timer::add_timer(timer::Timer::new(next_timeout, next_value));
                }
            }
            event::Event::Invalid => {
                serial_println!("invalid event!!");
                panic!()
            }
        }
    }
}
