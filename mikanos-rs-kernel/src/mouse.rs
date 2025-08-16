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
        let (new_x, new_y) = (
            current_x + mouse_event.displacement_x as usize,
            current_y + mouse_event.displacement_y as usize,
        );
        self.current_pos = (new_x, new_y);
        self.draw_mouse();
    }

    pub fn draw_mouse(&self) {
        let (x, y) = self.current_pos;
        for dy in 0..MOUSE_CURSOR_HEIGHT {
            for dx in 0..MOUSE_CURSOR_WIDTH {
                let c = MOUSE_CURSOR[dy][dx];
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
                self.frame_buffer.write_pixel(
                    old_x + dx,
                    old_y + dy,
                    &PixelColor::new(255, 255, 255),
                );
            }
        }
    }
}
