use mikanos_rs_frame_buffer::{FrameBufferWriter, PixelColor};

pub struct MouseEvent {
    _buttons: u8,
    displacement_x: i8,
    displacement_y: i8,
}

impl MouseEvent {
    pub fn new(buttons: u8, displacement_x: i8, displacement_y: i8) -> Self {
        Self {
            _buttons: buttons,
            displacement_x,
            displacement_y,
        }
    }
}

pub struct Mouse {
    current_pos: (usize, usize), // x, y
    screen_size: (usize, usize), // horizontal, vertical
}

static MOUSE: spin::Once<spin::Mutex<Mouse>> = spin::Once::new();

pub fn init_mouse(initial_pos: (usize, usize), screen_size: (usize, usize)) {
    MOUSE.call_once(|| spin::Mutex::new(Mouse::new(initial_pos, screen_size)));
    ()
}

pub fn get_mouse() -> &'static spin::Mutex<Mouse> {
    MOUSE.get().unwrap()
}

const MOUSE_CURSOR_WIDTH: usize = 15;
const MOUSE_CURSOR_HEIGHT: usize = 24;
const MOUSE_CURSOR: [&'static str; MOUSE_CURSOR_HEIGHT] = [
    "@              ",
    "@@             ",
    "@.@            ",
    "@..@           ",
    "@...@          ",
    "@....@         ",
    "@.....@        ",
    "@......@       ",
    "@.......@      ",
    "@........@     ",
    "@.........@    ",
    "@..........@   ",
    "@...........@  ",
    "@............@ ",
    "@......@@@@@@@@",
    "@......@       ",
    "@....@@.@      ",
    "@...@ @.@      ",
    "@..@   @.@     ",
    "@.@    @.@     ",
    "@@      @.@    ",
    "@       @.@    ",
    "         @.@   ",
    "         @@@   ",
];

impl Mouse {
    pub fn new(initial_pos: (usize, usize), screen_size: (usize, usize)) -> Self {
        Self {
            current_pos: initial_pos,
            screen_size,
        }
    }

    pub fn move_mouse(&mut self, mouse_event: &MouseEvent) {
        let (current_x, current_y) = self.current_pos;
        let screen_width = self.screen_size.0 as i32;
        let screen_height = self.screen_size.1 as i32;
        let new_x = i32::min(
            screen_width,
            i32::max(0, (current_x as i32) + (mouse_event.displacement_x as i32)),
        ) as usize;
        let new_y = i32::min(
            screen_height,
            i32::max(0, (current_y as i32) + (mouse_event.displacement_y as i32)),
        ) as usize;
        self.current_pos = (new_x, new_y);
    }

    pub fn draw_mouse<T: FrameBufferWriter>(&self, buffer: &mut T) {
        let (x, y) = self.current_pos;
        for dy in 0..MOUSE_CURSOR_HEIGHT {
            for dx in 0..MOUSE_CURSOR_WIDTH {
                let c = MOUSE_CURSOR[dy].as_bytes()[dx];
                let pixels_per_scan_line = buffer.get_pixels_per_scan_line();
                let vertical_resolution = buffer.get_vertical_resolution();
                if x + dx >= pixels_per_scan_line || y + dy >= vertical_resolution {
                    continue;
                }
                if c == b'@' {
                    let black = &PixelColor::new(0, 0, 0);
                    buffer.write_pixel(x + dx, y + dy, black);
                } else if c == b'.' {
                    let white = &PixelColor::new(255, 255, 255);
                    buffer.write_pixel(x + dx, y + dy, white);
                }
            }
        }
    }
}

pub extern "C" fn observer(buttons: u8, displacement_x: i8, displacement_y: i8) {
    let event = MouseEvent::new(buttons, displacement_x, displacement_y);
    get_mouse().lock().move_mouse(&event);
}
