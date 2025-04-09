use crate::{
    button::Button,
    display::Display,
    state::State,
    weather::WeatherClient,
    wifi::{WiFiManager, WifiStatus},
};
use chrono::Utc;
use chrono_tz::Asia::Shanghai;
use esp_idf_svc::{
    hal::{
        gpio::{Gpio15, Gpio18, Gpio19, Gpio4},
        i2c::{I2cConfig, I2cDriver},
        prelude::Peripherals,
        task::block_on,
        units::KiloHertz,
    },
    nvs::{EspDefaultNvsPartition, EspNvs},
    sntp::{EspSntp, SntpConf, SyncStatus},
};
use log::info;
use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

#[derive(Clone, Debug)]
pub enum AppPage {
    Home,
    WifiConfig,
}
impl AppPage {
    pub fn next(&self) -> Self {
        match self {
            AppPage::Home => AppPage::WifiConfig,
            AppPage::WifiConfig => AppPage::Home,
        }
    }
    pub fn prev(&self) -> Self {
        match self {
            AppPage::Home => AppPage::WifiConfig,
            AppPage::WifiConfig => AppPage::Home,
        }
    }
}

#[derive(Debug)]
pub enum DisplayMessage {
    ShowPage(AppPage),
    UpdateTime(String, String),
    UpdateWeather(String, String),
    UpdateWifi(WifiStatus, String),
}

