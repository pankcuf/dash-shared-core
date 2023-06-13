#![allow(unused_variables)]
#![allow(dead_code)]

use std::ffi::CString;
use std::os::raw::c_char;
use crate::types;
use crate::types::opaque_key::{OpaqueKey, OpaqueKeys, OpaqueSerializedKeys};

/// # Safety
pub unsafe fn unbox_any<T: ?Sized>(any: *mut T) -> Box<T> {
    Box::from_raw(any)
}

/// # Safety
pub unsafe fn unbox_vec<T>(vec: Vec<*mut T>) -> Vec<Box<T>> {
    vec.iter().map(|&x| unbox_any(x)).collect()
}

/// # Safety
pub unsafe fn unbox_vec_ptr<T>(ptr: *mut T, count: usize) -> Vec<T> {
    Vec::from_raw_parts(ptr, count, count)
}

/// # Safety
pub unsafe fn unbox_masternode_entry(x: *mut types::MasternodeEntry) {
    let entry = unbox_any(x);
    unbox_any(entry.confirmed_hash);
    if !entry
        .confirmed_hash_hashed_with_provider_registration_transaction_hash
        .is_null()
    {
        unbox_any(entry.confirmed_hash_hashed_with_provider_registration_transaction_hash);
    }
    unbox_any(entry.key_id_voting);
    unbox_any(entry.entry_hash);
    unbox_any(entry.operator_public_key);
    unbox_vec_ptr(
        entry.previous_entry_hashes,
        entry.previous_entry_hashes_count,
    );
    unbox_vec_ptr(
        entry.previous_operator_public_keys,
        entry.previous_operator_public_keys_count,
    );
    unbox_vec_ptr(entry.previous_validity, entry.previous_validity_count);
    unbox_any(entry.provider_registration_transaction_hash);
    unbox_any(entry.ip_address);
    unbox_any(entry.platform_node_id);
}

/// # Safety
pub unsafe fn unbox_llmq_entry(x: *mut types::LLMQEntry) {
    let entry = unbox_any(x);
    unbox_any(entry.all_commitment_aggregated_signature);
    if !entry.commitment_hash.is_null() {
        unbox_any(entry.commitment_hash);
    }
    unbox_any(entry.entry_hash);
    unbox_any(entry.llmq_hash);
    unbox_any(entry.public_key);
    unbox_any(entry.threshold_signature);
    unbox_any(entry.verification_vector_hash);
    unbox_any(std::ptr::slice_from_raw_parts_mut::<u8>(
        entry.signers_bitset,
        entry.signers_bitset_length,
    ));
    unbox_any(std::ptr::slice_from_raw_parts_mut::<u8>(
        entry.valid_members_bitset,
        entry.valid_members_bitset_length,
    ));
}

/// # Safety
pub unsafe fn unbox_llmq_map(x: *mut types::LLMQMap) {
    let entry = unbox_any(x);
    let values = unbox_vec_ptr(entry.values, entry.count);
    for &x in values.iter() {
        unbox_llmq_entry(x);
    }
}

/// # Safety
pub unsafe fn unbox_masternode_list(list: *mut types::MasternodeList) {
    let masternode_list = unbox_any(list);
    unbox_any(masternode_list.block_hash);
    if !masternode_list.masternode_merkle_root.is_null() {
        unbox_any(masternode_list.masternode_merkle_root);
    }
    if !masternode_list.llmq_merkle_root.is_null() {
        unbox_any(masternode_list.llmq_merkle_root);
    }
    unbox_masternode_vec(unbox_vec_ptr(
        masternode_list.masternodes,
        masternode_list.masternodes_count,
    ));
    unbox_llmq_map_vec(unbox_vec_ptr(
        masternode_list.llmq_type_maps,
        masternode_list.llmq_type_maps_count,
    ));
}
/// # Safety
pub unsafe fn unbox_quorums_cl_sigs_object(x: *mut types::QuorumsCLSigsObject) {
    let result = unbox_any(x);
    unbox_any(result.signature);
    let index_set = unbox_vec_ptr(result.index_set, result.index_set_count);
    drop(index_set);

}
/// # Safety
pub unsafe fn unbox_quorums_cl_sigs_vec(vec: Vec<*mut types::QuorumsCLSigsObject>) {
    for &x in vec.iter() {
        unbox_quorums_cl_sigs_object(x);
    }
}

/// # Safety
pub unsafe fn unbox_masternode_vec(vec: Vec<*mut types::MasternodeEntry>) {
    for &x in vec.iter() {
        unbox_masternode_entry(x);
    }
}

/// # Safety
pub unsafe fn unbox_llmq_vec(vec: Vec<*mut types::LLMQEntry>) {
    for &x in vec.iter() {
        unbox_llmq_entry(x);
    }
}

