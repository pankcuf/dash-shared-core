use std::ffi::CString;
use hashes::hex::{FromHex, ToHex};
use crate::bindings::keys::{key_bls_chaincode, key_bls_public_key, key_bls_with_bip32_seed_data, key_create_from_extended_public_key_data, key_extended_public_key_data, key_private_key_at_index_path, key_with_private_key};
use crate::chain::common::ChainType;
use crate::chain::derivation::{IIndexPath, IndexPath};
use crate::crypto::{byte_util::ConstDecodable, UInt256, UInt384};
use crate::ffi::IndexPathData;
use crate::keys::KeyKind;
use crate::types::opaque_key::OpaqueKey;

#[test]
fn test_keys() {
    let key_type = KeyKind::ECDSA;
    let extended_public_key_data_string = "3dc2e416b0f9fcfd74fe847ccd80f71cf961e7c4ddede29ce5e4b72a19ebccf2831ba2d803740b52e94ad4d526ff2b4340646b2ce1423545755b5825fabc741d1d74c155d7";
    let extended_public_key_data = Vec::from_hex(extended_public_key_data_string).unwrap();

    let extended_public_key = unsafe { key_create_from_extended_public_key_data(extended_public_key_data.as_ptr(), extended_public_key_data.len(), key_type) };
    println!("extended_public_key: {:?}", extended_public_key);
    let extended_public_key_data = unsafe { key_extended_public_key_data(extended_public_key) };
    println!("extended_public_key_data: {:?}", extended_public_key_data);
    let seed_bytes = unsafe { std::slice::from_raw_parts(extended_public_key_data.ptr, extended_public_key_data.len) };
    println!("extended_public_key_data: {:?}", seed_bytes.to_hex());
    assert_eq!(seed_bytes.to_hex(), extended_public_key_data_string);
}

#[test]
fn derive_bls() {
    let seed = Vec::from_hex("467c2dd58bbd29427fb3c5467eee339021a87b21309eeabfe9459d31eeb6eba9b2a1213c12a173118c84fd49e8b4bf9282272d67bf7b7b394b088eab53b438bc").unwrap();
    let index_path = IndexPath::index_path_with_index(0u32);
    let indexes = vec![
        UInt256::from_hex("0900000000000000000000000000000000000000000000000000000000000000").unwrap(),
        UInt256::from_hex("0500000000000000000000000000000000000000000000000000000000000000").unwrap(),
        UInt256::from_hex("0500000000000000000000000000000000000000000000000000000000000000").unwrap(),
        UInt256::from_hex("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
        UInt256::from_hex("0100000000000000000000000000000000000000000000000000000000000000").unwrap(),
    ];
    let hardened = vec![true, true, true, true, true];
    let indexes = vec![0];
    let index_path_data = IndexPathData  {
        indexes: indexes.as_ptr(),
        len: 1,
    };
    let indexes = b"09000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000";

    unsafe {
        let key = key_private_key_at_index_path(seed.as_ptr(), seed.len(), KeyKind::BLS, &index_path_data as *const IndexPathData, indexes.as_ptr(), hardened.as_ptr(), 5);
        let ext_pub = key_extended_public_key_data(key);
        println!("{:?}", ext_pub);
    }
}

#[test]
fn bls_chaincode() {
    let seed = [1, 50, 6, 244, 24, 199, 1, 25];
    let key_pair = unsafe { key_bls_with_bip32_seed_data(seed.as_ptr(), seed.len(), true) };
    let chain_code = unsafe { key_bls_chaincode(key_pair) };
    let chaincode = UInt256::from_const(chain_code.ptr).unwrap();
    assert_eq!(chaincode.0.to_hex(), "d8b12555b4cc5578951e4a7c80031e22019cc0dce168b3ed88115311b8feb1e3", "Testing BLS derivation chain code");

}

#[test]
fn bls_operator_key() {
    //return key_with_private_key([key UTF8String], keyType, chainType);
    let operator_key_hex_string = "0fc63f4e6d7572a6c33465525b5c3323f57036873dd37c98c393267c58b50533";
    let str = CString::new(operator_key_hex_string).unwrap();
    let key = unsafe { key_with_private_key(str.as_ptr(), KeyKind::BLS, ChainType::TestNet.into()) };

    unsafe {
        match key.as_ref() {
            Some(OpaqueKey::BLSLegacy(bls_key)) => {
                let pubkey = key_bls_public_key(*bls_key);
                let k = UInt384::from_const(pubkey.ptr).unwrap();
                let tst = UInt384::from_hex("139b654f0b1c031e1cf2b934c2d895178875cfe7c6a4f6758f02bc66eea7fc292d0040701acbe31f5e14a911cb061a2f").unwrap();
                assert_eq!(tst, k);
            },
            _ => {}
        }
    }
}
