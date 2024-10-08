use crate::aes::cbc::encrypt_pkcs7;
use crate::constants::SECP256K1_ENGINE;
use crate::error::CommonError;
use crate::hex::FromHex;
use crate::Result;
use bitcoin::hashes::{sha256, Hash};
use bitcoin::util::base58;
use bitcoin::util::bip32::{
    ChainCode, ChildNumber, Error as Bip32Error, ExtendedPubKey, Fingerprint,
};
use bitcoin::Network;
use byteorder::BigEndian;
use byteorder::ByteOrder;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{FromPrimitive, Num, Zero};
use regex::Regex;
use secp256k1::ecdsa::{RecoverableSignature, RecoveryId, Signature};
use secp256k1::{Message, PublicKey as PublicKey2, Secp256k1, SecretKey};

pub fn hex_to_bytes(value: &str) -> Result<Vec<u8>> {
    let ret_data;
    if value.to_lowercase().starts_with("0x") {
        ret_data = hex::decode(&value[2..value.len()])?
    } else {
        ret_data = hex::decode(value)?
    }
    Ok(ret_data)
}

pub fn sha256_hash(data: &[u8]) -> Vec<u8> {
    sha256::Hash::hash(data).into_inner().to_vec()
}

pub fn secp256k1_sign(private_key: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    //calc twice sha256 hash
    let message_hash = sha256_hash(sha256_hash(message).as_ref());
    //generator SecretKey obj
    let secret_key = SecretKey::from_slice(private_key)?;
    //generator Message obj
    let message_data = Message::from_slice(message_hash.as_ref())?;
    let secp = Secp256k1::new();
    //sign data
    Ok(secp
        .sign_ecdsa(&message_data, &secret_key)
        .serialize_der()
        .to_vec())
}

/**
sign verify
*/
pub fn secp256k1_sign_verify(public: &[u8], signed: &[u8], message: &[u8]) -> Result<bool> {
    let secp = Secp256k1::new();
    //build public
    let public_obj = PublicKey2::from_slice(public)?;
    //build message
    let hash_result = sha256_hash(message);
    let message_obj = Message::from_slice(hash_result.as_ref())?;
    //build signature obj
    let mut sig_obj = Signature::from_der(signed)?;
    sig_obj.normalize_s();
    //verify
    Ok(secp
        .verify_ecdsa(&message_obj, &sig_obj, &public_obj)
        .is_ok())
}

pub fn bigint_to_byte_vec(val: u64) -> Vec<u8> {
    let mut return_data = BigInt::from(val).to_signed_bytes_be();
    while return_data.len() < 8 {
        return_data.insert(0, 0x00);
    }
    return_data
}

pub fn uncompress_pubkey_2_compress(uncomprs_pubkey: &str) -> String {
    let x = &uncomprs_pubkey[2..66];
    let y = &uncomprs_pubkey[66..130];
    let y_bint = BigInt::from_str_radix(&y, 16).unwrap();
    let two_bint = BigInt::from_i64(2).unwrap();

    let (_d, m) = y_bint.div_mod_floor(&two_bint);
    return if m.is_zero() {
        ("02".to_owned() + x).to_lowercase()
    } else {
        ("03".to_owned() + x).to_lowercase()
    };
}

pub fn is_valid_hex(input: &str) -> bool {
    let mut value = input;

    if input.starts_with("0x") || input.starts_with("0X") {
        value = input[2..].as_ref();
    };

    if value.len() == 0 || value.len() % 2 != 0 {
        return false;
    }

    let regex = Regex::new(r"^[0-9a-fA-F]+$").unwrap();
    regex.is_match(value.as_ref())
}

pub fn retrieve_recid(msg: &[u8], sign_compact: &[u8], pubkey: &Vec<u8>) -> Result<RecoveryId> {
    let secp_context = secp256k1::Secp256k1::new();

    let mut recid_final = -1i32;
    for i in 0..3 {
        let rec_id = RecoveryId::from_i32(i as i32)?;
        let sig = RecoverableSignature::from_compact(sign_compact, rec_id)?;
        let msg_to_sign = Message::from_slice(msg)?;

        if let Ok(rec_pubkey) = secp_context.recover_ecdsa(&msg_to_sign, &sig) {
            let rec_pubkey_raw = rec_pubkey.serialize_uncompressed();
            if rec_pubkey_raw.to_vec() == *pubkey {
                recid_final = i;
                break;
            }
        } else {
            continue;
        }
    }

    let rec_id = RecoveryId::from_i32(recid_final)?;
    Ok(rec_id)
}

