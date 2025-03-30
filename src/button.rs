use std::sync::{Arc, Mutex};

use esp_idf_hal::{
    gpio::{Input, InputPin, InterruptType, OutputPin, PinDriver, Pull},
    peripheral::Peripheral,
    sys::{gpio_pull_mode_t_GPIO_PULLUP_ONLY, gpio_set_pull_mode},
};
use log::info;

pub struct Button<P: InputPin> {
    btn: PinDriver<'static, P, Input>,
    callback: Option<Arc<Mutex<Box<dyn FnMut() + Send>>>>,
}
impl<P> Button<P>
where
    P: InputPin + OutputPin + 'static,
{
    pub fn new(pin: impl Peripheral<P = P> + 'static) -> anyhow::Result<Self> {
        let mut btn = PinDriver::input(pin)?;
        btn.set_pull(Pull::Up)?;
        btn.set_interrupt_type(InterruptType::NegEdge)?;

        Ok(Self {
            btn,
            callback: None,
        })
    }
    pub fn subscribe<F: FnMut() + Send + 'static>(&mut self, callback: F) -> anyhow::Result<()> {
        let callback = Arc::new(Mutex::new(Box::new(callback) as Box<dyn FnMut() + Send>));
        let callback_clone = callback.clone();
        unsafe {
            self.btn.subscribe(move || {
                log::info!("🔥 按钮中断触发！");
                if let Ok(mut cb) = callback_clone.lock() {
                    (*cb)();
                }
            })?;
        }
        log::info!("✅ 按钮订阅成功！");
        self.callback = Some(callback);
        Ok(())
    }
}
