use crate::types::coinbase_transaction::CoinbaseTransaction;
use crate::types::llmq_entry::LLMQEntry;
use crate::types::masternode_entry::MasternodeEntryFFI;
use crate::types::LLMQTypedHash;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MNListDiff {
    pub base_block_hash: *mut [u8; 32],
    pub block_hash: *mut [u8; 32],
    pub total_transactions: u32,

    pub merkle_hashes: *mut *mut [u8; 32],
    pub merkle_hashes_count: usize,

    pub merkle_flags: *mut u8,
    pub merkle_flags_count: usize,

    pub coinbase_transaction: *mut CoinbaseTransaction,

    pub deleted_masternode_hashes_count: usize,
    pub deleted_masternode_hashes: *mut *mut [u8; 32],

    pub added_or_modified_masternodes_count: usize,
    pub added_or_modified_masternodes: *mut *mut MasternodeEntryFFI,

    pub deleted_quorums_count: usize,
    pub deleted_quorums: *mut *mut LLMQTypedHash,

    pub added_quorums_count: usize,
    pub added_quorums: *mut *mut LLMQEntry,

    pub base_block_height: u32,
    pub block_height: u32,
}
