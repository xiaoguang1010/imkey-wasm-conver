use std::path;

use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use ikc_webusb::webusb::connect;
use ikc_device::device_manager;

#[wasm_bindgen]
pub async fn connect_imkey() {
    connect().await;
}

#[wasm_bindgen]
pub async fn bind_check(file_path: String) -> String{
    device_manager::bind_check(&file_path).unwrap()
}