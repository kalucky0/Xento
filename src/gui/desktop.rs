use crate::renderer::{LockedRenderer};
use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_6X12, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use embedded_layout::{layout::linear::LinearLayout, prelude::*};

pub struct Desktop {
    renderer: &'static LockedRenderer,
}

impl Desktop {
    pub fn new(renderer: &'static LockedRenderer) -> Self {
        Desktop { renderer }
    }

    pub fn start(&mut self) {
        let mut r = self.renderer.lock();
        let renderer = r.get();

        let display_area = renderer.bounding_box();
        let display_width = display_area.size.width as u32;
        let display_height = display_area.size.height as u32;

        let text_style = MonoTextStyle::new(&FONT_6X12, Rgb888::new(218, 218, 218));
        // let clock_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(218, 218, 218));
        let line_style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(86, 87, 93))
            .build();
        let style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb888::new(27, 29, 37))
            .build();

        let panel_width = display_width / 3;
        let fs_height = (display_height as f32 * 0.3) as u32;

        let space = |width, height| {
            Rectangle::new(Point::new(0, 0), Size::new(width, height)).into_styled(style)
        };

        let title_bar = |left, right, len: u32| {
            LinearLayout::vertical(
                Chain::new(
                    LinearLayout::horizontal(
                        Chain::new(space(5, 1))
                            .append(Text::new(left, Point::zero(), text_style))
                            .append(space(panel_width / 2 - (len + 3) * 6, 1))
                            .append(Text::new(right, Point::zero(), text_style)),
                    )
                        .arrange(),
                )
                    .append(
                        LinearLayout::horizontal(
                            Chain::new(
                                Rectangle::new(Point::zero(), Size::new(1, 8)).into_styled(line_style),
                            )
                                .append(
                                    Rectangle::new(Point::zero(), Size::new(panel_width / 2 - 12, 1))
                                        .into_styled(line_style),
                                )
                                .append(
                                    Rectangle::new(Point::zero(), Size::new(1, 8)).into_styled(line_style),
                                ),
                        )
                            .with_alignment(vertical::Center)
                            .arrange(),
                    ),
            )
                .arrange()
        };

        Rectangle::new(Point::zero(), display_area.size)
            .into_styled(style)
            .draw(renderer)
            .unwrap();

        let rect_style = PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::new(86, 87, 93))
            .stroke_width(1)
            .build();

        // let (clock_u8, clock_width) = self.clock("19:21:21");
        // let clock_raw = ImageRaw::<Rgb888>::new(&clock_u8, clock_width);

        let system_panel = LinearLayout::vertical(
            Chain::new(title_bar("PANEL", "SYSTEM", 11))
                .append(title_bar(" ", " ", 2))
                .append(space(1, 11))
                // .append(Image::new(&clock_raw, Point::zero()))
                .append(title_bar(" ", " ", 2))
                .append(
                    Rectangle::new(
                        Point::zero(),
                        Size::new(panel_width / 2, display_height - fs_height - 106),
                    )
                        .into_styled(rect_style),
                ),
        )
            .with_alignment(horizontal::Center)
            .arrange();

        let network_panel = LinearLayout::vertical(
            Chain::new(title_bar("PANEL", "NETWORK", 12))
                .append(title_bar(" ", " ", 2))
                .append(
                    Rectangle::new(
                        Point::zero(),
                        Size::new(panel_width / 2 + 1, display_height - fs_height - 40),
                    )
                        .into_styled(rect_style),
                ),
        )
            .with_alignment(horizontal::Center)
            .arrange();

        let side_panel_top =
            LinearLayout::horizontal(Chain::new(system_panel).append(network_panel))
                .with_alignment(vertical::Top)
                .arrange();

        let side_panel_bottom = Rectangle::new(Point::zero(), Size::new(panel_width, fs_height))
            .into_styled(rect_style);

        let side_panel =
            LinearLayout::vertical(Chain::new(side_panel_top).append(side_panel_bottom)).arrange();

        let main_panel = Rectangle::new(
            Point::zero(),
            Size::new(display_width - panel_width, display_height),
        )
            .into_styled(rect_style);

        LinearLayout::horizontal(Chain::new(side_panel).append(main_panel))
            .with_alignment(vertical::Center)
            .arrange()
            .align_to(&display_area, horizontal::Center, vertical::Center)
            .draw(renderer)
            .unwrap();

        renderer.update();
    }
}
