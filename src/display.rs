use embedded_graphics::{
    draw_target::DrawTarget,
    image::{Image, ImageRaw, ImageRawLE},
    mono_font::{ascii::FONT_7X13, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::Point,
    text::Text,
    Drawable,
};
use embedded_layout::{layout::linear::LinearLayout, prelude::Views, View};
use esp_idf_hal::i2c::I2cDriver;
use ssd1306::{
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::{DisplayRotation, I2CInterface},
    size::DisplaySize128x64,
    I2CDisplayInterface, Ssd1306,
};

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
    fn update_ui(&mut self) {
        let style = MonoTextStyle::new(&FONT_7X13, BinaryColor::On);
        let time = format!("Time: {}", self.time);
        let status = format!("Wi-Fi: {}", self.wifi_status.to_string());
        let wifi_status = Text::new(&status, Point::new(0, 60), style);
        

        let ip_string = format!("IP: {}", self.wifi_ip);
        let wether_string = format!("Weather: {}", self.weather);
        let ip_address = Text::new(&ip_string, Point::new(0, 90), style);
        let weather = Text::new(&wether_string, Point::new(0, 120), style);
        let time = Text::new(&time, Point::new(0, 120), style);
        let mut texts = [time, wifi_status, ip_address, weather];
        let views = Views::new(&mut texts);
        let layout = LinearLayout::vertical(views).arrange();
        self.screen.clear(BinaryColor::Off).unwrap();
        layout.draw(&mut self.screen).unwrap();
        let raw: ImageRawLE<'a, BinaryColor> =
        ImageRaw::new(include_bytes!("./assets/wifi.raw"), 16);
    Image::new(&raw, Point::new(0, 40))
        .draw(&mut self.screen)
        .unwrap();
        self.screen.flush().unwrap();
    }
}
