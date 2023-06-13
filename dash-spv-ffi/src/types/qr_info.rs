use std::ptr::null_mut;
use crate::types::llmq_snapshot::LLMQSnapshot;
use crate::types::mn_list_diff::MNListDiff;
use crate::types::LLMQEntry;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct QRInfo {
    pub snapshot_at_h_c: *mut LLMQSnapshot,
    pub snapshot_at_h_2c: *mut LLMQSnapshot,
    pub snapshot_at_h_3c: *mut LLMQSnapshot,
    pub snapshot_at_h_4c: *mut LLMQSnapshot, // exist only if extra_share is true
    pub mn_list_diff_tip: *mut MNListDiff,
    pub mn_list_diff_at_h: *mut MNListDiff,
    pub mn_list_diff_at_h_c: *mut MNListDiff,
    pub mn_list_diff_at_h_2c: *mut MNListDiff,
    pub mn_list_diff_at_h_3c: *mut MNListDiff,
    pub mn_list_diff_at_h_4c: *mut MNListDiff, // exist only if extra_share is true
    pub extra_share: bool,
    pub last_quorum_per_index: *mut *mut LLMQEntry,
    pub last_quorum_per_index_count: usize,
    pub quorum_snapshot_list: *mut *mut LLMQSnapshot,
    pub quorum_snapshot_list_count: usize,
    pub mn_list_diff_list: *mut *mut MNListDiff,
    pub mn_list_diff_list_count: usize,
}

impl Default for QRInfo {
    fn default() -> Self {
        QRInfo {
            snapshot_at_h_c: null_mut(),
            snapshot_at_h_2c: null_mut(),
            snapshot_at_h_3c: null_mut(),
            mn_list_diff_tip: null_mut(),
            mn_list_diff_at_h: null_mut(),
            mn_list_diff_at_h_c: null_mut(),
            mn_list_diff_at_h_2c: null_mut(),
            mn_list_diff_at_h_3c: null_mut(),
            extra_share: false,
            snapshot_at_h_4c: null_mut(),
            mn_list_diff_at_h_4c: null_mut(),
            last_quorum_per_index: null_mut(),
            last_quorum_per_index_count: 0,
            quorum_snapshot_list: null_mut(),
            quorum_snapshot_list_count: 0,
            mn_list_diff_list: null_mut(),
            mn_list_diff_list_count: 0,
        }
    }
}