pub fn to_ss58check_with_version(extended_key: ExtendedPubKey, version: &[u8]) -> String {
    let mut ret = [0; 78];
    // let extended_key = self.0;
    ret[0..4].copy_from_slice(version);
    ret[4] = extended_key.depth;
    ret[5..9].copy_from_slice(&extended_key.parent_fingerprint[..]);

    BigEndian::write_u32(&mut ret[9..13], u32::from(extended_key.child_number));

    ret[13..45].copy_from_slice(&extended_key.chain_code[..]);
    ret[45..78].copy_from_slice(&extended_key.public_key.serialize()[..]);
    base58::check_encode_slice(&ret[..])
}

pub fn from_ss58check_with_version(s: &str) -> Result<(ExtendedPubKey, Vec<u8>)> {
    let data = base58::from_check(s)?;

    if data.len() != 78 {
        return Err(CommonError::InvalidBase58.into());
    }
    let cn_int: u32 = BigEndian::read_u32(&data[9..13]);
    let child_number: ChildNumber = ChildNumber::from(cn_int);

    let epk = ExtendedPubKey {
        network: Network::Bitcoin,
        depth: data[4],
        parent_fingerprint: Fingerprint::from(&data[5..9]),
        child_number,
        chain_code: ChainCode::from(&data[13..45]),
        public_key: secp256k1::PublicKey::from_slice(&data[45..78])?,
    };

    let mut network = [0; 4];
    network.copy_from_slice(&data[0..4]);
    Ok((epk, network.to_vec()))
}

pub fn extended_pub_key_derive(
    extended_pub_key: &ExtendedPubKey,
    path: &str,
) -> Result<ExtendedPubKey> {
    let mut parts = path.split('/').peekable();
    if *parts.peek().unwrap() == "m" {
        parts.next();
    }

    let children_nums = parts
        .map(str::parse)
        .collect::<std::result::Result<Vec<ChildNumber>, Bip32Error>>()?;

    let child_key = extended_pub_key.derive_pub(&SECP256K1_ENGINE, &children_nums)?;

    Ok(child_key)
}

pub fn get_xpub_prefix(network: &str) -> Vec<u8> {
    if network == "MAINNET" {
        hex_to_bytes("0488b21e").unwrap()
    } else {
        hex_to_bytes("043587cf").unwrap()
    }
}

pub fn encrypt_xpub(xpub: &str) -> Result<String> {
    let key = crate::XPUB_COMMON_KEY_128.read();
    let iv = crate::XPUB_COMMON_IV.read();
    let key_bytes = hex::decode(&*key)?;
    let iv_bytes = hex::decode(&*iv)?;
    let encrypted = encrypt_pkcs7(xpub.as_bytes(), &key_bytes, &iv_bytes)?;
    anyhow::Ok(base64::encode(encrypted))
}

pub fn network_convert(network: &str) -> Network {
    match network.to_uppercase().as_str() {
        "MAINNET" => Network::Bitcoin,
        "TESTNET" => Network::Testnet,
        _ => Network::Testnet,
    }
}

