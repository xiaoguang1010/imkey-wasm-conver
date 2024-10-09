use super::key_manager::KeyManager;
use crate::error::{BindError, ImkeyError};
use crate::Result;
use crate::{device_manager, TsmService};
use ikc_common::aes::cbc::encrypt_pkcs7;
use ikc_common::apdu::{Apdu, ApduCheck, ImkApdu};
use ikc_common::constants::{
    BIND_RESULT_ERROR, BIND_RESULT_SUCCESS, BIND_STATUS_BOUND_OTHER, BIND_STATUS_BOUND_THIS,
    BIND_STATUS_UNBOUND, IMK_AID, TIMEOUT_LONG,
};
use ikc_common::utility::sha256_hash;
#[cfg(target_arch = "wasm32")]
use ikc_webusb::webusb::{send_apdu};
use parking_lot::Mutex;
use rand::rngs::OsRng;
use regex::Regex;
use rsa::{BigUint, PaddingScheme, PublicKey as RSAPublic, RsaPublicKey};
use secp256k1::{ecdh, PublicKey, SecretKey};
use sha1::Sha1;
use std::collections::HashMap;
use futures::executor::block_on;

lazy_static! {
    pub static ref KEY_MANAGER: Mutex<KeyManager> = Mutex::new(KeyManager::new());
    static ref BIND_STATUS_MAP: HashMap<&'static str, &'static str> = {
        let mut bind_status_mapping = HashMap::new();
        bind_status_mapping.insert(BIND_STATUS_UNBOUND, "unbound");
        bind_status_mapping.insert(BIND_STATUS_BOUND_THIS, "bound_this");
        bind_status_mapping.insert(BIND_STATUS_BOUND_OTHER, "bound_other");
        bind_status_mapping.insert(BIND_RESULT_SUCCESS, "success");
        bind_status_mapping.insert(BIND_RESULT_ERROR, "authcode_error");
        bind_status_mapping
    };
}

pub struct DeviceManage {}

impl DeviceManage {
    pub fn bind_check(file_path: &String) -> Result<String> {
        
        //get seid
        let seid = device_manager::get_se_id()?;
        //get SN number
        let sn = device_manager::get_sn()?;
        //Calculate encryption key
        let mut key_manager_obj = KEY_MANAGER.lock();
        key_manager_obj.gen_encrypt_key(&seid, &sn);

        //Get the ciphertext of the local key file
        let ciphertext = KeyManager::get_key_file_data(file_path, &seid)?;
        let mut key_flag = false;
        if !ciphertext.is_empty() {
            //Decrypt and parse the ciphertext
            key_flag = !key_manager_obj.decrypt_keys(&ciphertext)?;
        }

        //If the key file does not exist or is empty then regenerate
        if ciphertext.is_empty() || key_flag {
            key_manager_obj.gen_local_keys()?;
            key_flag = true;
        }

        //gen bindchec apdu
        let bind_check_apdu = ImkApdu::bind_check(&key_manager_obj.pub_key);
        //send bindcheck command and get return data
        select_imk_applet()?;
        let bind_check_apdu_resp_data = block_on(send_apdu(bind_check_apdu))?;
        ApduCheck::check_response(bind_check_apdu_resp_data.as_str())?;

        let status = String::from(&bind_check_apdu_resp_data[..2]);
        let se_pub_key_cert: String =
            String::from(&bind_check_apdu_resp_data[2..(bind_check_apdu_resp_data.len() - 4)]);

        if status.eq(BIND_STATUS_UNBOUND) || status.eq(BIND_STATUS_BOUND_OTHER) {
            // //check se cert
            // DeviceCertCheckRequest::build_request_data(seid.clone(), sn, se_pub_key_cert.clone())
            //     .send_message()?;

            //get se public key
            key_manager_obj.se_pub_key = hex::decode(get_se_pubkey(se_pub_key_cert)?)?;

            //calc the session key
            let pk2 = PublicKey::from_slice(key_manager_obj.se_pub_key.as_slice())?;
            let sk1 = SecretKey::from_slice(key_manager_obj.pri_key.as_slice())?;
            let shared_secret = ecdh::shared_secret_point(&pk2, &sk1);
            let sha1_result = Sha1::from(&shared_secret[..32]).digest().bytes();

            //set the session key
            key_manager_obj.session_key = sha1_result[..16].to_vec();

            //Save the ciphertext to a local file
            if key_flag {
                let ciphertext = key_manager_obj.encrypt_data()?;
                KeyManager::save_keys_to_local_file(&ciphertext, file_path, &seid)?;
            }
        }
        Ok(BIND_STATUS_MAP.get(status.as_str()).unwrap().to_string())
    }

