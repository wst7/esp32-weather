use std::sync::{Arc, Mutex};

use esp_idf_svc::http::{
    server::{Configuration, EspHttpConnection, EspHttpServer, Request},
    Method,
};
use log::info;

use crate::{config::parse_form_data, oled::OLed};

/// 共享 Wi-Fi 配置信息
#[derive(Clone)]
pub struct WifiConfig {
    pub ssid: String,
    pub password: String,
}
pub fn start_http_server(
    server: Arc<Mutex<EspHttpServer<'static>>>,
    wifi_config: Arc<Mutex<Option<WifiConfig>>>,
    display: Arc<Mutex<OLed<'static>>>,
    ip: String,
) {
    let mut server = server.lock().unwrap();
    
    // let mut server: EspHttpServer<'_> = EspHttpServer::new(&Configuration::default()).unwrap();
    let display_clone = display.clone();
    server
        .fn_handler(
            "/",
            embedded_svc::http::Method::Get,
            index_handler(display_clone),
        )
        .unwrap();

    // 处理 Wi-Fi 连接请求
    let wifi_config_clone = wifi_config.clone();
    let display_clone = display.clone();
    server
        .fn_handler(
            "/connect",
            Method::Post,
            connect_handler(wifi_config_clone, display_clone),
        )
        .unwrap();
    display.lock().unwrap().clear();
    display.lock().unwrap().draw_qr_code(format!("http://{}", ip).as_str());
}

fn index_handler(
    display: Arc<Mutex<OLed<'static>>>,
) -> impl Fn(Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
    move |req: Request<&mut EspHttpConnection>| -> anyhow::Result<()> {
        let html = r#"
          <html>
          <body>
              <h2>Wi-Fi 设置</h2>
              <form action="/connect" method="post">
                  SSID: <input type="text" name="ssid"><br>
                  密码: <input type="password" name="password"><br>
                  <input type="submit" value="连接">
              </form>
          </body>
          </html>
          "#;
        let mut response = req.into_ok_response()?;
        response.write(html.as_bytes())?;
        display.lock().unwrap().clear();
        display.lock().unwrap().show_message("");
        anyhow::Ok(())
    }
}

fn connect_handler(
    wifi: Arc<Mutex<Option<WifiConfig>>>,
    display: Arc<Mutex<OLed<'static>>>,
) -> impl Fn(Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
    return move |mut req: Request<&mut EspHttpConnection>| -> anyhow::Result<()> {
        let mut buffer = [0u8; 256];
        let len = req.read(&mut buffer).unwrap_or(0);
        let body = String::from_utf8_lossy(&buffer[..len]);

        if let Some((ssid, password)) = parse_form_data(&body) {
            *wifi.lock().unwrap() = Some(WifiConfig { ssid, password });
            info!("收到 Wi-Fi 连接信息");

            let mut display = display.lock().unwrap();
            display.show_message("Wi-Fi Configured!");
        }

        req.into_ok_response()?
            .write(b"Wait ESP32 connecting Wi-Fi...")?;
        anyhow::Ok(())
    };
}
