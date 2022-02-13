use crate::renderer::LockedRenderer;
use alloc::format;
use embedded_graphics::{
    image::Image,
    mono_font::{ascii::FONT_8X13, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use tinytga::Tga;

pub fn show(renderer: &'static LockedRenderer) {
    let mut r = renderer.lock();
    let renderer = r.get();

    let display_area = renderer.bounding_box().size;

    let text_style = MonoTextStyle::new(&FONT_8X13, Rgb888::new(123, 124, 128));
    let style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb888::new(27, 29, 37))
        .build();

    Rectangle::new(Point::zero(), display_area)
        .into_styled(style)
        .draw(renderer)
        .unwrap();

    let ver = format!(
        "Xento v{} ({}x{})",
        env!("CARGO_PKG_VERSION"),
        renderer.info().horizontal_resolution,
        renderer.info().vertical_resolution
    );

    Text::new(
        &ver,
        Point::new(10, display_area.height as i32 - 10),
        text_style,
    )
    .draw(renderer)
    .unwrap();

    Text::new(
        "2022 (c) kalucky0",
        Point::new(
            display_area.width as i32 - 146,
            display_area.height as i32 - 10,
        ),
        text_style,
    )
    .draw(renderer)
    .unwrap();

    let data = include_bytes!("../resources/logo.tga");

    let tga: Tga<Rgb888> = Tga::from_slice(data).unwrap();

    let center_point = Point::new(
        display_area.width as i32 / 2 - 118,
        display_area.height as i32 / 2 - 100,
    );

    let image = Image::new(&tga, center_point);

    image.draw(renderer).unwrap();

    renderer.update();
}
