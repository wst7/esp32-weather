


mod app;
mod display;
mod weather;
mod wifi;

fn main() {
    // Initialize the IDF stuff and logger
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let app = app::App::new().unwrap();
    app.start();
}
