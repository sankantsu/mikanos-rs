#![no_std]

mod font;

use core::slice;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

use font::FONTS;

pub struct FontMetrics {
    xmin: i32,
    ymin: i32,
    width: usize,
    height: usize,
}

impl FontMetrics {
    pub fn new(xmin: i32, ymin: i32, width: usize, height: usize) -> Self {
        Self {
            xmin,
            ymin,
            width,
            height,
        }
    }
}

pub struct Font {
    metrics: FontMetrics,
    bitmap_ptr: *const u8,
}

impl Font {
    pub fn new(metrics: FontMetrics, bitmap_ptr: *const u8) -> Self {
        Self {
            metrics,
            bitmap_ptr,
        }
    }

    pub fn get_bitmap(&self) -> &[u8] {
        let size = self.metrics.width * self.metrics.height;
        unsafe { core::slice::from_raw_parts(self.bitmap_ptr, size) }
    }
}

pub struct PixelColor {
    r: u8,
    g: u8,
    b: u8,
}

impl PixelColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

pub trait FrameBufferWriter {
    fn get_buffer_mut(&self) -> *mut u8;
    fn size(&self) -> usize;
    fn get_pixels_per_scan_line(&self) -> usize;
    fn get_horizontal_resolution(&self) -> usize;
    fn get_vertical_resolution(&self) -> usize;
    fn get_pixel_format(&self) -> PixelFormat;

    // Default impls
    fn as_slice_mut(&self) -> &'static mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.get_buffer_mut(), self.size()) }
    }

    fn write_pixel(&self, pos_x: usize, pos_y: usize, c: &PixelColor) {
        let pixel_idx = self.get_pixels_per_scan_line() * pos_y + pos_x;
        let pixel_format = self.get_pixel_format();
        let p = self.as_slice_mut();
        match pixel_format {
            PixelFormat::Rgb => {
                p[4 * pixel_idx] = c.r;
                p[4 * pixel_idx + 1] = c.g;
                p[4 * pixel_idx + 2] = c.b;
            }
            PixelFormat::Bgr => {
                p[4 * pixel_idx] = c.b;
                p[4 * pixel_idx + 1] = c.g;
                p[4 * pixel_idx + 2] = c.r;
            }
            _ => unimplemented!(),
        }
    }

    fn write_ascii(&self, x: usize, y: usize, ch: u8, color: &PixelColor) {
        for dy in 0..16 {
            for dx in 0..8 {
                if ((FONTS[ch as usize][dy] << dx) & 0x80) != 0 {
                    self.write_pixel(x + dx, y + dy, color);
                }
            }
        }
    }

    fn write_char(&self, x: usize, y: usize, f: &Font, color: &PixelColor) {
        let threshold = 80;
        let bitmap = f.get_bitmap();
        for i in 0..f.metrics.height {
            for j in 0..f.metrics.width {
                let idx = i * f.metrics.width + j;
                if bitmap[idx] < threshold {
                    continue;
                }
                let py =
                    (y as i32 + 16 - f.metrics.height as i32 - f.metrics.ymin + i as i32) as usize;
                let px = (x as i32 + j as i32) as usize;
                if py < self.get_vertical_resolution() && px < self.get_horizontal_resolution() {
                    self.write_pixel(px, py, color);
                }
            }
        }
    }

    fn write_string(&self, x: usize, y: usize, s: &str, color: &PixelColor) {
        for (idx, ch) in s.as_bytes().iter().enumerate() {
            self.write_ascii(x + 8 * idx, y, *ch, color);
        }
    }

    fn fill(&self, color: &PixelColor) {
        for x in 0..self.get_horizontal_resolution() {
            for y in 0..self.get_vertical_resolution() {
                self.write_pixel(x, y, color)
            }
        }
    }
}

#[repr(C)]
pub struct FrameBuffer {
    frame_buffer: *mut u8,
    pixels_per_scanline: usize,
    horizontal_resolution: usize,
    vertical_resolution: usize,
    pixel_format: PixelFormat,
}

impl FrameBuffer {
    pub fn new(gop: &mut GraphicsOutput) -> Self {
        let (horizontal, vertical) = gop.current_mode_info().resolution();
        let pixel_format = gop.current_mode_info().pixel_format();

        Self {
            frame_buffer: gop.frame_buffer().as_mut_ptr(),
            pixels_per_scanline: gop.current_mode_info().stride(),
            horizontal_resolution: horizontal,
            vertical_resolution: vertical,
            pixel_format,
        }
    }
}

impl FrameBufferWriter for FrameBuffer {
    fn get_buffer_mut(&self) -> *mut u8 {
        self.frame_buffer
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
