use std::{error::Error, thread, time::Duration};

use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{ascii::FONT_6X9, MonoTextStyleBuilder},
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor},
    text::{Baseline, Text},
    Drawable,
};
use esp_idf_hal::{
    delay::Ets,
    gpio::{AnyIOPin, AnyOutputPin, Gpio15, Gpio18, Gpio2, Gpio23, Gpio4, Gpio5, PinDriver},
    spi::{
        config::{Config, DriverConfig},
        SpiDeviceDriver, SpiDriver, SPI2,
    },
};
use esp_idf_svc::log;
use mipidsi::{
    interface::SpiInterface,
    models::ST7789,
    options::{Orientation, Rotation},
    Builder,
};

fn main() -> Result<(), Box<dyn Error>> {
    // 重要：确保 ESP-IDF 运行时正确链接
    esp_idf_sys::link_patches();

    // 启动 ESP32 日志
    log::EspLogger::initialize_default();

    // ✅ 正确的 GPIO 配置
    let mut rst = PinDriver::output(unsafe { Gpio4::new() })?; // RST 复位引脚
    let dc = PinDriver::output(unsafe { Gpio2::new() })?; // DC（数据/命令）
    let cs = PinDriver::output(unsafe { Gpio5::new() })?; // CS（片选）
    let mut bl = PinDriver::output(unsafe { Gpio15::new() })?; // BL（背光）
                                                               // ✅ 启用背光

    let mut delay = Ets;

    // ✅ SPI 配置
    let sclk = unsafe { Gpio18::new() }; // SCK (SPI 时钟)
    let mosi = unsafe { Gpio23::new() }; // MOSI (SPI 数据输出)
    let spi = unsafe { SPI2::new() };

    // 创建 SPI 驱动
    let spi_driver = SpiDriver::new(spi, sclk, mosi, None::<AnyIOPin>, &DriverConfig::new())?;
    let spi_device = SpiDeviceDriver::new(spi_driver, None::<AnyOutputPin>, &Config::new())?;

    // ✅ 修正 SPIInterface 缓冲区
    let mut buf = [0u8; 512];
    let di = SpiInterface::new(spi_device, dc, &mut buf);

     // ✅ 复位 ST7789
     rst.set_low()?; 
     thread::sleep(Duration::from_millis(300)); // ✅ 增加到 300ms
     rst.set_high()?; 
     thread::sleep(Duration::from_millis(300)); 

     
    // ✅ 初始化 ST7789 显示屏
    let mut display = Builder::new(ST7789, di)
        .init(&mut delay)
        .map_err(|_| Box::<dyn Error>::from("ST7789 显示屏初始化失败"))?;

    bl.set_high()?;
    display.clear(Rgb565::WHITE).unwrap();
    // ✅ 设置屏幕旋转方向
    // display
    //     .set_orientation(Orientation::default().rotate(Rotation::Deg180))
    //     .unwrap();

    // ✅ 在屏幕上绘制 "Hello ST7789"
    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X9)
        .text_color(Rgb565::WHITE)
        .build();

    Text::with_baseline(
        "Hello ST7789",
        Point::new(20, 50),
        character_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    println!("ST7789 显示屏初始化完成！");

    Ok(())
}
