use embedded_svc::http::client::Client;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize, Debug)]
pub struct Temperature {
    pub data: Data,
}
#[derive(Deserialize, Debug)]
pub struct Data {
    pub wendu: String,
}

const WEATHER_URL: &str = "http://t.weather.sojson.com/api/weather/city/101120201";

pub struct WeatherClient;

impl WeatherClient {
    pub fn new() -> Self {
        Self
    }
    pub fn fetch_weather(&self) -> anyhow::Result<String>  {
        let config = Configuration {
            timeout: Some(Duration::from_secs(5)),
            use_global_ca_store: true,
            ..Default::default()
        };
        let mut client = Client::wrap(EspHttpConnection::new(&config)?);

        let mut response = client.get(WEATHER_URL)?.submit()?;

        let mut buffer = [0; 4096];
        let bytes_read = response.read(&mut buffer)?;
        let json_str = String::from_utf8_lossy(&buffer[..bytes_read]);
        let parsed: Temperature = serde_json::from_str(&json_str)?;
        let temp = parsed.data.wendu;
        Ok(format!("{} ’C", temp))
    }
}
