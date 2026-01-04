use mikanos_rs_frame_buffer::{Font, FontMetrics, FrameBuffer, FrameBufferWriter, PixelColor};
use uefi::proto::console::gop::PixelFormat;

pub struct ShadowBuffer {
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

const CHAR_WIDTH: usize = 8;
const CHAR_HEIGHT: usize = 16;

pub struct Console {
    shadow_buffer: ShadowBuffer,
    fg_color: PixelColor,
    bg_color: PixelColor,
    cursor_row: usize,
    cursor_col: usize,
    font_data: fontdue::Font,
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
        let raw_font = include_bytes!("../fonts/Tamzen7x14r.ttf") as &[u8];
        let font_data =
            fontdue::Font::from_bytes(raw_font, fontdue::FontSettings::default()).unwrap();
        Self {
            shadow_buffer,
            fg_color,
            bg_color,
            cursor_row: 0,
            cursor_col: 0,
            font_data,
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
        let x = CHAR_WIDTH * self.cursor_col;
        let y = CHAR_HEIGHT * self.cursor_row;
        let (metrics, bitmap) = self.font_data.rasterize(b as char, 16.0);
        let metrics = FontMetrics::new(metrics.xmin, metrics.ymin, metrics.width, metrics.height);
        let font = Font::new(metrics, bitmap.as_ptr());
        self.shadow_buffer.write_char(x, y, &font, &self.fg_color);
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
        unsafe {
            let offset = 4 * self.shadow_buffer.get_pixels_per_scan_line() * CHAR_HEIGHT;
            let src = self.shadow_buffer.get_buffer_mut().add(offset);
            let dst = self.shadow_buffer.get_buffer_mut();
            let count =
                4 * self.shadow_buffer.get_pixels_per_scan_line() * CHAR_HEIGHT * Self::N_ROWS;
            core::ptr::copy(src, dst, count);
            for x in 0..self.shadow_buffer.get_horizontal_resolution() {
                for y in
                    ((Self::N_ROWS - 1) * CHAR_HEIGHT)..self.shadow_buffer.get_vertical_resolution()
                {
                    self.shadow_buffer.write_pixel(x, y, &self.bg_color)
                }
            }
        }
    }
    pub fn get_buffer(&mut self) -> &mut ShadowBuffer {
        &mut self.shadow_buffer
    }
}

pub fn copy_buffer<T: FrameBufferWriter, U: FrameBufferWriter>(src: &T, dest: &U) {
    assert_eq!(src.size(), dest.size());
    let src = src.get_buffer_mut();
    let dst = dest.get_buffer_mut();
    let count = dest.size();
    unsafe {
        core::ptr::copy(src, dst, count);
    }
}
