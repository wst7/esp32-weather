use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::{Point, Primitive, Size},
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
    Drawable,
};
use esp_idf_hal::{delay::Delay, i2c::I2cDriver};
use qrcodegen::{QrCode, QrCodeEcc};
use ssd1306::{
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::{DisplayRotation, I2CInterface},
    size::DisplaySize128x64,
    I2CDisplayInterface, Ssd1306,
};

pub struct OLed<'a> {
    display: SSD1306Display<'a>,
    // battery: u8,
}

pub type SSD1306Display<'a> = Ssd1306<
    I2CInterface<I2cDriver<'a>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

impl<'a> OLed<'a> {
    pub fn new(i2c: I2cDriver<'a>) -> Self {
        // 创建 SSD1306 驱动实例
        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init().expect("display init");

        // 清屏并绘制内容
        display.clear(BinaryColor::Off).unwrap();

        // TODO: add start screen image
        display.flush().unwrap();
        Self {
            display,
            // battery: 100,
        }
    }
    // pub fn get_display(&mut self) -> &mut SSD1306Display<'a> {
    //     &mut self.display
    // }

    // pub fn set_battery(&mut self, battery: u8) {
    //     self.battery = battery;
    // }
    pub fn show_message(&mut self, msg: &str) {
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        self.display.clear(BinaryColor::Off).unwrap();
        Text::new(msg, Point::new(0, 10), text_style)
            .draw(&mut self.display)
            .unwrap();
        self.display.flush().unwrap();
        let delay: Delay = Default::default();
        delay.delay_ms(500);
    }

    pub fn draw_qr_code(&mut self, text: &str) {
        println!("drawing qr code address: {}", text);
        let qr = QrCode::encode_text(text, QrCodeEcc::Medium).unwrap();

        for y in 0..qr.size() {
            for x in 0..qr.size() {
                if qr.get_module(x, y) {
                    // 在 OLED 上绘制 QR 码的像素
                    Rectangle::new(Point::new(x as i32 * 2 + 40, y as i32 * 2 + 10), Size::new(2, 2))
                        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                        .draw(&mut self.display)
                        .unwrap();
                }
            }
        }
        self.display.flush().unwrap();
    }

    pub fn clear(&mut self) {
        self.display.clear(BinaryColor::Off).unwrap();
        self.display.flush().unwrap();
    }
}
