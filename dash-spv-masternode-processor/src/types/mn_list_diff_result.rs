use dash_spv_ffi::{boxed, boxed_vec, FFIConversion};
use dash_spv_macro_derive::ffi_conversion;
use std::ptr::null_mut;
use crate::processing::{MNListDiffResult, ProcessingError};
use crate::types;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[ffi_conversion(MNListDiffResult)]
pub struct MNListDiffResultFFI {
    pub error_status: ProcessingError,
    pub base_block_hash: *mut [u8; 32],
    pub block_hash: *mut [u8; 32],
    pub has_found_coinbase: bool,       //1 byte
    pub has_valid_coinbase: bool,       //1 byte
    pub has_valid_mn_list_root: bool,   //1 byte
    pub has_valid_llmq_list_root: bool, //1 byte
    pub has_valid_quorums: bool,        //1 byte
    pub masternode_list: *mut types::MasternodeListFFI,
    pub added_masternodes: *mut *mut types::MasternodeEntryFFI,
    pub added_masternodes_count: usize,
    pub modified_masternodes: *mut *mut types::MasternodeEntryFFI,
    pub modified_masternodes_count: usize,
    pub added_llmq_type_maps: *mut *mut types::LLMQMap,
    pub added_llmq_type_maps_count: usize,
    pub needed_masternode_lists: *mut *mut [u8; 32],
    pub needed_masternode_lists_count: usize,
    pub quorums_cl_sigs: *mut *mut types::QuorumsCLSigsObject,
    pub quorums_cl_sigs_count: usize,
}
impl MNListDiffResultFFI {
    pub fn default_with_error(error: ProcessingError) -> Self {
        Self { error_status: error, ..Default::default() }
    }
}

impl Default for MNListDiffResultFFI {
    fn default() -> Self {
        MNListDiffResultFFI {
            error_status: ProcessingError::None,
            base_block_hash: null_mut(),
            block_hash: null_mut(),
            has_found_coinbase: false,
            has_valid_coinbase: false,
            has_valid_mn_list_root: false,
            has_valid_llmq_list_root: false,
            has_valid_quorums: false,
            masternode_list: null_mut(),
            added_masternodes: null_mut(),
            added_masternodes_count: 0,
            modified_masternodes: null_mut(),
            modified_masternodes_count: 0,
            added_llmq_type_maps: null_mut(),
            added_llmq_type_maps_count: 0,
            needed_masternode_lists: null_mut(),
            needed_masternode_lists_count: 0,
            quorums_cl_sigs: null_mut(),
            quorums_cl_sigs_count: 0,
        }
    }
}

impl MNListDiffResultFFI {
    pub fn is_valid(&self) -> bool {
        self.has_found_coinbase
            && self.has_valid_quorums
            && self.has_valid_mn_list_root
            && self.has_valid_llmq_list_root
    }
}
