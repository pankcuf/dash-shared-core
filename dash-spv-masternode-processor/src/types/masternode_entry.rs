use crate::types::{BlockOperatorPublicKey, MasternodeEntryHash, OperatorPublicKey, Validity};
#[repr(C)]
#[derive(Clone, Copy, Debug)]
// #[dash_spv_macro_derive::ffi_conversion(MasternodeEntry)]
pub struct MasternodeEntryFFI {
    pub confirmed_hash: *mut [u8; 32],
    pub confirmed_hash_hashed_with_provider_registration_transaction_hash: *mut [u8; 32], // nullable
    pub is_valid: bool,
    pub key_id_voting: *mut [u8; 20],
    pub known_confirmed_at_height: u32, // nullable
    pub entry_hash: *mut [u8; 32],
    pub operator_public_key: *mut OperatorPublicKey,
    pub previous_entry_hashes: *mut MasternodeEntryHash,
    pub previous_entry_hashes_count: usize,
    pub previous_operator_public_keys: *mut BlockOperatorPublicKey,
    pub previous_operator_public_keys_count: usize,
    pub previous_validity: *mut Validity,
    pub previous_validity_count: usize,
    pub provider_registration_transaction_hash: *mut [u8; 32],
    pub ip_address: *mut [u8; 16],
    pub port: u16,
    pub update_height: u32,
    // Core v0.19 (70227+)
    // 0: regular, 1: high performance
    pub mn_type: u16,
    pub platform_http_port: u16,
    pub platform_node_id: *mut [u8; 20],
}
