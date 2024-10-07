use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
mod webusb;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn connect() {
    webusb::connect().await;
}