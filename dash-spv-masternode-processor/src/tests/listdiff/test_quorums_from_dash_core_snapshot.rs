use crate::bindings::common::{processor_create_cache, register_processor};
use crate::chain::common::{ChainType, IHaveChainSettings};
use crate::crypto::{byte_util::Reversable, UInt256};
use crate::lib_tests::tests::{add_insight_lookup_default, FFIContext, get_block_hash_by_height_from_context, get_block_height_by_hash_from_context, get_llmq_snapshot_by_block_hash_default, get_masternode_list_by_block_hash_default, get_merkle_root_by_hash_default, hash_destroy_default, masternode_list_destroy_default, masternode_list_save_default, message_from_file, save_llmq_snapshot_default, should_process_diff_with_range_default, snapshot_destroy_default};
use crate::tests::block_store::init_testnet_store;
use crate::tests::json_from_core_snapshot::{masternode_list_from_genesis_diff, QRInfo, snapshot_to_snapshot};

#[test]
pub fn test_from_snapshot() {
    let qrinfo: QRInfo = serde_json::from_slice(&message_from_file("snapshot_0000021715c8575620382ceee42cc7556bac5ed395eaf9c75e2119aa2876a1e0.json")).unwrap();
    let chain = ChainType::TestNet;
    let cache = unsafe { &mut *processor_create_cache() };
    let context = &mut (FFIContext {
        chain,
        is_dip_0024: true,
        cache,
        blocks: init_testnet_store()
    });
    let block_height_lookup = |hash: UInt256| context.block_for_hash(hash).unwrap().height;
    let quorum_snapshot_h_c = snapshot_to_snapshot(qrinfo.quorum_snapshot_at_hminus_c);
    let quorum_snapshot_h_2c = snapshot_to_snapshot(qrinfo.quorum_snapshot_at_hminus2c);
    let quorum_snapshot_h_3c = snapshot_to_snapshot(qrinfo.quorum_snapshot_at_hminus3c);
    let mn_list_diff_tip = masternode_list_from_genesis_diff(qrinfo.mn_list_diff_tip, block_height_lookup, false);
    let mn_list_diff_h = masternode_list_from_genesis_diff(qrinfo.mn_list_diff_h, block_height_lookup, false);
    let mn_list_diff_h_c = masternode_list_from_genesis_diff(qrinfo.mn_list_diff_at_hminus_c, block_height_lookup, false);
    let mn_list_diff_h_2c = masternode_list_from_genesis_diff(qrinfo.mn_list_diff_at_hminus2c, block_height_lookup, false);
    let mn_list_diff_h_3c = masternode_list_from_genesis_diff(qrinfo.mn_list_diff_at_hminus3c, block_height_lookup, false);
    let processor = unsafe { &mut *register_processor(
        get_merkle_root_by_hash_default,
        get_block_height_by_hash_from_context,
        get_block_hash_by_height_from_context,
            get_llmq_snapshot_by_block_hash_default,
            save_llmq_snapshot_default,
            get_masternode_list_by_block_hash_default,
            masternode_list_save_default,
            masternode_list_destroy_default,
            add_insight_lookup_default,
            hash_destroy_default,
            snapshot_destroy_default,
            should_process_diff_with_range_default)
    };
    processor.opaque_context = context as *mut _ as *mut std::ffi::c_void;
    processor.use_insight_as_backup = true;
    processor.chain_type = context.chain;

    println!("rotated_quorums at h ({}: {})", mn_list_diff_h.block_height, mn_list_diff_h.block_hash);
    let cached_blocks = &context.blocks;
    let get_height = |hash: UInt256| cached_blocks.iter().find(|block| block.hash == hash.reversed()).unwrap().height;
    let cached_llmq_members = &mut context.cache.llmq_members;
    let cached_llmq_indexed_members = &mut context.cache.llmq_indexed_members;
    if let Some(rotated_quorums_h) = mn_list_diff_h.added_quorums.get(&chain.isd_llmq_type()) {
        rotated_quorums_h.iter().for_each(|(&llmq_block_hash, entry)| {
            println!("rotated_quorum: ({}: {})", llmq_block_hash, llmq_block_hash.reversed());
            let llmq_block_height = get_height(llmq_block_hash);
            println!("rotated_quorum: ({}: {})\n {:#?}", llmq_block_height, llmq_block_hash, entry);

            let masternodes = processor.get_rotated_masternodes_for_quorum(
                entry.llmq_type,
                llmq_block_hash,
                llmq_block_height,
                cached_llmq_members,
                cached_llmq_indexed_members,
                &context.cache.mn_lists,
                &context.cache.llmq_snapshots,
                &mut context.cache.needed_masternode_lists,
                false
            );
            println!("masternodes: {:#?}", masternodes);
        });
    }
    // let masternodes = processor
    //     .get_rotated_masternodes_for_quorum(chain.isd_llmq_type(), mn_list_diff_h.added_quorums)


    // let result_at_tip = processor.get_list_diff_result_internal_with_base_lookup(mn_list_diff_tip, cache);
    // let result_at_h = processor.get_list_diff_result_internal_with_base_lookup(mn_list_diff_h, cache);
    // let result_at_h_c = processor.get_list_diff_result_internal_with_base_lookup(mn_list_diff_h_c, cache);
    // let result_at_h_2c = processor.get_list_diff_result_internal_with_base_lookup(mn_list_diff_h_2c, cache);
    // let result_at_h_3c = processor.get_list_diff_result_internal_with_base_lookup(mn_list_diff_h_3c, cache);
    // println!("result_at_tip: {:#?}", result_at_tip);
    // println!("result_at_h: {:#?}", result_at_h);
    // println!("result_at_h_c: {:#?}", result_at_h_c);
    // println!("result_at_h_2c: {:#?}", result_at_h_2c);
    // println!("result_at_h_3c: {:#?}", result_at_h_3c);
}

