use std::path;

use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use ikc_webusb::webusb::{connect, send_apdu};
use ikc_device::device_manager;

#[wasm_bindgen]
pub async fn connect_imkey() {
    connect().await;
}

#[wasm_bindgen]
pub async fn send_command(apdu: &str) ->String {
    send_apdu(apdu.to_string()).await.unwrap()
}

#[wasm_bindgen]
pub async fn bind_check(file_path: String) -> String{
    device_manager::bind_check(&file_path).await.unwrap()
}

#[wasm_bindgen]
pub async fn bind_acquire(file_path: String) -> String{
    device_manager::bind_acquire(&file_path).await.unwrap()
}