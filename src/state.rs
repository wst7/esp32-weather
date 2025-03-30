use crate::{app::AppPage, wifi::WifiStatus};

#[derive(Clone)]
pub struct State {
    pub page: AppPage,
    pub weather: String,
    pub wifi_status: WifiStatus,
    pub wifi_ip: String,
    pub date: String,
    pub time: String,
    pub city: String,
}
impl State {
    pub fn new() -> Self {
        Self {
            page: AppPage::Home,
            weather: String::from("--"),
            wifi_status: WifiStatus::Disconnected,
            wifi_ip: String::from("0.0.0.0"),
            date: String::from("0000-00-00"),
            time: String::from("00:00:00"),
            city: String::from(""),
        }
    }
    pub fn update_page(&mut self, page: AppPage) {
        self.page = page;
    }
    pub fn update_wifi(&mut self, status: WifiStatus, ip: String) {
        self.wifi_status = status;
        self.wifi_ip = ip;
    }
    pub fn update_weather(&mut self, weather: String, city: String) {
        self.weather = weather;
        self.city = city;
    }
    pub fn update_date_time(&mut self, date: String, time: String) {
        self.date = date;
        self.time = time;
    }

    pub fn get_current_page(&self) -> &AppPage {
        &self.page
    }
    pub fn get_wifi_status(&self) -> (WifiStatus, String) {
        (self.wifi_status.clone(), self.wifi_ip.clone())
    }
}