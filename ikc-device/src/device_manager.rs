#[cfg(target_arch = "wasm32")]
use ikc_webusb::webusb::send_apdu;
use ikc_common::apdu::{Apdu, ApduCheck};
use crate::Result;
use crate::device_binding::DeviceManage;
use futures::executor::block_on;

pub async fn select_isd() -> Result<String> {
    let res = send_apdu("00A4040000".to_string()).await?;
    ApduCheck::check_response(res.as_str())?;
    Ok(res)
}

pub async fn get_se_id() -> Result<String> {
    select_isd().await?;
    let res = send_apdu("80CB800005DFFF028101".to_string()).await?;
    ApduCheck::check_response(res.as_str())?;
    Ok(String::from(&res[0..res.len() - 4]))
}

pub async fn get_sn() -> Result<String> {
    select_isd().await?;
    let res = send_apdu("80CA004400".to_string()).await?;
    ApduCheck::check_response(res.as_str())?;
    let hex_decode = hex::decode(String::from(&res[0..res.len() - 4]));
    match hex_decode {
        Ok(sn) => Ok(String::from_utf8(sn).unwrap()),
        Err(error) => Err(error.into()),
    }
}

// pub fn get_cert() -> Result<String> {
//     select_isd()?;
//     let res = block_on(send_apdu("80CABF2106A6048302151800".to_string()))?;
//     ApduCheck::check_response(&res)?;
//     Ok(res.chars().take(res.len() - 4).collect())
// }

pub async fn bind_check(file_path: &str) -> Result<String> {
    DeviceManage::bind_check(&file_path.to_string()).await
}
pub async fn bind_display_code() -> Result<()> {
    DeviceManage::display_bind_code().await
}
pub async fn bind_acquire(bind_code: &str) -> Result<String> {
    DeviceManage::bind_acquire(&bind_code.to_string()).await
}

