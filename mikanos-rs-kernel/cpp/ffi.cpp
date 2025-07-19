#include <array>
#include "error.hpp"
#include "logger.hpp"
#include "usb/xhci/port.hpp"
#include "usb/xhci/xhci.hpp"
#include "usb/classdriver/mouse.hpp"
#include "usb/classdriver/keyboard.hpp"

// MikanOS libcxx_support depends on printk()
int printk(const char* format, ...) {
    return 0;
}

extern "C" {

void* create_xhci_controller(uint64_t mmio_base) {
    static usb::xhci::Controller xhc{mmio_base};
    return (void*)&xhc;
}

int initialize_xhci_controller(usb::xhci::Controller* xhc) {
    int res = (int)xhc->Initialize().Cause();
    return res;
}

int start_xhci_controller(usb::xhci::Controller* xhc) {
    int res = (int)xhc->Run().Cause();
    return res;
}

int configure_xhci_port(usb::xhci::Controller* xhc, uint8_t port_num) {
    usb::xhci::Port port = xhc->PortAt(port_num);
    if (port.IsConnected()) {
        Log(kInfo, "Configuring port %d.\n", port_num);
        int res = usb::xhci::ConfigurePort(*xhc, port).Cause();
        return res;
    }
    Log(kDebug, "Ignoring non-connected port %d.\n", port_num);
    return (int)Error::kSuccess;
}

int process_xhci_event(usb::xhci::Controller* xhc) {
    int res = usb::xhci::ProcessEvent(*xhc).Cause();
    return res;
}

void set_default_mouse_observer() {
    usb::HIDMouseDriver::default_observer =
        [](uint8_t buttons, int8_t displacement_x, int8_t displacement_y) {
            Log(kInfo, "Mouse event: buttons=%d, displacement=(%d,%d)\n", buttons, displacement_x, displacement_y);
        };
}

void set_default_keyboard_observer() {
    usb::HIDKeyboardDriver::default_observer =
        [](uint8_t modifier, uint8_t keycode, bool press) {
            Log(kInfo, "Keyboard event: modifier=%d, keycode=%d, press=%d\n", modifier, keycode, press);
        };
}

// Test functions for FFI functionality

int add(int a, int b) {
    return a + b;
}

int foo() {
    std::array<int, 16> v{};
    for (int i = 0; i < 16; i++) {
        v[i] = i;
    }

    int s = 0;
    for (auto x : v) {
        s += x;
    }
    return s;
}

}
