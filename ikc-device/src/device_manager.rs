#[cfg(target_arch = "wasm32")]
use ikc_webusb::send_apdu;

#[cfg(target_arch = "wasm32")]
pub fn select_isd() -> Result<String> {
    let res = send_apdu("00A4040000".to_string())?;
    ApduCheck::check_response(res.as_str())?;
    Ok(res)
}

#[cfg(target_arch = "wasm32")]
pub fn get_se_id() -> Result<String> {
    select_isd()?;
    let res = send_apdu("80CB800005DFFF028101".to_string())?;
    ApduCheck::check_response(res.as_str())?;
    Ok(String::from(&res[0..res.len() - 4]))
}

#[cfg(target_arch = "wasm32")]
pub fn get_sn() -> Result<String> {
    select_isd()?;
    let res = send_apdu("80CA004400".to_string())?;
    ApduCheck::check_response(res.as_str())?;
    let hex_decode = hex::decode(String::from(&res[0..res.len() - 4]));
    match hex_decode {
        Ok(sn) => Ok(String::from_utf8(sn).unwrap()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get_cert() -> Result<String> {
    select_isd()?;
    let res = send_apdu("80CABF2106A6048302151800".to_string())?;
    ApduCheck::check_response(&res)?;
    Ok(res.chars().take(res.len() - 4).collect())
}

#[cfg(target_arch = "wasm32")]
pub fn bind_check(file_path: &str) -> Result<String> {
    DeviceManage::bind_check(&file_path.to_string())
}
#[cfg(target_arch = "wasm32")]
pub fn bind_display_code() -> Result<()> {
    DeviceManage::display_bind_code()
}
#[cfg(target_arch = "wasm32")]
pub fn bind_acquire(bind_code: &str) -> Result<String> {
    DeviceManage::bind_acquire(&bind_code.to_string())
}

