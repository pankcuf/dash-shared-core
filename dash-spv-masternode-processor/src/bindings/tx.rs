use std::slice;
use crate::chain::common::ChainType;
use crate::consensus::Encodable;
use crate::crypto::{byte_util::{AsBytes, Reversable}, UInt160, UInt256};
use crate::ffi::ByteArray;
use crate::keys::ECDSAKey;
use crate::util::address::address;
use crate::util::data_ops::DASH_MESSAGE_MAGIC;


#[no_mangle]
pub unsafe extern "C" fn pro_reg_tx_verify_payload_signature(signature: *const u8, signature_len: usize, payload: *const u8, payload_len: usize, owner_key_hash: *const u8) -> bool {
    let signature = slice::from_raw_parts(signature, signature_len);
    let payload = slice::from_raw_parts(payload, payload_len);
    let owner_key_hash = slice::from_raw_parts(owner_key_hash, 20);
    let payload_hash = UInt256::sha256d(payload);
    ECDSAKey::key_with_compact_sig(signature, payload_hash)
        .map_or(false, |key| key.hash160().as_bytes().eq(owner_key_hash))
}
#[no_mangle]
pub unsafe extern "C" fn pro_reg_tx_payload_collateral_digest(
    payload: *const u8, payload_len: usize,
    script_payout: *const u8, script_payout_len: usize,
    operator_reward: u16,
    owner_key_hash: *const u8,
    voter_key_hash: *const u8,
    chain_type: ChainType) -> ByteArray {
    let payload = slice::from_raw_parts(payload, payload_len);
    let script_payout = slice::from_raw_parts(script_payout, script_payout_len);
    let owner_key_hash = UInt160::from(slice::from_raw_parts(owner_key_hash, 20));
    let voter_key_hash = UInt160::from(slice::from_raw_parts(voter_key_hash, 20));
    let script_map = chain_type.script_map();
    let mut writer = Vec::<u8>::new();
    DASH_MESSAGE_MAGIC.to_string().enc(&mut writer);
    let payout_address = address::with_script_pub_key(&script_payout.to_vec(), &script_map)
        .expect("Can't extract payout address");
    let payload_hash = UInt256::sha256d(payload).reverse();
    let owner_address = address::from_hash160_for_script_map(&owner_key_hash, &script_map);
    let voter_address = address::from_hash160_for_script_map(&voter_key_hash, &script_map);
    let payload_collateral_string = format!("{}|{}|{}|{}|{}", payout_address, operator_reward, owner_address, voter_address, payload_hash);
    payload_collateral_string.enc(&mut writer);
    ByteArray::from(UInt256::sha256d(&writer))
}
