use crate::{AncillaryChunks, enums::PixelType};

pub struct ScanlineIterator<'a> {
    extra_chunks: &'a AncillaryChunks<'a>,
    width: usize,
    cursor: usize,
    pixel_type: PixelType,
    scanline: &'a [u8],
}

impl<'a> ScanlineIterator<'a> {
    pub fn new(
        extra_chunks: &'a AncillaryChunks<'a>,
        image_width: u32,
        pixel_type: PixelType,
        scanline: &'a [u8],
    ) -> ScanlineIterator<'a> {
        Self {
            extra_chunks,
            width: image_width as usize,
            cursor: 0,
            pixel_type,
            scanline,
        }
    }
}

impl<'a> Iterator for ScanlineIterator<'a> {
    type Item = (u8, u8, u8, u8);

    fn next(&mut self) -> Option<(u8, u8, u8, u8)> {
        if self.width < self.cursor {
            return None;
        }

        let pixel = match self.pixel_type {
            PixelType::Grayscale1 => self.get_grayscale1(),
            PixelType::Grayscale2 => self.get_grayscale2(),
            PixelType::Grayscale4 => self.get_grayscale4(),
            PixelType::Grayscale8 => self.get_grayscale8(),
            PixelType::Grayscale16 => self.get_grayscale16(),
            PixelType::Rgb8 => self.get_rgb8(),
            PixelType::Rgb16 => self.get_rgb16(),
            PixelType::Palette1 => self.get_palette1(),
            PixelType::Palette2 => self.get_palette2(),
            PixelType::Palette4 => self.get_palette4(),
            PixelType::Palette8 => self.get_palette8(),
            PixelType::GrayscaleAlpha8 => self.get_grayscale_alpha8(),
            PixelType::GrayscaleAlpha16 => self.get_grayscale_alpha16(),
            PixelType::RgbAlpha8 => self.get_rgb_alpha8(),
            PixelType::RgbAlpha16 => self.get_rgb_alpha16(),
        };

        self.cursor += 1;
        pixel
    }
}

impl<'a> ScanlineIterator<'a> {
    fn get_grayscale1(&mut self) -> Option<(u8, u8, u8, u8)> {
        let byte = self.scanline[self.cursor / 8];
        let bit = 7 - (self.cursor % 8);
        let value = (byte >> bit) & 0b1;
        let value = value * 255;
        // TODO: Transparency
        Some((value, value, value, 255))
    }

    fn get_grayscale2(&mut self) -> Option<(u8, u8, u8, u8)> {
        let byte = self.scanline[self.cursor / 4];
        let bit = 6 - (self.cursor % 4) * 2;
        let value = (byte >> bit) & 0b11;
        let value = (value as f32 / 3.0) * 255.0;
        let value = value as u8;
        // TODO: Transparency
        Some((value, value, value, 255))
    }

    fn get_grayscale4(&mut self) -> Option<(u8, u8, u8, u8)> {
        let byte = self.scanline[self.cursor / 2];
        let bit = 4 - (self.cursor % 2) * 4;
        let value = (byte >> bit) & 0b1111;
        let value = (value as f32 / 15.0) * 255.0;
        let value = value as u8;
        // TODO: Transparency
        Some((value, value, value, 255))
    }

    fn get_grayscale8(&mut self) -> Option<(u8, u8, u8, u8)> {
        let value = self.scanline[self.cursor];
        // TODO: Transparency
        Some((value, value, value, 255))
    }

    fn get_grayscale16(&mut self) -> Option<(u8, u8, u8, u8)> {
        let cursor = self.cursor * 2;
        let value = u16::from_be_bytes([self.scanline[cursor], self.scanline[cursor + 1]]);
        let value = (value >> 8) as u8;
        // TODO: Transparency
        Some((value, value, value, 255))
    }

    fn get_rgb8(&mut self) -> Option<(u8, u8, u8, u8)> {
        let cursor = self.cursor * 3;
        let r = self.scanline[cursor];
        let g = self.scanline[cursor + 1];
        let b = self.scanline[cursor + 2];
        // TODO: Transparency
        Some((r, g, b, 255))
    }

