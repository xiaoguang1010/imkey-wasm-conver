use std::path;

use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use ikc_webusb::webusb::{connect, send_apdu};
use ikc_device::device_manager;
use coin_bitcoin::address::BtcAddress;
use ikc_common::utility::network_convert;

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

#[wasm_bindgen]
pub async fn bind_display_code(){
    device_manager::bind_display_code().await.expect("display_bind_code_error");
}

#[wasm_bindgen]
pub async fn get_address(seg_wit: String, network: String, path: String) -> String{
    let network = network_convert(&network);
    let main_address = match seg_wit.as_str() {
        "P2WPKH" => BtcAddress::p2shwpkh(network, format!("{}/0/0", path).as_str()).await.unwrap(),
        "VERSION_0" => BtcAddress::p2wpkh(network, format!("{}/0/0", path).as_str()).await.unwrap(),
        "VERSION_1" => BtcAddress::p2tr(network, format!("{}/0/0", path).as_str()).await.unwrap(),
        _ => BtcAddress::p2pkh(network, format!("{}/0/0", path).as_str()).await.unwrap(),
    };
    main_address
}