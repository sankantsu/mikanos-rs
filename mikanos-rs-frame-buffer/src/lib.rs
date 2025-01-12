#![no_std]

use core::slice;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

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

    pub fn size(&self) -> usize {
        4 * self.pixels_per_scanline * self.vertical_resolution
    }

    fn as_slice_mut(&self) -> &'static mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.frame_buffer, self.size()) }
    }

    pub fn write_pixel(&self, pos_x: usize, pos_y: usize, c: &PixelColor) {
        let pixel_idx = self.pixels_per_scanline * pos_y + pos_x;
        let pixel_format = self.pixel_format;
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

    pub fn fill(&self, color: &PixelColor) {
        for x in 0..self.horizontal_resolution {
            for y in 0..self.vertical_resolution {
                self.write_pixel(x, y, color)
            }
        }
    }
}
