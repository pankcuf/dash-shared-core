use crate::types;
use std::ptr::null_mut;
use dash_spv_masternode_processor::processing::ProcessingError;
use crate::ffi::boxer::{boxed, boxed_vec};
use crate::ffi::to::{encode_masternodes_map, encode_quorums_map, ToFFI};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MNListDiffResult {
    pub error_status: ProcessingError,
    pub base_block_hash: *mut [u8; 32],
    pub block_hash: *mut [u8; 32],
    pub has_found_coinbase: bool,       //1 byte
    pub has_valid_coinbase: bool,       //1 byte
    pub has_valid_mn_list_root: bool,   //1 byte
    pub has_valid_llmq_list_root: bool, //1 byte
    pub has_valid_quorums: bool,        //1 byte
    pub masternode_list: *mut types::MasternodeList,
    pub added_masternodes: *mut *mut types::MasternodeEntry,
    pub added_masternodes_count: usize,
    pub modified_masternodes: *mut *mut types::MasternodeEntry,
    pub modified_masternodes_count: usize,
    pub added_llmq_type_maps: *mut *mut types::LLMQMap,
    pub added_llmq_type_maps_count: usize,
    pub needed_masternode_lists: *mut *mut [u8; 32],
    pub needed_masternode_lists_count: usize,
    pub quorums_cl_sigs: *mut *mut types::QuorumsCLSigsObject,
    pub quorums_cl_sigs_count: usize,
}
impl MNListDiffResult {
    pub fn default_with_error(error: ProcessingError) -> Self {
        Self { error_status: error, ..Default::default() }
    }
}

impl Default for MNListDiffResult {
    fn default() -> Self {
        MNListDiffResult {
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

impl MNListDiffResult {
    pub fn is_valid(&self) -> bool {
        self.has_found_coinbase
            && self.has_valid_quorums
            && self.has_valid_mn_list_root
            && self.has_valid_llmq_list_root
    }
}

impl MNListDiffResult {
    pub fn encode(result: dash_spv_masternode_processor::processing::MNListDiffResult) -> Self {
        Self {
            error_status: result.error_status.into(),
            base_block_hash: boxed(result.base_block_hash.0),
            block_hash: boxed(result.block_hash.0),
            has_found_coinbase: result.has_found_coinbase,
            has_valid_coinbase: result.has_valid_coinbase,
            has_valid_mn_list_root: result.has_valid_mn_list_root,
            has_valid_llmq_list_root: result.has_valid_llmq_list_root,
            has_valid_quorums: result.has_valid_quorums,
            masternode_list: boxed(result.masternode_list.encode()),
            added_masternodes: encode_masternodes_map(&result.added_masternodes),
            added_masternodes_count: result.added_masternodes.len(),
            modified_masternodes: encode_masternodes_map(&result.modified_masternodes),
            modified_masternodes_count: result.modified_masternodes.len(),
            added_llmq_type_maps: encode_quorums_map(&result.added_quorums),
            added_llmq_type_maps_count: result.added_quorums.len(),
            needed_masternode_lists: boxed_vec(
                result.needed_masternode_lists
                    .iter()
                    .map(|h| boxed(h.0))
                    .collect(),
            ),
            needed_masternode_lists_count: result.needed_masternode_lists.len(),
            quorums_cl_sigs_count: result.quorums_cl_sigs.len(),
            quorums_cl_sigs: boxed_vec(
                result.quorums_cl_sigs
                    .iter()
                    .map(|h| boxed(h.encode()))
                    .collect())
        }
    }
}