/// # Safety
pub unsafe fn unbox_llmq_map_vec(vec: Vec<*mut types::LLMQMap>) {
    for &x in vec.iter() {
        unbox_llmq_map(x);
    }
}

/// # Safety
pub unsafe fn unbox_llmq_hash_vec(vec: Vec<*mut types::LLMQTypedHash>) {
    for &x in vec.iter() {
        unbox_llmq_typed_hash(x);
    }
}

/// # Safety
pub unsafe fn unbox_llmq_typed_hash(typed_hash: *mut types::LLMQTypedHash) {
    let hash = unbox_any(typed_hash);
    unbox_any(hash.llmq_hash);
}

/// # Safety
pub unsafe fn unbox_llmq_validation_data(llmq_validation_data: *mut types::LLMQValidationData) {
    let result = unbox_any(llmq_validation_data);
    unbox_any(result.all_commitment_aggregated_signature);
    unbox_any(result.commitment_hash);
    unbox_any(result.public_key);
    unbox_any(result.threshold_signature);
    unbox_vec(unbox_vec_ptr(result.items, result.count));
}

/// # Safety
pub unsafe fn unbox_snapshot_vec(vec: Vec<*mut types::LLMQSnapshot>) {
    for &x in vec.iter() {
        unbox_llmq_snapshot(x);
    }
}

/// # Safety
pub unsafe fn unbox_mn_list_diff_result_vec(vec: Vec<*mut types::MNListDiffResult>) {
    for &x in vec.iter() {
        unbox_mn_list_diff_result(x);
    }
}

/// # Safety
pub unsafe fn unbox_block(block: *mut types::Block) {
    let result = unbox_any(block);
    unbox_any(result.hash);
}

/// # Safety
pub unsafe fn unbox_llmq_indexed_hash(indexed_hash: *mut types::LLMQIndexedHash) {
    let result = unbox_any(indexed_hash);
    unbox_any(result.hash);
}

/// # Safety
pub unsafe fn unbox_llmq_snapshot(quorum_snapshot: *mut types::LLMQSnapshot) {
    let result = unbox_any(quorum_snapshot);
    let member_list = unbox_vec_ptr(result.member_list, result.member_list_length);
    drop(member_list);
    let skip_list = unbox_vec_ptr(result.skip_list, result.skip_list_length);
    drop(skip_list);
}

/// # Safety
pub unsafe fn unbox_tx_input(result: *mut types::TransactionInput) {
    let input = unbox_any(result);
    unbox_any(input.input_hash);
    if !input.script.is_null() && input.script_length > 0 {
        unbox_any(
            std::ptr::slice_from_raw_parts_mut(input.script, input.script_length) as *mut [u8],
        );
    }
    if !input.signature.is_null() && input.signature_length > 0 {
        unbox_any(
            std::ptr::slice_from_raw_parts_mut(input.signature, input.signature_length)
                as *mut [u8],
        );
    }
}

/// # Safety
pub unsafe fn unbox_tx_output(result: *mut types::TransactionOutput) {
    let output = unbox_any(result);
    if !output.script.is_null() && output.script_length > 0 {
        unbox_any(
            std::ptr::slice_from_raw_parts_mut(output.script, output.script_length) as *mut [u8],
        );
    }
    if !output.address.is_null() && output.address_length > 0 {
        unbox_any(
            std::ptr::slice_from_raw_parts_mut(output.address, output.address_length)
                as *mut [u8],
        );
    }
}

/// # Safety
pub unsafe fn unbox_tx_input_vec(result: Vec<*mut types::TransactionInput>) {
    for &x in result.iter() {
        unbox_tx_input(x);
    }
}

/// # Safety
pub unsafe fn unbox_tx_output_vec(result: Vec<*mut types::TransactionOutput>) {
    for &x in result.iter() {
        unbox_tx_output(x);
    }
}

/// # Safety
pub unsafe fn unbox_tx(result: *mut types::Transaction) {
    let tx = unbox_any(result);
    unbox_tx_input_vec(unbox_vec_ptr(tx.inputs, tx.inputs_count));
    unbox_tx_output_vec(unbox_vec_ptr(tx.outputs, tx.outputs_count));
    unbox_any(tx.tx_hash);
}

/// # Safety
pub unsafe fn unbox_coinbase_tx(result: *mut types::CoinbaseTransaction) {
    let ctx = unbox_any(result);
    unbox_tx(ctx.base);
    unbox_any(ctx.merkle_root_mn_list);
    if !ctx.merkle_root_llmq_list.is_null() {
        unbox_any(ctx.merkle_root_llmq_list);
    }
    if !ctx.best_cl_signature.is_null() {
        unbox_any(ctx.best_cl_signature);
    }
}

