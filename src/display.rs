use embedded_graphics::{
    draw_target::DrawTarget,
    image::{Image, ImageRaw},
    pixelcolor::{raw::LittleEndian, BinaryColor},
    prelude::{Point, Primitive},
    primitives::{Line, PrimitiveStyle},
    text::Text,
    Drawable,
};
use embedded_layout::{
    align::vertical::{self},
    layout::linear::{spacing::DistributeFill, LinearLayout},
    prelude::Chain,
};
use esp_idf_svc::hal::i2c::I2cDriver;
use ssd1306::{
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::{DisplayRotation, I2CInterface},
    size::DisplaySize128x64,
    I2CDisplayInterface, Ssd1306,
};
use u8g2_fonts::{
    fonts::{u8g2_font_wqy12_t_gb2312, u8g2_font_wqy16_t_gb2312},
    U8g2TextStyle,
};

use crate::{state::State, wifi::WifiStatus};

pub type SSD1306Display<'a> = Ssd1306<
    I2CInterface<I2cDriver<'a>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;
/***
 *  ----------------------------
 * |wifi_icon               date|
 * |----------------------------|
 * |  time                      |
 * |  weather                   |
 *  ----------------------------
 *
 *
 *
 */
pub struct Display<'a> {
    screen: SSD1306Display<'a>,
}

impl<'a> Display<'a> {
    pub fn new(i2c: I2cDriver<'a>) -> Self {
        let interface = I2CDisplayInterface::new(i2c);
        let mut screen = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        screen.init().expect("display init");
        Self { screen }
    }

    pub fn show_message(&mut self, message: &str) {
        let style = U8g2TextStyle::new(u8g2_font_wqy12_t_gb2312, BinaryColor::On);
        let text = Text::new(message, Point::new(0, 0), style.clone());
        self.screen.clear(BinaryColor::Off).unwrap();
        text.draw(&mut self.screen).unwrap();
        self.screen.flush().unwrap();
    }

    pub fn update_home(
        &mut self,
        state: State,
    ) {
        let style_12 = U8g2TextStyle::new(u8g2_font_wqy12_t_gb2312, BinaryColor::On);
        let style_16 = U8g2TextStyle::new(u8g2_font_wqy16_t_gb2312, BinaryColor::On);
        // let ip_string = format!("IP: {}", self.wifi_ip);
        // let ip_address = Text::new(&ip_string, Point::new(0, 90), style.clone());

        let wether_string = format!("{} {}", state.city, state.weather);
        let weather = Text::new(&wether_string, Point::zero(), style_12.clone());

        let time = format!("{}", state.time);
        let time = Text::new(&time, Point::zero(), style_16.clone());

        let date = Text::new(&state.date, Point::zero(), style_12.clone());
        let raw: ImageRaw<'a, BinaryColor, LittleEndian> =
            ImageRaw::new(include_bytes!("./assets/wifi.raw"), 16);
        let icon = Image::new(&raw, Point::zero());
        let bar = LinearLayout::horizontal(Chain::new(icon).append(date))
            .with_spacing(DistributeFill(128))
            .with_alignment(vertical::Center)
            .arrange();
        let line = Line::new(Point::zero(), Point::new(128, 0))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1)); // 添加样式

        let status_bar = LinearLayout::vertical(Chain::new(bar).append(line)).arrange();
        let views = Chain::new(status_bar).append(time).append(weather);

        let layout = LinearLayout::vertical(views)
            .with_spacing(DistributeFill(64))
            .arrange();

        self.screen.clear(BinaryColor::Off).unwrap();

        layout.draw(&mut self.screen).unwrap();

        self.screen.flush().unwrap();
    }

    pub fn show_wifi_config(&mut self) {
        self.show_message("请输入WiFi名称和密码");
    }
}