pub struct App {
    current_page: Arc<Mutex<AppPage>>,
    display_tx: Sender<DisplayMessage>,
    wifi: Arc<Mutex<WiFiManager<'static>>>,
    weather_client: Arc<Mutex<WeatherClient>>,
    display: Arc<Mutex<Display<'static>>>,
    state: Arc<Mutex<State>>,
    left_button: Arc<Mutex<Button<Gpio4>>>,
    right_button: Arc<Mutex<Button<Gpio15>>>,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let nvs_partition = EspDefaultNvsPartition::take()?;
        let mut wifi_nvs = EspNvs::new(nvs_partition, "wifi", true)?;

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
        let (display_tx, display_rx) = mpsc::channel();
        let ins = Self {
            current_page: Arc::new(Mutex::new(AppPage::Home)),
            wifi,
            weather_client,
            display,
            display_tx,
            state: Arc::new(Mutex::new(State::new())),
            left_button: Arc::new(Mutex::new(Button::new(peripherals.pins.gpio4)?)),
            right_button: Arc::new(Mutex::new(Button::new(peripherals.pins.gpio15)?)),
        };
        let ins_state = ins.state.clone();
        thread::Builder::new()
            .stack_size(6000)
            .spawn(move || {
                for message in display_rx {
                    info!("DisplayController: {:?}", message);
                    match message {
                        DisplayMessage::ShowPage(page) => {
                            ins_state.lock().unwrap().update_page(page);
                        }
                        DisplayMessage::UpdateTime(date, time) => {
                            ins_state.lock().unwrap().update_date_time(date, time);
                        }
                        DisplayMessage::UpdateWeather(weather, city) => {
                            ins_state.lock().unwrap().update_weather(weather, city);
                        }
                        DisplayMessage::UpdateWifi(status, ip) => {
                            ins_state.lock().unwrap().update_wifi(status, ip);
                        }
                    }
                }
            })
            .unwrap();
        Ok(ins)
    }
    pub fn run(&mut self) {
        self.wifi_thread();
        self.sntp_thread();
        self.weather_thread();
        self.actions_thread();

        loop {
            let page = self.current_page.lock().unwrap().clone();
            self.show_page(page);
            thread::sleep(Duration::from_secs(1));
        }
    }
    fn wifi_thread(&mut self) {
        let wifi = self.wifi.clone();
        let display_tx = self.display_tx.clone();
        thread::Builder::new()
            .stack_size(9000)
            .spawn(move || {
                block_on(async {
                    wifi.lock().unwrap().connect().await.unwrap();
                    loop {
                        let (status, ip) = wifi.lock().unwrap().get_wifi_status().await.unwrap();

                        if status == WifiStatus::Connected {
                            info!("Wi-Fi connected! Breaking the loop.");
                            display_tx
                                .send(DisplayMessage::UpdateWifi(status, ip))
                                .unwrap();
                            break;
                        }
                        info!("Wi-Fi not connected. Retrying in 2 seconds...");
                        thread::sleep(Duration::from_secs(2));
                    }
                });
            })
            .unwrap();
    }
    fn sntp_thread(&mut self) {
        let display_tx = self.display_tx.clone();
        let state = self.state.clone();
        thread::Builder::new()
            .stack_size(9000)
            .spawn(move || {
                loop {
                    let (wifi_status, _) = state.lock().unwrap().get_wifi_status();
                    if wifi_status == WifiStatus::Connected {
                        break;
                    }
                    info!("时间同步模块: 等待 Wi-Fi 连接...");
                    thread::sleep(Duration::from_secs(1));
                }
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
                loop {
                    let now = Utc::now();
                    let shanghai_time = now.with_timezone(&Shanghai);
                    let date = shanghai_time.format("%Y-%m-%d").to_string();
                    let time = shanghai_time.format("%H:%M:%S").to_string();
                    display_tx
                        .send(DisplayMessage::UpdateTime(date, time))
                        .unwrap();
                    thread::sleep(Duration::from_secs(1));
                }
            })
            .unwrap();
    }
    fn weather_thread(&mut self) {
        let weather_client = self.weather_client.clone();
        let display_tx = self.display_tx.clone();
        let state = self.state.clone();

        thread::Builder::new()
            .stack_size(9000)
            .spawn(move || {
                block_on(async {
                    loop {
                        let (wifi_status, _) = state.lock().unwrap().get_wifi_status();
                        if wifi_status == WifiStatus::Connected {
                            break;
                        }
                        info!("天气模块: 等待 Wi-Fi 连接...");
                        thread::sleep(Duration::from_secs(1));
                    }

                    loop {
                        let weather_data = weather_client.lock().unwrap().fetch_weather().unwrap();
                        display_tx
                            .send(DisplayMessage::UpdateWeather(
                                weather_data.temp,
                                weather_data.city,
                            ))
                            .unwrap();
                        thread::sleep(Duration::from_secs(3600));
                    }
                })
            })
            .unwrap();
    }

    fn actions_thread(&mut self) {
        let display_tx = self.display_tx.clone();
        let current_page = self.current_page.clone();
        let left = self.left_button.clone();
        let right = self.right_button.clone();

        thread::Builder::new()
            .stack_size(9000)
            .spawn(move || {
                let display_tx_clone = display_tx.clone();
                let current_page_clone = current_page.clone();
                left.lock()
                    .unwrap()
                    .subscribe(move || {
                        display_tx_clone
                            .send(DisplayMessage::ShowPage(
                                current_page_clone.lock().unwrap().prev(),
                            ))
                            .unwrap();
                    })
                    .unwrap();
                let display_tx_clone = display_tx.clone();
                let current_page_clone = current_page.clone();
                right
                    .lock()
                    .unwrap()
                    .subscribe(move || {
                        display_tx_clone
                            .send(DisplayMessage::ShowPage(
                                current_page_clone.lock().unwrap().next(),
                            ))
                            .unwrap();
                    })
                    .unwrap();
                loop {
                    thread::sleep(Duration::from_secs(1));
                }
            })
            .unwrap();
    }

    fn show_page(&mut self, page: AppPage) {
        match page {
            AppPage::Home => self.render_home_page(),
            AppPage::WifiConfig => self.render_wifi_page(),
        }
    }

    fn render_home_page(&mut self) {
        let state = self.state.lock().unwrap().clone();
        self.display.lock().unwrap().update_home(state);
    }
    fn render_wifi_page(&mut self) {
        self.display.lock().unwrap().show_wifi_config();
    }
}
