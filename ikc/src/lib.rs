use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use ikc_webusb::webusb::connect;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn connect_imkey() {
    connect().await;
}