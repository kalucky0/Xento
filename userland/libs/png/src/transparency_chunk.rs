use crate::{Chunk, PixelType};

pub enum TransparencyChunk<'a> {
    Grayscale(u8),
    Palette(&'a [u8]),
    Rgb(u8, u8, u8),
}

impl<'a> TransparencyChunk<'a> {
    pub fn new(chunk: &Chunk<'a>, pixel_type: PixelType) -> Option<TransparencyChunk<'a>> {
        let data = chunk.data();
        match pixel_type {
            PixelType::Grayscale1 => Some(TransparencyChunk::Grayscale(data[1] & 0b1)),
            PixelType::Grayscale2 => Some(TransparencyChunk::Grayscale(data[1] & 0b11)),
            PixelType::Grayscale4 => Some(TransparencyChunk::Grayscale(data[1] & 0b1111)),
            PixelType::Grayscale8 => Some(TransparencyChunk::Grayscale(data[1])),
            PixelType::Grayscale16 => Some(TransparencyChunk::Grayscale(
                (u16::from_be_bytes([data[0], data[1]]) >> 8) as u8,
            )),
            PixelType::Palette1 => Some(TransparencyChunk::Palette(data)),
            PixelType::Palette2 => Some(TransparencyChunk::Palette(data)),
            PixelType::Palette4 => Some(TransparencyChunk::Palette(data)),
            PixelType::Palette8 => Some(TransparencyChunk::Palette(data)),
            PixelType::Rgb8 => Some(TransparencyChunk::Rgb(data[1], data[3], data[5])),
            PixelType::Rgb16 => Some(TransparencyChunk::Rgb(
                (u16::from_be_bytes([data[0], data[1]]) >> 8) as u8,
                (u16::from_be_bytes([data[2], data[3]]) >> 8) as u8,
                (u16::from_be_bytes([data[4], data[5]]) >> 8) as u8,
            )),
            PixelType::GrayscaleAlpha8 => None,
            PixelType::GrayscaleAlpha16 => None,
            PixelType::RgbAlpha8 => None,
            PixelType::RgbAlpha16 => None,
        }
    }
}
