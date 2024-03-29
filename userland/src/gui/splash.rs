use super::widgets::text;
use crate::{renderer::LockedRenderer, resources};
use alloc::format;
use fontdue::Font;
use tiny_skia::{Color, PixmapPaint, Transform};

pub fn render(renderer: &'static LockedRenderer) {
    let mut r = renderer.lock();
    let renderer = r.get();

    renderer.fill(Color::from_rgba8(27, 29, 37, 255));

    let width = renderer.width() as i32;
    let height = renderer.height() as i32;

    let ver = format!(
        "Xento v{} ({}x{})",
        env!("CARGO_PKG_VERSION"),
        renderer.width(),
        renderer.height()
    );

    let paint = PixmapPaint::default();
    let font = Font::from_bytes(resources::JETBRAINS_MONO_BOLD, fontdue::FontSettings::default()).unwrap();
    renderer.pixmap().draw_pixmap(
        12,
        height - 26,
        text::render(&font, &ver, 14.0, 0x7b7c80).as_ref(),
        &paint,
        Transform::identity(),
        None,
    );

    renderer.pixmap().draw_pixmap(
        width - 164,
        height - 26,
        text::render(&font, "2023 (c) kalucky0", 14.0, 0x7b7c80).as_ref(),
        &paint,
        Transform::identity(),
        None,
    );

    renderer.update();
}
