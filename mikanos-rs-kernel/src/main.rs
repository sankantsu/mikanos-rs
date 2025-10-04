#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
mod event;
#[allow(static_mut_refs)]
mod interrupt;
mod mouse;
mod pci;
mod queue;
mod serial;
mod xhci;

use core::panic::PanicInfo;
use interrupt::{disable_maskable_interrupts, enable_maskable_interrupts};
use mikanos_rs_frame_buffer::{FrameBuffer, PixelColor};
use mouse::{MouseEvent, init_mouse};
use uefi::mem::memory_map::{MemoryMap, MemoryMapOwned};
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

struct Console {
    frame_buffer: &'static FrameBuffer,
    fg_color: PixelColor,
    bg_color: PixelColor,
    cursor_row: usize,
    cursor_col: usize,
    buffer: [[u8; Console::N_COLS]; Console::N_ROWS],
}

impl Console {
    const N_ROWS: usize = 37;
    const N_COLS: usize = 100;
    pub fn new(
        frame_buffer: &'static FrameBuffer,
        fg_color: PixelColor,
        bg_color: PixelColor,
    ) -> Self {
        Self {
            frame_buffer,
            fg_color,
            bg_color,
            cursor_row: 0,
            cursor_col: 0,
            buffer: [[0; Self::N_COLS]; Self::N_ROWS],
        }
    }
    pub fn put_string(&mut self, s: &str) {
        for c in s.as_bytes() {
            if *c == b'\n' {
                self.new_line();
                continue;
            }
            self.write_byte(*c);
        }
    }
    fn write_byte(&mut self, b: u8) {
        let x = 8 * self.cursor_col;
        let y = 16 * self.cursor_row;
        self.frame_buffer.write_ascii(x, y, b, &self.fg_color);
        self.buffer[self.cursor_row][self.cursor_col] = b;
        self.cursor_col += 1;
        if self.cursor_col == Self::N_COLS {
            self.new_line();
        }
    }
    fn new_line(&mut self) {
        self.cursor_col = 0;
        if self.cursor_row < Self::N_ROWS - 1 {
            self.cursor_row += 1;
        } else {
            self.scroll_line();
        }
    }
    fn scroll_line(&mut self) {
        self.frame_buffer.fill(&self.bg_color);
        self.cursor_col = 0;
        self.cursor_row = 0;
        for row in 0..(Self::N_ROWS - 1) {
            self.buffer[row] = self.buffer[row + 1];
            for col in 0..Self::N_COLS {
                self.write_byte(self.buffer[row][col]);
            }
        }
        self.buffer[Self::N_ROWS - 1].fill(0);
    }
}

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
pub extern "C" fn kernel_main_new_stack(
    frame_buffer: &'static FrameBuffer,
    memory_map: &'static MemoryMapOwned,
) {
    interrupt::init_idt();

    frame_buffer.fill(&PixelColor::new(255, 255, 255));

    let mut console = Console::new(
        &frame_buffer,
        PixelColor::new(0, 0, 0),
        PixelColor::new(255, 255, 255),
    );

    for _ in 0..4 {
        for i in 0..10 {
            let mut format_str: [u8; 256] = [0; 256];
            (&mut format_str[0..13]).copy_from_slice("? HelloWorld\n".as_bytes());
            format_str[0] = b'0' + i;
            console.put_string(core::str::from_utf8(&format_str[0..13]).unwrap());
        }
    }

    init_mouse(frame_buffer, (200, 100));
    for _ in 0..100 {
        let dummy_event = MouseEvent::new(0, -10, 0);
        mouse::get_mouse().lock().move_mouse(&dummy_event);

        for _ in 0..300000 {}
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
    let msi_cap_ptr = xhci_controller_addr.configure_msi(msg_addr, msg_data);

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

    // Start responding hardware interrupts.
    enable_maskable_interrupts();

    serial_println!("Checking for a xhc event...");
    // main event loop
    loop {
        disable_maskable_interrupts();
        if event::EVENT_QUEUE.lock().is_empty() {
            enable_maskable_interrupts();
            continue;
        }
        disable_maskable_interrupts();
        let event = event::EVENT_QUEUE.lock().pop().unwrap();
        enable_maskable_interrupts();

        match event {
            event::Event::XHCI => {
                //TODO: process all events that has been pushed into usb event ring.
                get_xhc().lock().process_event();
            }
            event::Event::Invalid => {
                serial_println!("invalid event!!");
                panic!()
            }
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
