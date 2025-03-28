use esp_idf_hal::modem::Modem;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{
        AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi
    },
};
use heapless::String as HString;
use log::info;


// https://github.com/esp-rs/std-training/blob/main/common/lib/wifi/src/lib.rs

const SSID: &str = "ChinaUnicom-UG9SCX";
const PASSWORD: &str = "Buxingjie007";

// static WIFI_CONFIG: Mutex<Option<(HString<32>, HString<32>)>> = Mutex::new(None);

#[derive(PartialEq, Clone)]
pub enum WifiStatus {
    Disconnected,
    Connecting,
    Connected,
}
impl ToString for WifiStatus {
    fn to_string(&self) -> String {
        match self {
            WifiStatus::Connected => "Connected".to_string(),
            WifiStatus::Disconnected => "Disconnected".to_string(),
            WifiStatus::Connecting => "Connecting".to_string(),
        }
    }
}
pub struct WiFiManager<'d> {
    wifi: BlockingWifi<EspWifi<'d>>,
}

impl<'d> WiFiManager<'d> {
    pub fn new(modem: Modem) -> anyhow::Result<Self>  {
        let sys_loop = EspSystemEventLoop::take().unwrap();
        let nvs = EspDefaultNvsPartition::take().unwrap();

        let wifi = BlockingWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs)).unwrap(),
            sys_loop
        )?;
        Ok(Self { wifi })
    }
    // pub fn start_ap(&mut self) -> anyhow::Result<()> {
    //     let config = Configuration::AccessPoint(AccessPointConfiguration {
    //         ssid: HString::try_from("ESP32_Weather_App").unwrap(),
    //         password: HString::try_from("").unwrap(),
    //         channel: 1,
    //         auth_method: AuthMethod::None,
    //         max_connections: 4,
    //         ..Default::default()
    //     });
    //     self.wifi.set_configuration(&config)?;
    //     Ok(())
    // }
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        self.set_config()?;

        self.wifi.start()?;
        info!("Wifi started");

        self.wifi.connect()?;
        info!("Wifi connected");

        self.wifi.wait_netif_up()?;
        info!("Wifi netif up");

        Ok(())
    }
    fn set_config(&mut self) -> anyhow::Result<()> {
        let config = Configuration::Mixed(
            ClientConfiguration {
                ssid: HString::try_from(SSID).map_err(|_| anyhow::anyhow!("Invalid SSID"))?,
                password: HString::try_from(PASSWORD).map_err(|_| anyhow::anyhow!("Invalid password"))?,
                auth_method: AuthMethod::WPA2Personal,
                ..Default::default()
            },
            AccessPointConfiguration {
                ssid: HString::try_from("ESP32_Weather_App").unwrap(),
                password: HString::try_from("").unwrap(),
                channel: 1,
                auth_method: AuthMethod::None,
                max_connections: 4,
                ..Default::default()
            },
        );
        self.wifi.set_configuration(&config)?;
        Ok(())
    }
    pub async fn get_wifi_status(&mut self) -> anyhow::Result<(WifiStatus, String)>  {
        if self.wifi.is_connected()? {
            let ip = self.wifi.wifi().sta_netif().get_ip_info()?.ip;
            return Ok((WifiStatus::Connected, ip.to_string()));
        } else {
            return Ok((WifiStatus::Connecting, String::from("0.0.0.0")));
        }
    }
    
}

// pub fn start_wifi_ap(modem: Modem) -> BlockingWifi<EspWifi<'static>> {
//     let sys_loop = EspSystemEventLoop::take().unwrap();
//     let nvs = EspDefaultNvsPartition::take().unwrap();

//     let mut wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs)).unwrap();

//     let ap_config = Configuration::AccessPoint(AccessPointConfiguration {
//         ssid: HString::try_from("ESP32_Weather_App").unwrap(),
//         password: HString::try_from("").unwrap(),
//         channel: 1,
//         auth_method: AuthMethod::None,
//         max_connections: 4,
//         ..Default::default()
//     });

//     wifi.set_configuration(&ap_config).unwrap();
//     wifi.start().unwrap();

//     let blocking_wifi = BlockingWifi::wrap(wifi, sys_loop).unwrap();
//     blocking_wifi.wait_netif_up().unwrap();
//     info!("Wi-Fi AP 启动成功，SSID: ESP32_Setup");
//     blocking_wifi
// }

// /// **2. 连接到用户输入的 Wi-Fi**
// pub fn connect_to_wifi(
//     wifi: &mut BlockingWifi<EspWifi<'static>>,
//     ssid: &str,
//     password: &str,
//     display: &mut OLed,
// ) -> anyhow::Result<bool> {
//     info!("尝试连接 Wi-Fi: SSID = {}, Password = {}", ssid, password);
//     display.show_message("Connecting to Wi-Fi...");

//     let config = Configuration::Client(ClientConfiguration {
//         ssid: HString::try_from(ssid).unwrap(),
//         password: HString::try_from(password).unwrap(),
//         auth_method: AuthMethod::WPA2Personal,
//         ..Default::default()
//     });

//     wifi.set_configuration(&config).unwrap();
//     wifi.start().unwrap();

//     if wifi.connect().is_ok() {
//         info!("Wi-Fi 连接成功，ESP32 已联网！");
//         display.show_message("Wi-Fi Connected!");
//         Ok(true)
//         // esp_restart(); // **重启进入 STA 模式**
//     } else {
//         error!("Wi-Fi 连接失败，等待用户重新输入...");
//         display.show_message("Wi-Fi Failed!");
//         Ok(false)
//     }
// }
