use crate::xhci::get_xhc;

fn notify_end_of_interrupt() {
    let eoi_reg = 0xfee000b0 as *mut u32;
    unsafe { *eoi_reg = 0 };
}

pub extern "x86-interrupt" fn handle_xhci_event() {
    get_xhc().lock().process_event();
    notify_end_of_interrupt();
}