/// # Safety
pub unsafe fn unbox_mn_list_diff_result(result: *mut types::MNListDiffResult) {
    let res = unbox_any(result);
    if !res.base_block_hash.is_null() {
        unbox_any(res.base_block_hash);
    }
    if !res.block_hash.is_null() {
        unbox_any(res.block_hash);
    }
    if !res.masternode_list.is_null() {
        unbox_masternode_list(res.masternode_list);
    }
    if !res.needed_masternode_lists.is_null() {
        unbox_vec(unbox_vec_ptr(
            res.needed_masternode_lists,
            res.needed_masternode_lists_count,
        ));
    }
    if !res.added_masternodes.is_null() {
        unbox_masternode_vec(unbox_vec_ptr(
            res.added_masternodes,
            res.added_masternodes_count,
        ));
    }
    if !res.modified_masternodes.is_null() {
        unbox_masternode_vec(unbox_vec_ptr(
            res.modified_masternodes,
            res.modified_masternodes_count,
        ));
    }
    if !res.added_llmq_type_maps.is_null() {
        unbox_llmq_map_vec(unbox_vec_ptr(
            res.added_llmq_type_maps,
            res.added_llmq_type_maps_count,
        ));
    }
    if !res.quorums_cl_sigs.is_null() {
        unbox_quorums_cl_sigs_vec(unbox_vec_ptr(
            res.quorums_cl_sigs,
            res.quorums_cl_sigs_count,
        ));
    }
}

/// # Safety
pub unsafe fn unbox_qr_info_result(result: *mut types::QRInfoResult) {
    let res = unbox_any(result);
    if !res.result_at_tip.is_null() {
        unbox_mn_list_diff_result(res.result_at_tip);
    }
    if !res.result_at_h.is_null() {
        unbox_mn_list_diff_result(res.result_at_h);
    }
    if !res.result_at_h_c.is_null() {
        unbox_mn_list_diff_result(res.result_at_h_c);
    }
    if !res.result_at_h_2c.is_null() {
        unbox_mn_list_diff_result(res.result_at_h_2c);
    }
    if !res.result_at_h_3c.is_null() {
        unbox_mn_list_diff_result(res.result_at_h_3c);
    }
    if !res.snapshot_at_h_c.is_null() {
        unbox_llmq_snapshot(res.snapshot_at_h_c);
    }
    if !res.snapshot_at_h_2c.is_null() {
        unbox_llmq_snapshot(res.snapshot_at_h_2c);
    }
    if !res.snapshot_at_h_3c.is_null() {
        unbox_llmq_snapshot(res.snapshot_at_h_3c);
    }
    if res.extra_share {
        if !res.result_at_h_4c.is_null() {
            unbox_mn_list_diff_result(res.result_at_h_4c);
        }
        if !res.snapshot_at_h_4c.is_null() {
            unbox_llmq_snapshot(res.snapshot_at_h_4c);
        }
    }
    if !res.last_quorum_per_index.is_null() {
        unbox_llmq_vec(unbox_vec_ptr(
            res.last_quorum_per_index,
            res.last_quorum_per_index_count,
        ));
    }
    if !res.quorum_snapshot_list.is_null() {
        unbox_snapshot_vec(unbox_vec_ptr(
            res.quorum_snapshot_list,
            res.quorum_snapshot_list_count,
        ));
    }
    if !res.mn_list_diff_list.is_null() {
        unbox_mn_list_diff_result_vec(unbox_vec_ptr(
            res.mn_list_diff_list,
            res.mn_list_diff_list_count,
        ));
    }
}

/// # Safety
pub unsafe fn unbox_string(data: *mut c_char) {
    let _ = CString::from_raw(data);
}

pub unsafe fn unbox_opaque_key(data: *mut OpaqueKey) {
    let k = unbox_any(data);
    match *k {
        OpaqueKey::ECDSA(key) => { let _ = unbox_any(key); },
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => { let _ = unbox_any(key); },
        OpaqueKey::ED25519(key) => { let _ = unbox_any(key); },
    };
}

/// # Safety
pub unsafe fn unbox_opaque_keys(data: *mut OpaqueKeys) {
    let res = unbox_any(data);
    let keys = unbox_vec_ptr(res.keys, res.len);
    for &x in keys.iter() {
        unbox_opaque_key(x);
    }
}

/// # Safety
pub unsafe fn unbox_opaque_serialized_keys(data: *mut OpaqueSerializedKeys) {
    let res = unbox_any(data);
    let keys = unbox_vec_ptr(res.keys, res.len);
    for &x in keys.iter() {
        unbox_string(x)
    }
}
