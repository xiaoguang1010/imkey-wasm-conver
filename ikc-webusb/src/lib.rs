extern crate web_sys;
use wasm_bindgen::prelude::*;
// #[cfg(target_arch = "wasm32")]
pub mod webusb;
use core::result;
pub type Result<T> = result::Result<T, anyhow::Error>;


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn connect() {
    webusb::connect().await;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn send_apdu(apdu: &str) ->String {
    webusb::send_apdu(apdu).await.unwrap()
}

