


mod app;
mod display;
mod weather;
mod wifi;
mod button;
mod state;

fn main() {
    // Initialize the IDF stuff and logger
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut app = app::App::new().unwrap();
    app.run();
}
