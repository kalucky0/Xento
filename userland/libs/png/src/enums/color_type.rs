pub enum ColorType {
    Grayscale = 0,
    Rgb = 2,
    Palette = 3,
    GrayscaleAlpha = 4,
    RgbAlpha = 6,
}

impl ColorType {
    pub fn new(value: u8) -> Option<ColorType> {
        match value {
            0 => Some(ColorType::Grayscale),
            2 => Some(ColorType::Rgb),
            3 => Some(ColorType::Palette),
            4 => Some(ColorType::GrayscaleAlpha),
            6 => Some(ColorType::RgbAlpha),
            _ => None,
        }
    }
}