use alloc::vec;
use alloc::vec::Vec;
use bootloader::boot_info::{FrameBufferInfo, PixelFormat};
use conquer_once::spin::OnceCell;
use core::fmt::{self, Write};
use embedded_graphics::{
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    text::Text
};
use spinning_top::{RawSpinlock, Spinlock};

pub static RENDERER: OnceCell<LockedRenderer> = OnceCell::uninit();
pub struct LockedRenderer(Spinlock<Renderer>);

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
    }

    fn flush(&self) {}
}

pub struct Renderer {
    framebuffer: &'static mut [u8],
    buffer: Vec<u8>,
    info: FrameBufferInfo,
}

#[derive(Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn u8(&self, pixel_format: PixelFormat) -> [u8; 4] {
        match pixel_format {
            PixelFormat::RGB => [self.r, self.g, self.b, 0],
            PixelFormat::BGR => [self.b, self.g, self.r, 0],
            PixelFormat::U8 => [self.r, self.g, self.b, 0],
            _ => [self.r, self.g, self.b, 0],
        }
    }

    pub const fn rgb888(&self) -> Rgb888 {
        Rgb888::new(self.r, self.g, self.b)
    }

    pub fn from_rgb888(rgb: Rgb888) -> Self {
        Self {
            r: rgb.r(),
            g: rgb.g(),
            b: rgb.b(),
        }
    }
}

impl DrawTarget for Renderer {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let width = self.info.horizontal_resolution;
        let height = self.info.vertical_resolution;

        for Pixel(coord, color) in pixels.into_iter() {
            let x = coord.x as usize;
            let y = coord.y as usize;
            if x < width && y < height {
                let pixel_offset = y * self.info.stride + x;
                let color = Color::from_rgb888(color).u8(self.info.pixel_format);

                self.buffer[pixel_offset * self.info.bytes_per_pixel
                    ..(pixel_offset + 1) * self.info.bytes_per_pixel]
                    .copy_from_slice(&color);
            }
        }

        Ok(())
    }
}

impl OriginDimensions for Renderer {
    fn size(&self) -> Size {
        Size::new(
            self.info.horizontal_resolution as u32,
            self.info.vertical_resolution as u32,
        )
    }
}

impl Renderer {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut renderer = Self {
            framebuffer,
            buffer: vec![
                0;
                info.vertical_resolution
                    * info.horizontal_resolution
                    * info.bytes_per_pixel
            ],
            info
        };
        renderer.clear();
        renderer
    }

    pub fn get(&mut self) -> &mut Self {
        self
    }

    pub fn info(&self) -> &FrameBufferInfo {
        &self.info
    }

    pub fn clear(&mut self) {
        self.buffer.fill(0);
    }

    pub fn fill(&mut self, color: Color) {
        let color = color.u8(self.info.pixel_format);
        for x in 0..self.width() {
            for y in 0..self.height() {
                let index = (y * self.info.horizontal_resolution + x) * self.info.bytes_per_pixel;
                self.buffer[index..index + 4].copy_from_slice(&color);
            }
        }
    }

    pub fn update(&mut self) {
        self.framebuffer.copy_from_slice(&self.buffer);
    }

    pub fn width(&self) -> usize {
        self.info.horizontal_resolution
    }

    pub fn height(&self) -> usize {
        self.info.vertical_resolution
    }
}

unsafe impl Send for Renderer {}
unsafe impl Sync for Renderer {}

impl fmt::Write for Renderer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let style = MonoTextStyle::new(&FONT_8X13, Rgb888::WHITE);
        Text::new(s, Point::new(10, 10), style);
        Ok(())
    }
}
