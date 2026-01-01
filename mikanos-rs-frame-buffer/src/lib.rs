#![no_std]

mod font;

use core::slice;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

use font::FONTS;

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
