use embedded_graphics::{
    draw_target::DrawTarget,
    image::{Image, ImageRaw, ImageRawLE},
    pixelcolor::BinaryColor,
    prelude::Point,
    text::Text,
    Drawable,
};
use embedded_layout::{layout::linear::LinearLayout, prelude::Chain};
use esp_idf_hal::i2c::I2cDriver;
use ssd1306::{
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::{DisplayRotation, I2CInterface},
    size::DisplaySize128x64,
    I2CDisplayInterface, Ssd1306,
};
use u8g2_fonts::{fonts::u8g2_font_wqy12_t_gb2312, U8g2TextStyle};

use crate::wifi::WifiStatus;

pub type SSD1306Display<'a> = Ssd1306<
    I2CInterface<I2cDriver<'a>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

pub struct Display<'a> {
    screen: SSD1306Display<'a>,
    weather: String,
    wifi_ip: String,
    wifi_status: WifiStatus,
    time: String,
}

impl<'a> Display<'a> {
    pub fn new(i2c: I2cDriver<'a>) -> Self {
        let interface = I2CDisplayInterface::new(i2c);
        let mut screen = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        screen.init().expect("display init");
        let mut ins = Self {
            screen,
            weather: String::from("--"),
            wifi_ip: String::from("0.0.0.0"),
            wifi_status: WifiStatus::Disconnected,
            time: String::from("00:00:00"),
        };
        ins.update_ui();
        ins
    }
    pub fn update_wifi(&mut self, status: WifiStatus, ip: String) {
        self.wifi_status = status;
        self.wifi_ip = ip;
        self.update_ui();
    }

    pub fn update_weather(&mut self, weather: String) {
        self.weather = weather;
        self.update_ui();
    }
    pub fn update_time(&mut self, time: String) {
        self.time = time;
        self.update_ui();
    } 

    // TODO: 优化布局
    fn update_ui(&mut self) {
        let style = U8g2TextStyle::new(u8g2_font_wqy12_t_gb2312, BinaryColor::On);

        let ip_string = format!("IP: {}", self.wifi_ip);
        let ip_address = Text::new(&ip_string, Point::new(0, 90), style.clone());

        let wether_string = format!("温度: {}℃", self.weather);
        let weather = Text::new(&wether_string, Point::new(0, 120), style.clone());

        let time = format!("时间: {}", self.time);
        let time = Text::new(&time, Point::new(0, 120), style.clone());

        let raw: ImageRawLE<'a, BinaryColor> =
            ImageRaw::new(include_bytes!("./assets/wifi.raw"), 16);
        let icon = Image::new(&raw, Point::new(0, 40));

        let views = Chain::new(LinearLayout::horizontal(Chain::new(icon).append(time)))
            .append(ip_address)
            .append(weather);

        let layout = LinearLayout::vertical(views).arrange();

        self.screen.clear(BinaryColor::Off).unwrap();

        layout.draw(&mut self.screen).unwrap();

        self.screen.flush().unwrap();
    }
    pub fn show_message(&mut self, message: &str) {
        let style = U8g2TextStyle::new(u8g2_font_wqy12_t_gb2312, BinaryColor::On);
        let text = Text::new(message, Point::new(0, 0), style.clone());
        self.screen.clear(BinaryColor::Off).unwrap();
        text.draw(&mut self.screen).unwrap();
        self.screen.flush().unwrap();
    }
}