    fn get_rgb16(&mut self) -> Option<(u8, u8, u8, u8)> {
        let cursor = self.cursor * 6;
        let r = u16::from_be_bytes([self.scanline[cursor], self.scanline[cursor + 1]]);
        let g = u16::from_be_bytes([self.scanline[cursor + 2], self.scanline[cursor + 3]]);
        let b = u16::from_be_bytes([self.scanline[cursor + 4], self.scanline[cursor + 5]]);
        let r = (r >> 8) as u8;
        let g = (g >> 8) as u8;
        let b = (b >> 8) as u8;
        // TODO: Transparency
        Some((r, g, b, 255))
    }

    fn get_palette1(&mut self) -> Option<(u8, u8, u8, u8)> {
        let byte = self.scanline[self.cursor / 8];
        let bit = 7 - (self.cursor % 8);
        let index = (byte >> bit) & 0b1;
        let pallete = match self.extra_chunks.palette() {
            Some(pallete) => pallete,
            None => return None,
        };
        let r = pallete[index as usize];
        let g = pallete[index as usize + 1];
        let b = pallete[index as usize + 2];
        // TODO: Transparency
        Some((r, g, b, 255))
    }

    fn get_palette2(&mut self) -> Option<(u8, u8, u8, u8)> {
        let byte = self.scanline[self.cursor / 4];
        let bit = 6 - (self.cursor % 4) * 2;
        let index = (byte >> bit) & 0b11;
        let pallete = match self.extra_chunks.palette() {
            Some(pallete) => pallete,
            None => return None,
        };
        let r = pallete[index as usize];
        let g = pallete[index as usize + 1];
        let b = pallete[index as usize + 2];
        // TODO: Transparency
        Some((r, g, b, 255))
    }

    fn get_palette4(&mut self) -> Option<(u8, u8, u8, u8)> {
        let byte = self.scanline[self.cursor / 2];
        let bit = 4 - (self.cursor % 2) * 4;
        let index = (byte >> bit) & 0b1111;
        let pallete = match self.extra_chunks.palette() {
            Some(pallete) => pallete,
            None => return None,
        };
        let r = pallete[index as usize];
        let g = pallete[index as usize + 1];
        let b = pallete[index as usize + 2];
        // TODO: Transparency
        Some((r, g, b, 255))
    }

    fn get_palette8(&mut self) -> Option<(u8, u8, u8, u8)> {
        let index = self.scanline[self.cursor];
        let pallete = match self.extra_chunks.palette() {
            Some(pallete) => pallete,
            None => return None,
        };
        let r = pallete[index as usize];
        let g = pallete[index as usize + 1];
        let b = pallete[index as usize + 2];
        // TODO: Transparency
        Some((r, g, b, 255))
    }

    fn get_grayscale_alpha8(&mut self) -> Option<(u8, u8, u8, u8)> {
        let cursor = self.cursor * 2;
        let value = self.scanline[cursor];
        let alpha = self.scanline[cursor + 1];
        Some((value, value, value, alpha))
    }

    fn get_grayscale_alpha16(&mut self) -> Option<(u8, u8, u8, u8)> {
        let cursor = self.cursor * 4;
        let value = u16::from_be_bytes([self.scanline[cursor], self.scanline[cursor + 1]]);
        let alpha = u16::from_be_bytes([self.scanline[cursor + 2], self.scanline[cursor + 3]]);
        let value = (value >> 8) as u8;
        let alpha = (alpha >> 8) as u8;
        Some((value, value, value, alpha))
    }

    fn get_rgb_alpha8(&mut self) -> Option<(u8, u8, u8, u8)> {
        let cursor = self.cursor * 4;
        let r = self.scanline[cursor];
        let g = self.scanline[cursor + 1];
        let b = self.scanline[cursor + 2];
        let alpha = self.scanline[cursor + 3];
        Some((r, g, b, alpha))
    }

    fn get_rgb_alpha16(&mut self) -> Option<(u8, u8, u8, u8)> {
        let cursor = self.cursor * 8;
        let r = u16::from_be_bytes([self.scanline[cursor], self.scanline[cursor + 1]]);
        let g = u16::from_be_bytes([self.scanline[cursor + 2], self.scanline[cursor + 3]]);
        let b = u16::from_be_bytes([self.scanline[cursor + 4], self.scanline[cursor + 5]]);
        let alpha = u16::from_be_bytes([self.scanline[cursor + 6], self.scanline[cursor + 7]]);
        let r = (r >> 8) as u8;
        let g = (g >> 8) as u8;
        let b = (b >> 8) as u8;
        let alpha = (alpha >> 8) as u8;
        Some((r, g, b, alpha))
    }
}
