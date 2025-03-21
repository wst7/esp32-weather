use crate::{
    display::Display,
    weather::WeatherClient,
    wifi::{WiFiManager, WifiStatus},
};
use chrono::Utc;
use chrono_tz::Asia;
use esp_idf_hal::{
    i2c::{I2cConfig, I2cDriver},
    prelude::Peripherals,
    task::block_on,
    units::KiloHertz,
};
use esp_idf_svc::sntp::{EspSntp, SntpConf};
use log::info;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub struct App {
    display: Arc<Mutex<Display<'static>>>,
    wifi: Arc<Mutex<WiFiManager<'static>>>,
    weather_client: Arc<Mutex<WeatherClient>>,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let peripherals = Peripherals::take().unwrap();
        // config i2c
        let i2c = peripherals.i2c0;
        let sda = peripherals.pins.gpio21;
        let scl = peripherals.pins.gpio22;
        let modem = peripherals.modem;

        let config = I2cConfig::new().baudrate(KiloHertz::from(100).into());
        let i2c = I2cDriver::new(i2c, sda, scl, &config).unwrap();

        let display = Arc::new(Mutex::new(Display::new(i2c)));
        let wifi = Arc::new(Mutex::new(WiFiManager::new(modem)?));
        let weather_client = Arc::new(Mutex::new(WeatherClient::new()));
        Ok(Self {
            display,
            wifi,
            weather_client,
        })
    }

    pub fn start(&self) {
       

        block_on(self.connect_wifi());

        block_on(self.sync_time());

        block_on(async {
            let display = self.display.clone();
            let weather_client = self.weather_client.clone();
            // 更新时间&天气
            let mut loop_time = 0;
            loop {
                let now = Utc::now().with_timezone(&Asia::Shanghai);
                let time = now.format("%H:%M:%S").to_string();
                display.lock().unwrap().update_time(time);
                
                // TODO: 有问题,使用thread
                // https://github.com/esp-rs/esp-idf-hal/blob/master/examples/ledc_threads.rs
                if loop_time % 600 == 0 {
                    let wt = weather_client.lock().unwrap().fetch_weather().unwrap();
                    display.lock().unwrap().update_weather(wt);
                    loop_time = 0;
                }
                loop_time += 1;
                thread::sleep(Duration::from_secs(1));
            }
        });
        

        loop {
            thread::sleep(Duration::from_secs(1000));
        }
    }

    async fn connect_wifi(&self) {
        let wifi = self.wifi.clone();
        let display = self.display.clone();
        wifi.lock().unwrap().connect().await.unwrap();
        loop {
            let (status, ip) = wifi.lock().unwrap().get_wifi_status().await.unwrap();
            display.lock().unwrap().update_wifi(status.clone(), ip);
            if status == WifiStatus::Connected {
                println!("Wi-Fi connected! Breaking the loop.");
                break;
            }
            println!("Wi-Fi not connected. Retrying in 2 seconds...");
            thread::sleep(Duration::from_secs(2));
        }
    }
    async fn sync_time(&self) {
        let _sntp = EspSntp::new(&SntpConf {
            servers: ["ntp.aliyun.com"], // ✅ 修改 NTP 服务器
            ..Default::default()
        }).unwrap();
        // 等待 NTP 时间同步
        loop {
            if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                if now.as_secs() > 1700000000 {
                    break; // 时间有效（大于 2023 年）
                }
            }
            info!("等待 NTP 时间同步...");
            thread::sleep(Duration::from_secs(1));
        }
    }
}