pub fn utf8_or_hex_to_bytes(value: &str) -> Result<Vec<u8>> {
    if value.to_lowercase().starts_with("0x") {
        let ret = FromHex::from_0x_hex(value);
        if ret.is_err() {
            Ok(value.as_bytes().to_vec())
        } else {
            ret
        }
    } else {
        Ok(value.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::utility;
    use crate::utility::{
        bigint_to_byte_vec, retrieve_recid, secp256k1_sign, secp256k1_sign_verify, sha256_hash,
        uncompress_pubkey_2_compress,
    };
    use crate::utility::{is_valid_hex, network_convert};
    use bitcoin::Network;
    use hex::FromHex;

    #[test]
    fn hex_to_bytes_test() {
        assert_eq!(
            vec![0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72],
            utility::hex_to_bytes("666f6f626172").unwrap_or_default(),
        );
        assert_eq!(
            vec![0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72],
            utility::hex_to_bytes("0x666f6f626172").unwrap_or_default()
        );
    }

    #[test]
    fn sha256_hash_test() {
        let data = Vec::from_hex("11223344556677889900").unwrap();
        assert_eq!(
            hex::encode(utility::sha256_hash(&data)),
            "6fa6810c930ba44a979a1bdb029f56cc608eafa043cea7e1ed21050d7456b5d3",
        );
    }

    #[test]
    fn secp256k1_sign_and_verify_test() {
        let private_key =
            hex::decode("631e12677ef30f9b1a055b16bd9bf2d2a4f0795a484a9dc49683a05dc8328613")
                .unwrap();
        let public_key = hex::decode("04327a42790a3158d58bd68ee5763330b85b080c306534bf4d3c8fc711023db3090f302f9f7c8a2fc8ae81bfa22c9484b76326b1b2971eb7f7afea15cfd1996413").unwrap();
        let data = hex::decode("11223344556677889900").unwrap();
        let sign_result =
            secp256k1_sign(private_key.as_slice(), data.as_slice()).unwrap_or_default();
        assert_eq!(hex::encode(sign_result.clone()), "304402201b4197c869af37cea51e9ef34525c19f5e588ac5236b9e79dec3cdb1681498090220105d33d1217f76abd9a53ecab8beeb8de834ef5a5205a33288bb5bb4c3057742");
        let data = sha256_hash(data.as_slice());
        assert!(secp256k1_sign_verify(
            public_key.as_slice(),
            sign_result.as_slice(),
            data.as_slice()
        )
        .ok()
        .unwrap())
    }

    #[test]
    fn bigint_to_byte_vec_test() {
        assert_eq!(
            hex::encode(bigint_to_byte_vec(1111111111111111111)),
            "0f6b75ab2bc471c7"
        );
        assert_eq!(hex::encode(bigint_to_byte_vec(111111)), "000000000001b207");
    }

    #[test]
    fn uncompress_pubkey_2_compress_test() {
        let public_key_03 = "04327a42790a3158d58bd68ee5763330b85b080c306534bf4d3c8fc711023db3090f302f9f7c8a2fc8ae81bfa22c9484b76326b1b2971eb7f7afea15cfd1996413";
        //        privatekey:631e12677ef30f9b1a055b16bd9bf2d2a4f0795a484a9dc49683a05dc8328613
        assert_eq!(
            uncompress_pubkey_2_compress(public_key_03),
            "03327a42790a3158d58bd68ee5763330b85b080c306534bf4d3c8fc711023db309"
        );
        let public_key_02 = "04c390b4116d0f971c8f641f24346bd38377a22adb1426d27278e4cbb3e49e89986399de811e617faad763825e80af484e7fe16387929507baeaf633b03ce21f7e";
        //      privatekey:ef715e7b3509b87c89db3e173515eebfe1936f6b1cf9fb8c4ba15e82f9034f07
        assert_eq!(
            uncompress_pubkey_2_compress(public_key_02),
            "02c390b4116d0f971c8f641f24346bd38377a22adb1426d27278e4cbb3e49e8998"
        );
    }

    #[test]
    fn retrieve_recid_test() {
        let msg = hex::decode("b998c88d8478e87e6dee727adecec067a3201da03ec8f8e8861c946559be6355")
            .unwrap();
        let sign_compact = hex::decode("73bcac6f18a619f047693afb17c1574fd22bb65d184888c13b5f2715304304b15919cbb66a8ae244ed8ac6dddbde8cc381a828961cfbad070d6c368941516ec5").unwrap();
        let pubkey = hex::decode("04aaf80e479aac0813b17950c390a16438b307aee9a814689d6706be4fb4a4e30a4d2a7f75ef43344fa80580b5b1fbf9f233c378d99d5adb5cac9ae86f562803e1").unwrap();
        assert!(retrieve_recid(msg.as_slice(), sign_compact.as_slice(), &pubkey).is_ok());
    }

    #[test]
    fn valid_hex_test() {
        let input1 = "666f6f626172";
        assert_eq!(is_valid_hex(input1), true,);
        let input1 = "Hello imKey";
        assert_eq!(is_valid_hex(input1), false,);
        let input1 = "+9qMMqskYEMjYyy0YnHtjXDRR62cpMDcoWFSA/";
        assert_eq!(is_valid_hex(input1), false,);
        let input1 = "R4Sod77GQJ1uxcQ8DH8giotqKTTiOQpt4ukd84VdKwdHUyeajiGhi6AUPSkyjsu1i0mXL";
        assert_eq!(is_valid_hex(input1), false,);
        let input1 = "d8549e61c7c5fa21315f86c9b6bd7f2efd0e7aef8647c467679a8cfefff9996329c47a6509487d2ca4d0408ff8f683449d438a0491c8bf11d54fa3b2d6af9849c808ddd1b67e84e8029edc5df4dc485e41fb1de2cbdd3143f204fb4cb58ca9155a194e465dcc7fbcb9fc729147efba62fbba2ba0356a97dcf816ab1fa8f4ebedf8506fa2920ac1f92bf2d3709b3b1cbb57124db22beb866a3b42e6286a6f6b4bcab27ec9cf7403db78f43c3d957de89d5fb23b3d9bcb23c0f62d9064da159714";
        assert_eq!(is_valid_hex(input1), true,);
    }

    #[test]
    fn test_network_convert() {
        let network = network_convert("MAINNET");
        assert_eq!(network, Network::Bitcoin);
        let network = network_convert("TESTNET");
        assert_eq!(network, Network::Testnet);
        let network = network_convert("mainnet");
        assert_eq!(network, Network::Bitcoin);
        let network = network_convert("ERRORNET");
        assert_eq!(network, Network::Testnet);
    }
}