    pub fn bind_acquire(binding_code: &String) -> Result<String> {
        let temp_binding_code = binding_code.to_uppercase();
        let binding_code_bytes = temp_binding_code.as_bytes();
        //check auth code
        let bind_code_verify_regex = Regex::new(r"^[A-HJ-NP-Z2-9]{8}$").unwrap();
        if !bind_code_verify_regex.is_match(temp_binding_code.as_ref()) {
            return Err(BindError::ImkeySdkIllegalArgument.into());
        }
        //encryption auth code
        let auth_code_ciphertext = auth_code_encrypt(&temp_binding_code)?;

        //save auth Code cipher
        let seid = device_manager::get_se_id()?;
        // AuthCodeStorageRequest::build_request_data(seid, auth_code_ciphertext).send_message()?;

        let key_manager_obj = KEY_MANAGER.lock();
        //select IMK applet
        select_imk_applet()?;
        //calc HASH
        let mut data: Vec<u8> = vec![];
        data.extend(binding_code_bytes);
        data.extend(&key_manager_obj.pub_key);
        data.extend(&key_manager_obj.se_pub_key);
        let data_hash = sha256_hash(data.as_slice());

        //encryption hash value by session key
        let ciphertext = encrypt_pkcs7(
            &data_hash.as_ref(),
            &key_manager_obj.session_key,
            &gen_iv(&temp_binding_code),
        )?;
        //gen identityVerify command
        let mut apdu_data = vec![];
        apdu_data.extend(&key_manager_obj.pub_key);
        apdu_data.extend(ciphertext);
        let identity_verify_apdu = ImkApdu::identity_verify(&apdu_data);
        std::mem::drop(key_manager_obj);
        //send command to device
        // let bind_result = send_apdu_timeout(identity_verify_apdu, TIMEOUT_LONG * 2)?;
        let bind_result = block_on(send_apdu(identity_verify_apdu))?;
        ApduCheck::check_response(&bind_result)?;
        let result_code = &bind_result[..bind_result.len() - 4];

        match result_code {
            BIND_RESULT_ERROR => Err(BindError::ImkeyAuthcodeError.into()),
            _ => Ok(BIND_STATUS_MAP.get(result_code).unwrap().to_string()),
        }
    }

    pub fn display_bind_code() -> Result<()> {
        select_imk_applet()?;
        let gen_auth_code_ret_data = block_on(send_apdu(ImkApdu::generate_auth_code()))?;
        ApduCheck::check_response(&gen_auth_code_ret_data)
    }
}

#[cfg(target_arch = "wasm32")]
fn select_imk_applet() -> Result<()> {
    let apdu_response = block_on(send_apdu(Apdu::select_applet(IMK_AID)))?;
    ApduCheck::check_response(apdu_response.as_str())
}

/**
generator iv
*/
fn gen_iv(auth_code: &String) -> [u8; 16] {
    let salt_bytes = sha256_hash("bindingCode".as_bytes());
    let auth_code_hash = sha256_hash(auth_code.as_bytes());
    let mut result = [0u8; 32];
    for (index, value) in auth_code_hash.iter().enumerate() {
        result[index] = value ^ salt_bytes.get(index).unwrap();
    }
    let mut return_data = [0u8; 16];
    return_data.copy_from_slice(&result[..16]);
    return_data
}

/**
encrypt auth code
*/
fn auth_code_encrypt(auth_code: &String) -> Result<String> {
    let n = hex::decode("C6627A6F0485B33DDC1CA7E062C64E8841133B9246A41F40D0767BAE44EAB2EF453D008FFB07B8D9FDFCD21882487ECC4DA933C97E494242ADA3CE02C5A05189AA49410E771A66E8100E43CB1AF6CC610B59EE4EBB236FF38C62AD7B1D11DFBD4E054D19E3349391A31F5E89CA721292B7380295745D8968CC5C2D223AC6750BB0ACA27773687E9CD76065E47F42F4AE005459BCE5746BD760646A5BD119BA3469A935F48EB898CBAB72CB394C3FEC9E41635EAE954107A17AC7B8C6321D8F1755AD3915A9D2398DB268A3F642CEE9CBE9F82ECD5AD64EBEDDDE66601DC2B891E2FEDDF72DAF627FA8FA16F7C640DB661BE15DCB4274D9576D98DBEB20C25309");
    let e = hex::decode("010001");
    let u32_vec_n = BigUint::from_bytes_be(&n.unwrap());
    let u32_vec_e = BigUint::from_bytes_be(&e.unwrap());
    let rsa_pub_key = RsaPublicKey::new(u32_vec_n, u32_vec_e)?;
    let mut rng = OsRng;
    // let mut rng = OsRng;
    let enc_data = rsa_pub_key.encrypt(
        &mut rng,
        PaddingScheme::PKCS1v15Encrypt,
        auth_code.as_bytes(),
    )?;
    Ok(hex::encode_upper(enc_data))
}

fn get_se_pubkey(se_pubkey_cert: String) -> Result<String> {
    let index;
    if se_pubkey_cert.contains("7F4947B041") {
        index = se_pubkey_cert
            .find("7F4947B041")
            .expect("parsing_se_cert_error");
    } else if se_pubkey_cert.contains("7F4946B041") {
        index = se_pubkey_cert
            .find("7F4946B041")
            .expect("parsing_se_cert_error");
    } else {
        return Err(ImkeyError::ImkeySeCertInvalid.into());
    }

    Ok(se_pubkey_cert[index + 10..index + 130 + 10].to_string())
}

