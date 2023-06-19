use dash_spv_ffi::FFIConversion;
use crate::types;
use crate::models::MasternodeList;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[dash_spv_macro_derive::ffi_conversion(MasternodeList)]
pub struct MasternodeListFFI {
    pub block_hash: *mut [u8; 32],
    pub known_height: u32,
    pub masternode_merkle_root: *mut [u8; 32], // nullable
    pub llmq_merkle_root: *mut [u8; 32],       // nullable
    pub masternodes: *mut *mut types::MasternodeEntryFFI,
    pub masternodes_count: usize,
    pub llmq_type_maps: *mut *mut types::LLMQMap,
    pub llmq_type_maps_count: usize,
}
