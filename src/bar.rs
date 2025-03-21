use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{Point, Primitive},
    primitives::{Line, PrimitiveStyle},
    Drawable,
};
use esp_idf_hal::modem::Modem;

use crate::{oled::SSD1306Display, wifi};

pub fn init(display: &mut SSD1306Display, modem: Modem) {
    Line::new(Point::new(0, 10), Point::new(128, 10))
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(display)
        .unwrap();

    // wifi::Wifi::new(modem).connect(display);

}
