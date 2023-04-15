use fontdue::{
    layout::{CoordinateSystem, Layout, TextStyle},
    Font,
};
use tiny_skia::Pixmap;

pub fn render(font: &Font, text: &str, size: f32, color: u32) -> Pixmap {
    let fonts = &[font];
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.append(fonts, &TextStyle::new(text, size, 0));

    let height = layout.height();
    let mut width = 0;
    for glyph in layout.glyphs() {
        let x = glyph.x as usize + glyph.width;
        width = width.max(x);
    }

    let mut pixmap = Pixmap::new(width as u32, height as u32).unwrap();
    let data = pixmap.data_mut();

    let r = ((color >> 16) & 0xFF) as f32;
    let g = ((color >> 8) & 0xFF) as f32;
    let b = (color & 0xFF) as f32;
    let a = ((color >> 24) & 0xFF) as f32;

    for glyph in layout.glyphs() {
        let (_, bitmap) = font.rasterize(glyph.parent, size);
        let glyph_x = glyph.x as usize;
        let glyph_y = glyph.y as usize;

        for gx in 0..glyph.width {
            for gy in 0..glyph.height {
                let index = (gy * glyph.width + gx) as usize;
                let alpha = bitmap[index];

                let index = ((glyph_y + gy) * width + glyph_x + gx) as usize * 4;
                if index < data.len() {
                    let f_alpha = alpha as f32 / 255.0;
                    data[index + 0] = (r * f_alpha) as u8;
                    data[index + 1] = (g * f_alpha) as u8;
                    data[index + 2] = (b * f_alpha) as u8;
                    data[index + 3] = (a * f_alpha) as u8;
                }
            }
        }
    }

    pixmap
}

pub fn measure_width(font: &Font, text: &str, size: f32) -> f32 {
    let fonts = &[font];
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.append(fonts, &TextStyle::new(text, size, 0));

    let mut width = 0;
    for glyph in layout.glyphs() {
        let x = glyph.x as usize + glyph.width;
        width = width.max(x);
    }

    width as f32
}