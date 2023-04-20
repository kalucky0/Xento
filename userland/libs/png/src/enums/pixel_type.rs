use super::{ColorType, BitDepth};

#[derive(Clone, Copy)]
pub enum PixelType {
    Grayscale1,
    Grayscale2,
    Grayscale4,
    Grayscale8,
    Grayscale16,
    Rgb8,
    Rgb16,
    Palette1,
    Palette2,
    Palette4,
    Palette8,
    GrayscaleAlpha8,
    GrayscaleAlpha16,
    RgbAlpha8,
    RgbAlpha16,
}

impl PixelType {
    pub fn new(color: ColorType, depth: BitDepth) -> Option<PixelType> {
        match color {
            ColorType::Grayscale => match depth {
                BitDepth::One => Some(PixelType::Grayscale1),
                BitDepth::Two => Some(PixelType::Grayscale2),
                BitDepth::Four => Some(PixelType::Grayscale4),
                BitDepth::Eight => Some(PixelType::Grayscale8),
                BitDepth::Sixteen => Some(PixelType::Grayscale16),
            },
            ColorType::Rgb => match depth {
                BitDepth::Eight => Some(PixelType::Rgb8),
                BitDepth::Sixteen => Some(PixelType::Rgb16),
                _ => None,
            },
            ColorType::Palette => match depth {
                BitDepth::One => Some(PixelType::Palette1),
                BitDepth::Two => Some(PixelType::Palette2),
                BitDepth::Four => Some(PixelType::Palette4),
                BitDepth::Eight => Some(PixelType::Palette8),
                _ => None,
            },
            ColorType::GrayscaleAlpha => match depth {
                BitDepth::Eight => Some(PixelType::GrayscaleAlpha8),
                BitDepth::Sixteen => Some(PixelType::GrayscaleAlpha16),
                _ => None,
            },
            ColorType::RgbAlpha => match depth {
                BitDepth::Eight => Some(PixelType::RgbAlpha8),
                BitDepth::Sixteen => Some(PixelType::RgbAlpha16),
                _ => None,
            },
        }
    }
}