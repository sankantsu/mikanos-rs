use mikanos_rs_frame_buffer::{FrameBuffer, PixelColor};

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

pub fn draw_mouse(frame_buffer: &FrameBuffer, x: usize, y: usize) {
    for dy in 0..MOUSE_CURSOR_HEIGHT {
        for dx in 0..MOUSE_CURSOR_WIDTH {
            let c = MOUSE_CURSOR[dy][dx];
            if c == b'@' {
                let black = &PixelColor::new(0, 0, 0);
                frame_buffer.write_pixel(x + dx, y + dy, black);
            } else if c == b'.' {
                let white = &PixelColor::new(255, 255, 255);
                frame_buffer.write_pixel(x + dx, y + dy, white);
            }
        }
    }
}
