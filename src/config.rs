


/// 解析表单数据
pub fn parse_form_data(body: &str) -> Option<(String, String)> {
  let mut ssid = None;
  let mut password = None;

  for pair in body.split('&') {
      let mut kv = pair.split('=');
      match (kv.next(), kv.next()) {
          (Some("ssid"), Some(value)) => ssid = Some(value.replace("+", " ")),
          (Some("password"), Some(value)) => password = Some(value.replace("+", " ")),
          _ => {}
      }
  }

  match (ssid, password) {
      (Some(s), Some(p)) => Some((s, p)),
      _ => None,
  }
}