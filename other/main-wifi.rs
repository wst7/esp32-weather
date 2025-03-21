use display_interface_spi::SPIInterface;
use embedded_graphics::{
    pixelcolor::{Rgb565, Rgb888},
    prelude::{Point, Primitive, RgbColor, Size},
    primitives::{Circle, Rectangle, Triangle},
    text::{renderer::CharacterStyle, Text, TextStyleBuilder},
    Drawable, Pixel,
};
use esp_idf_hal::{
    gpio::PinDriver,
    prelude::Peripherals,
    spi::{self, config::DriverConfig, Spi, SpiConfig, SpiDriver},
    units::Hertz,
};
use esp_idf_svc::{eventloop, http, io::Write, log, nvs, sys, wifi};
use heapless::String;

use mipidsi::{interface::SpiInterface, Builder};
use mipidsi::{models::ST7789, options::ColorInversion, Builder};
use serde::{Deserialize, Serialize};
use st7789::{Orientation, ST7789};
use std::{thread, time::Duration};

struct WifiNetwork {
    ssid: String<32>,
}
impl Serialize for WifiNetwork {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.ssid)
    }
}

#[derive(Deserialize)]
struct WifiCredentials {
    ssid: &'static str,
    password: &'static str,
}

fn main() {
    // const INDEX: &[u8] = include_bytes!("index.html");
    // const SSID: &str = "ChinaUnicom-UG9SCX";
    // const PASSWORD: &str = "Buxingjie007";
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    log::EspLogger::initialize_default();
    // 获取外设
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // 配置SPI
    // let spi = SpiDriver::new(
    //     peripherals.spi2,
    //     peripherals.pins.gpio18, // SCL
    //     peripherals.pins.gpio19, // SDA
    //     None,                    // 不使用MISO
    //     &DriverConfig::new(),
    // )
    // .unwrap();
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss1, 60_000_000_u32, Mode::Mode0).unwrap();

    // 配置GPIO引脚
    let rst = PinDriver::output(peripherals.pins.gpio4).unwrap(); // RES
    let dc = PinDriver::output(peripherals.pins.gpio2).unwrap(); // DC
    let cs = PinDriver::output(peripherals.pins.gpio5).unwrap(); // CS
                                                                 // 初始化ST7789显示屏

    // create a SpiInterface from SpiDevice, a DC pin and a buffer
    let mut buffer = [0u8; 512];
    let di = SpiInterface::new(spi, dc, &mut buffer);
    // create the ILI9486 display driver in rgb666 color mode from the display interface and use a HW reset pin during init
    let mut display = Builder::new(ILI9486Rgb666, di)
        .reset_pin(rst)
        .init(&mut delay)?; // delay provider from your MCU
                            // clear the display to black
    display.clear(Rgb666::BLACK)?;

    println!("Display initialized and graphics drawn!");
    // 进入无限循环，防止程序退出
    loop {
        thread::sleep(Duration::from_secs(10));
    }
    // // 初始化 NVS（用于 Wi-Fi 配置存储）
    // let nvs = nvs::EspDefaultNvsPartition::take().unwrap();

    // let event_loop = eventloop::EspEventLoop::take().unwrap();
    // let modem = unsafe { esp_idf_svc::hal::modem::Modem::new() };

    // let mut wifi = wifi::EspWifi::new(modem, event_loop, Some(nvs)).unwrap();

    // // 设置 Wi-Fi 模式为 STA（Station 模式）
    // wifi.set_configuration(&wifi::Configuration::Client(wifi::ClientConfiguration {
    //     ssid: String::try_from(SSID).unwrap(),
    //     password: String::try_from(PASSWORD).unwrap(),
    //     ..Default::default()
    // }))
    // .unwrap();
    // // 启动 Wi-Fi
    // wifi.start().unwrap();
    // println!("WiFi 启动成功...");

    // // 连接到 Wi-Fi
    // wifi.connect().unwrap();
    // println!("正在连接 WiFi...");

    // // 等待连接成功
    // while !wifi.is_connected().unwrap() {
    //     println!("等待 Wi-Fi 连接...");
    //     thread::sleep(Duration::from_secs(1));
    // }
    // // 获取 IP 地址
    // if let Some(ip_info) = wifi.sta_netif().get_ip_info().ok() {
    //     println!("Wi-Fi 连接成功! IP 地址: {:?}", ip_info.ip);
    // } else {
    //     println!("Wi-Fi 连接失败！");
    // }
    // let networks = wifi.scan().unwrap();
    // // 启动HTTP服务器
    // let server_config = http::server::Configuration {
    //     stack_size: 10240, // 设置服务器栈大小
    //     ..Default::default()
    // };
    // let mut server = http::server::EspHttpServer::new(&server_config).unwrap();
    // // 定义一个简单的HTTP路由
    // server
    //     .fn_handler("/", http::Method::Get, |request| {

    //         request
    //             .into_ok_response()?
    //             .write_all(INDEX)
    //     })
    //     .unwrap();
    // server.fn_handler("/scan", http::Method::Get, move |request| {

    //     let mut wifi_networks: Vec<WifiNetwork> = Vec::new();
    //         println!("scan wifi: {:#?}", networks);

    //     for network in &networks {
    //         wifi_networks.push(WifiNetwork {
    //             ssid: String::try_from(network.ssid.clone()).unwrap(),
    //         });
    //     }
    //     let json_response = serde_json::to_string(&wifi_networks).unwrap();
    //     request
    //             .into_ok_response()?
    //             .write_all(json_response.as_bytes())

    // }).unwrap();

    // println!("Web server started on port 80");
}
