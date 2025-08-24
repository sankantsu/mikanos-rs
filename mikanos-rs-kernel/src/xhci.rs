#[allow(non_camel_case_types, dead_code)]
#[derive(Debug)]
#[repr(C)]
pub enum ErrorCode {
    kSuccess,
    kFull,
    kEmpty,
    kNoEnoughMemory,
    kIndexOutOfRange,
    kHostControllerNotHalted,
    kInvalidSlotID,
    kPortNotConnected,
    kInvalidEndpointNumber,
    kTransferRingNotSet,
    kAlreadyAllocated,
    kNotImplemented,
    kInvalidDescriptor,
    kBufferTooSmall,
    kUnknownDevice,
    kNoCorrespondingSetupStage,
    kTransferFailed,
    kInvalidPhase,
    kUnknownXHCISpeedID,
    kNoWaiter,
    kNoPCIMSI,
    kUnknownPixelFormat,
    kNoSuchTask,
    kInvalidFormat,
    kFrameTooSmall,
    kInvalidFile,
    kIsDirectory,
    kNoSuchEntry,
    kFreeTypeError,
    kLastOfCode,
}

impl ErrorCode {
    fn is_success(&self) -> bool {
        match self {
            Self::kSuccess => true,
            _ => false,
        }
    }
}
type MouseObserverType = extern "C" fn(buttons: u8, displacement_x: i8, displacement_y: i8);

unsafe extern "C" {
    fn create_xhci_controller(mmmio_base: u64) -> *mut Controller;
    fn initialize_xhci_controller(xhc: &mut Controller) -> ErrorCode;
    fn start_xhci_controller(xhc: &mut Controller) -> ErrorCode;
    fn configure_xhci_port(xhc: &mut Controller, port_num: u8) -> ErrorCode;
    fn process_xhci_event(xhc: &mut Controller) -> ErrorCode;
    fn set_default_mouse_observer(observer: MouseObserverType);
    fn set_default_keyboard_observer();
}

// Opaque type
pub enum Controller {}

impl Controller {
    pub fn new(mmio_base: u64) -> &'static mut Self {
        unsafe { &mut *create_xhci_controller(mmio_base) }
    }
    pub fn init(&mut self) {
        let err = unsafe { initialize_xhci_controller(self) };
        if !err.is_success() {
            crate::serial_println!("xHCI initialization failed!: {:?}", err);
            panic!();
        }
    }
    pub fn run(&mut self) {
        let err = unsafe { start_xhci_controller(self) };
        if !err.is_success() {
            crate::serial_println!("xHCI start failed!: {:?}", err);
            panic!();
        }
    }
    pub fn configure_port(&mut self, port: u8) {
        let err = unsafe { configure_xhci_port(self, port) };
        if !err.is_success() {
            crate::serial_println!("Error ocurred during configureing port{}: {:?}", port, err);
        }
    }
    pub fn process_event(&mut self) {
        let err = unsafe { process_xhci_event(self) };
        if !err.is_success() {
            crate::serial_println!("Error ocurred during processing xHCI event: {:?}", err);
        }
    }
}

pub fn initialize_mouse() {
    unsafe { set_default_mouse_observer(crate::mouse::observer) };
}

pub fn initialize_keyboard() {
    unsafe { set_default_keyboard_observer() };
}
