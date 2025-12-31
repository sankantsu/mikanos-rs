use mikanos_rs_frame_buffer::{FrameBuffer, FrameBufferWriter, PixelColor};
use uefi::proto::console::gop::PixelFormat;

struct ShadowBuffer {
    buffer: alloc::vec::Vec<u8>,
    pixels_per_scanline: usize,
    horizontal_resolution: usize,
    vertical_resolution: usize,
    pixel_format: PixelFormat,
}

impl ShadowBuffer {
    pub fn new(
        pixels_per_scanline: usize,
        horizontal_resolution: usize,
        vertical_resolution: usize,
        pixel_format: PixelFormat,
    ) -> Self {
        let bufsize = 4 * pixels_per_scanline * vertical_resolution;
        Self {
            buffer: alloc::vec![0; bufsize],
            pixels_per_scanline,
            horizontal_resolution,
            vertical_resolution,
            pixel_format,
        }
    }
}

impl FrameBufferWriter for ShadowBuffer {
    fn get_buffer_mut(&self) -> *mut u8 {
        self.buffer.as_ptr() as *mut u8
    }

    fn size(&self) -> usize {
        4 * self.pixels_per_scanline * self.vertical_resolution
    }

    fn get_pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }

    fn get_pixels_per_scan_line(&self) -> usize {
        self.pixels_per_scanline
    }

    fn get_horizontal_resolution(&self) -> usize {
        self.horizontal_resolution
    }

    fn get_vertical_resolution(&self) -> usize {
        self.vertical_resolution
    }
}

pub struct Console {
    frame_buffer: &'static FrameBuffer,
    shadow_buffer: ShadowBuffer,
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
        let shadow_buffer = ShadowBuffer::new(
            frame_buffer.get_pixels_per_scan_line(),
            frame_buffer.get_horizontal_resolution(),
            frame_buffer.get_vertical_resolution(),
            frame_buffer.get_pixel_format(),
        );
        shadow_buffer.fill(&bg_color);
        Self {
            frame_buffer,
            shadow_buffer,
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
        self.shadow_buffer.write_ascii(x, y, b, &self.fg_color);
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
        self.shadow_buffer.fill(&self.bg_color);
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
    pub fn draw(&mut self) {
        let src = self.shadow_buffer.get_buffer_mut();
        let dst = self.frame_buffer.get_buffer_mut();
        let count = self.frame_buffer.size();
        unsafe {
            core::ptr::copy(src, dst, count);
        }
    }
}
