use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;
use crate::chain::common::ChainType;
use crate::crypto::{byte_util::ConstDecodable, UInt160};
use crate::ffi::ByteArray;
use crate::types::opaque_key::AsCStringPtr;
use crate::util::address::address;
use crate::util::data_append::DataAppend;

/// # Safety
#[no_mangle]
pub extern "C" fn address_from_hash160(hash: *const u8, chain_type: ChainType) -> *mut c_char {
    let hash = UInt160::from_const(hash).unwrap_or(UInt160::MIN);
    let script_map = chain_type.script_map();
    address::from_hash160_for_script_map(&hash, &script_map)
        .to_c_string_ptr()
}

/// # Safety
#[no_mangle]
pub extern "C" fn address_with_script_pubkey(script: *const u8, script_len: usize, chain_type: ChainType) -> *mut c_char {
    let script = unsafe { slice::from_raw_parts(script, script_len) };
    let script_map = chain_type.script_map();
    address::with_script_pub_key(&script.to_vec(), &script_map)
        .to_c_string_ptr()
}

/// # Safety
#[no_mangle]
pub extern "C" fn address_with_script_sig(script: *const u8, script_len: usize, chain_type: ChainType) -> *mut c_char {
    let script = unsafe { slice::from_raw_parts(script, script_len) };
    let script_map = chain_type.script_map();
    address::with_script_sig(&script.to_vec(), &script_map)
        .to_c_string_ptr()
}

/// # Safety
#[no_mangle]
pub extern "C" fn script_pubkey_for_address(address: *const c_char, chain_type: ChainType) -> ByteArray {
    if address.is_null() {
        ByteArray::default()
    } else {
        let c_str = unsafe { CStr::from_ptr(address) };
        let script_map = chain_type.script_map();
        Vec::<u8>::script_pub_key_for_address(c_str.to_str().unwrap(), &script_map).into()
    }
}

/// # Safety
#[no_mangle]
pub extern "C" fn is_valid_dash_address_for_chain(address: *const c_char, chain_type: ChainType) -> bool {
    if address.is_null() {
        false
    } else {
        let c_str = unsafe { CStr::from_ptr(address) };
        let script_map = chain_type.script_map();
        address::is_valid_dash_address_for_script_map(c_str.to_str().unwrap(), &script_map)
    }
}
