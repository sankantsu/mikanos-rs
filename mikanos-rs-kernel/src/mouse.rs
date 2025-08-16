use mikanos_rs_frame_buffer::{FrameBuffer, PixelColor};

pub struct MouseEvent {
    buttons: u8,
    displacement_x: i8,
    displacement_y: i8,
}

impl MouseEvent {
    pub fn new(buttons: u8, displacement_x: i8, displacement_y: i8) -> Self {
        Self {
            buttons,
            displacement_x,
            displacement_y,
        }
    }
}

pub struct Mouse {
    frame_buffer: &'static FrameBuffer,
    current_pos: (usize, usize),
}

unsafe impl Send for Mouse {}

pub fn get_mouse() -> &'static spin::Mutex<Option<Mouse>> {
    static MOUSE: spin::Mutex<Option<Mouse>> = spin::Mutex::new(None);
    &MOUSE
}

const MOUSE_CURSOR_WIDTH: usize = 15;
const MOUSE_CURSOR_HEIGHT: usize = 24;
const MOUSE_CURSOR: [[u8; MOUSE_CURSOR_WIDTH]; MOUSE_CURSOR_HEIGHT] = [
    [
        b'@', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'@', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'@', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'@', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'@', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'@', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'.', b'@', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'.', b'.', b'@', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'@', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'@', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'@', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'@', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'@', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'.', b'@', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'.', b'.', b'@', b'@', b'@', b'@', b'@', b'@', b'@', b'@',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'.', b'.', b'@', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'.', b'@', b'@', b'.', b'@', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'.', b'@', b' ', b'@', b'.', b'@', b' ', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'.', b'@', b' ', b' ', b' ', b'@', b'.', b'@', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'.', b'@', b' ', b' ', b' ', b' ', b'@', b'.', b'@', b' ', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b'@', b' ', b' ', b' ', b' ', b' ', b' ', b'@', b'.', b'@', b' ', b' ', b' ', b' ',
    ],
    [
        b'@', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b'@', b'.', b'@', b' ', b' ', b' ', b' ',
    ],
    [
        b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b'@', b'.', b'@', b' ', b' ', b' ',
    ],
    [
        b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b'@', b'@', b'@', b' ', b' ', b' ',
    ],
];

impl Mouse {
    pub fn new(frame_buffer: &'static FrameBuffer, initial_pos: (usize, usize)) -> Self {
        Self {
            frame_buffer,
            current_pos: initial_pos,
        }
    }

    pub fn move_mouse(&mut self, mouse_event: &MouseEvent) {
        self.erase_mouse();
        let (current_x, current_y) = self.current_pos;
        let pixels_per_scan_line = self.frame_buffer.get_pixels_per_scan_line() as i32;
        let vertical_resolution = self.frame_buffer.get_vertical_resolution() as i32;
        let new_x = i32::min(
            pixels_per_scan_line,
            i32::max(0, (current_x as i32) + (mouse_event.displacement_x as i32)),
        ) as usize;
        let new_y = i32::min(
            vertical_resolution,
            i32::max(0, (current_y as i32) + (mouse_event.displacement_y as i32)),
        ) as usize;
        self.current_pos = (new_x, new_y);
        self.draw_mouse();
    }

    pub fn draw_mouse(&self) {
        let (x, y) = self.current_pos;
        for dy in 0..MOUSE_CURSOR_HEIGHT {
            for dx in 0..MOUSE_CURSOR_WIDTH {
                let c = MOUSE_CURSOR[dy][dx];
                let pixels_per_scan_line = self.frame_buffer.get_pixels_per_scan_line();
                let vertical_resolution = self.frame_buffer.get_vertical_resolution();
                if x + dx >= pixels_per_scan_line || y + dy >= vertical_resolution {
                    continue;
                }
                if c == b'@' {
                    let black = &PixelColor::new(0, 0, 0);
                    self.frame_buffer.write_pixel(x + dx, y + dy, black);
                } else if c == b'.' {
                    let white = &PixelColor::new(255, 255, 255);
                    self.frame_buffer.write_pixel(x + dx, y + dy, white);
                }
            }
        }
    }

    fn erase_mouse(&self) {
        let (old_x, old_y) = self.current_pos;
        for dy in 0..MOUSE_CURSOR_HEIGHT {
            for dx in 0..MOUSE_CURSOR_WIDTH {
                let c = MOUSE_CURSOR[dy][dx];
                let pixels_per_scan_line = self.frame_buffer.get_pixels_per_scan_line();
                let vertical_resolution = self.frame_buffer.get_vertical_resolution();
                if old_x + dx >= pixels_per_scan_line || old_y + dy >= vertical_resolution {
                    continue;
                }
                self.frame_buffer.write_pixel(
                    old_x + dx,
                    old_y + dy,
                    &PixelColor::new(255, 255, 255),
                );
            }
        }
    }
}

pub extern "C" fn observer(buttons: u8, displacement_x: i8, displacement_y: i8) {
    let event = MouseEvent::new(buttons, displacement_x, displacement_y);
    get_mouse().lock().as_mut().unwrap().move_mouse(&event);
}
