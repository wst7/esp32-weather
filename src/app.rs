use crate::{
    button::Button,
    display::Display,
    weather::WeatherClient,
    wifi::{WiFiManager, WifiStatus},
};
use chrono::Utc;
use chrono_tz::Asia;
use esp_idf_hal::{
    i2c::{I2cConfig, I2cDriver},
    prelude::Peripherals,
    sys,
    task::block_on,
    units::KiloHertz,
};
use esp_idf_svc::sntp::{EspSntp, SntpConf, SyncStatus};
use log::info;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Clone)]
pub enum DeviceStatus {
    Booting,
    Idle,
    Connecting,
    Connected,
    Updating,
    Sleeping,
    Rebooting,
}
pub struct App {
    display: Arc<Mutex<Display<'static>>>,
    wifi: Arc<Mutex<WiFiManager<'static>>>,
    weather_client: Arc<Mutex<WeatherClient>>,
    status: Arc<Mutex<DeviceStatus>>,
    idle_loop_time: u64,
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
        let status = Arc::new(Mutex::new(DeviceStatus::Booting));

        Ok(Self {
            display,
            wifi,
            weather_client,
            status,
            idle_loop_time: 0
        })
    }

    pub fn start(&mut self) {
        self.update_status(DeviceStatus::Booting);
        let peripherals = Peripherals::take().unwrap();
        let mut button = Button::new(peripherals.pins.gpio4).unwrap();
        let status = self.status.clone();
        let _ = button.subscribe(move || {
            let mut status = status.lock().unwrap();
            *status = DeviceStatus::Updating;
        });
        block_on(self.run_state_machine());
    }
    async fn run_state_machine(&mut self) {
        loop {
            let current_status = self.status.lock().unwrap().clone();
            match current_status {
                DeviceStatus::Booting => {
                    self.handle_booting().await;
                    self.update_status(DeviceStatus::Connecting);
                }
                DeviceStatus::Connecting => {
                    self.handle_connecting().await;
                    self.update_status(DeviceStatus::Connected);
                }
                DeviceStatus::Connected => {
                    self.handle_connected().await;
                    self.update_status(DeviceStatus::Idle);
                }
                DeviceStatus::Updating => {
                    self.handle_updating().await;
                    self.update_status(DeviceStatus::Rebooting);
                }
                DeviceStatus::Idle => {
                    self.handle_idle();
                }
                DeviceStatus::Sleeping => self.handle_sleeping(),
                DeviceStatus::Rebooting => {
                    self.handle_rebooting().await;
                }
            }
            thread::sleep(Duration::from_secs(1)); // 防止 CPU 占用过高
        }
    }
    async fn handle_booting(&self) {
        self.display.lock().unwrap().show_message("设备启动中...");
        thread::sleep(Duration::from_secs(2));
    }
    async fn handle_connecting(&self) {
        self.display.lock().unwrap().show_message("连接中...");
        self.connect_wifi().await;
    }
    async fn handle_connected(&self) {
        self.display
            .lock()
            .unwrap()
            .show_message("连接成功, 正在同步时间...");
        self.sync_time().await;
    }
    async fn handle_updating(&self) {
        self.display.lock().unwrap().show_message("更新中...");
        // TODO: 等待更新完成
        thread::sleep(Duration::from_secs(2));
    }
    async fn handle_rebooting(&self) {
        self.display.lock().unwrap().show_message("即将重启...");
        thread::sleep(Duration::from_secs(2));
        unsafe { sys::esp_restart() };
    }
    fn handle_idle(&mut self) {
        let display = self.display.clone();
        let weather_client = self.weather_client.clone();
        // 更新时间&天气
        let now = Utc::now().with_timezone(&Asia::Shanghai);
        let time = now.format("%H:%M:%S").to_string();
        display.lock().unwrap().update_time(time);

        // TODO: 有问题,使用thread
        // esp-idf-hal/examples/ledc_threads.rs at master · esp-rs/esp-idf-hal
        if self.idle_loop_time % 600 == 0 {
            let wt = weather_client.lock().unwrap().fetch_weather().unwrap();
            display.lock().unwrap().update_weather(wt);
            self.idle_loop_time = 0;
        }
        self.idle_loop_time += 1;
    }
    fn handle_sleeping(&self) {
        // TODO: 实现休眠功能
        self.display.lock().unwrap().show_message("休眠中...");
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
        })
        .unwrap();
        // 等待 NTP 时间同步
        loop {
            if _sntp.get_sync_status() == SyncStatus::Completed {
                info!("NTP 时间同步完成");
                break;
            }
            info!("等待 NTP 时间同步...");
            thread::sleep(Duration::from_secs(1));
        }
    }
    fn update_status(&self, new_status: DeviceStatus) {
        let mut status = self.status.lock().unwrap();
        *status = new_status;
    }
}
