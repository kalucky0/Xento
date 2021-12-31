use alloc::vec;
use alloc::vec::Vec;
use bootloader::boot_info::{FrameBufferInfo, PixelFormat};
use conquer_once::spin::OnceCell;
use core::{
    fmt::{self, Write},
    ptr,
};
use font8x8::UnicodeFonts;
use spinning_top::{RawSpinlock, Spinlock};

pub static RENDERER: OnceCell<LockedRenderer> = OnceCell::uninit();
pub struct LockedRenderer(Spinlock<Renderer>);

/// Additional vertical space between lines
const LINE_SPACING: usize = 2;
/// Additional vertical space between separate log messages
const LOG_SPACING: usize = 2;

impl LockedRenderer {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        LockedRenderer(Spinlock::new(Renderer::new(framebuffer, info)))
    }

    pub fn lock(&self) -> spinning_top::lock_api::MutexGuard<'_, RawSpinlock, Renderer> {
        self.0.lock()
    }

    pub unsafe fn force_unlock(&self) {
        self.0.force_unlock();
    }
}

impl log::Log for LockedRenderer {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut renderer = self.0.lock();
        if record.level() == log::Level::Info {
            writeln!(renderer, "{}", record.args()).unwrap();
        } else {
            writeln!(renderer, "{}: {}", record.level(), record.args()).unwrap();
        }
        renderer.add_vspace(LOG_SPACING);
    }

    fn flush(&self) {}
}

pub struct Renderer {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
    color: Color,
    should_clear: bool,
}

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Renderer {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut renderer = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
            color: Color {
                r: 74,
                g: 246,
                b: 38,
            },
            should_clear: true,
        };
        renderer.clear();
        renderer
    }

    fn newline(&mut self) {
        self.y_pos += 8 + LINE_SPACING;
        self.carriage_return()
    }

    pub fn add_vspace(&mut self, space: usize) {
        self.y_pos += space;
    }

    pub fn sub_vspace(&mut self, space: usize) {
        self.y_pos -= space;
    }

    pub fn set_vspace(&mut self, space: usize) {
        self.y_pos = space;
    }

    pub fn add_hspace(&mut self, space: usize) {
        self.x_pos += space;
    }

    pub fn sub_hspace(&mut self, space: usize) {
        self.x_pos -= space;
    }

    pub fn set_hspace(&mut self, space: usize) {
        self.x_pos = space;
    }

    pub fn should_clear(&mut self, should_clear: bool) {
        self.should_clear = should_clear;
    }

    fn carriage_return(&mut self) {
        self.x_pos = 0;
    }

    pub fn remove_last(&mut self, n: usize) {
        self.x_pos -= n * 8;
        for _ in 0..n {
            self.write_char(' ');
        }
        self.x_pos -= n * 8;
    }

    pub fn set_text_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn clear(&mut self) {
        self.x_pos = 0;
        self.y_pos = 0;
        self.framebuffer.fill(0);
    }

    fn width(&self) -> usize {
        self.info.horizontal_resolution
    }

    fn height(&self) -> usize {
        self.info.vertical_resolution
    }

    pub fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                if self.x_pos >= self.width() {
                    self.newline();
                }
                if self.y_pos >= (self.height() - 8) {
                    self.clear();
                }
                let rendered = crate::font::FONTS
                    .get(c)
                    .expect("character not found in basic font");
                self.write_rendered_char(rendered);
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

    pub fn write_centered_string(&mut self, s: &str) {
        self.x_pos = (self.width() - s.len() * 8) / 2;
        self.write_string(s);
    }

    fn write_rendered_char(&mut self, rendered_char: [u8; 8]) {
        for (y, byte) in rendered_char.iter().enumerate() {
            for (x, bit) in (0..8).enumerate() {
                let alpha = if *byte & (1 << bit) == 0 { 0 } else { 255 };
                self.write_pixel(self.x_pos + x, self.y_pos + y, alpha);
            }
        }
        self.x_pos += 8;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        if !self.should_clear && intensity == 0 {
            return;
        }

        let pixel_offset = y * self.info.stride + x;
        let color = if intensity != 0 {
            match self.info.pixel_format {
                PixelFormat::RGB => [self.color.r, self.color.g, self.color.b, 0],
                PixelFormat::BGR => [self.color.b, self.color.g, self.color.r, 0],
                PixelFormat::U8 => [self.color.r, self.color.g, self.color.b, 0],
                _ => [self.color.r, self.color.g, self.color.b, 0],
            }
        } else {
            match self.info.pixel_format {
                PixelFormat::RGB => [0, 0, 0, 0],
                PixelFormat::BGR => [0, 0, 0, 0],
                PixelFormat::U8 => [0, 0, 0, 0],
                _ => [0, 0, 0, 0],
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl fmt::Write for Renderer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
