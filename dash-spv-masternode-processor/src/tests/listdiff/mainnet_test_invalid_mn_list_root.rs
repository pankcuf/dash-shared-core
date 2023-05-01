use crate::lib_tests::tests::{add_insight_lookup_default, FFIContext, get_block_hash_by_height_from_context, get_block_height_by_hash_from_context, get_llmq_snapshot_by_block_hash_from_context, get_masternode_list_by_block_hash_from_cache, get_merkle_root_by_hash_default, hash_destroy_default, masternode_list_destroy_default, masternode_list_save_in_cache, message_from_file, save_llmq_snapshot_in_cache, should_process_diff_with_range_default, snapshot_destroy_default};
use crate::bindings::common::register_processor;
use crate::bindings::masternode::process_mnlistdiff_from_message;
use crate::chain::common::ChainType;
use crate::tests::block_store::init_mainnet_store;
use crate::tests::json_from_core_snapshot::masternode_list_from_json;

#[test]
fn mainnet_test_invalid_mn_list_root() {
    let chain = ChainType::MainNet;
    let context = &mut (FFIContext {
        chain,
        cache: &mut Default::default(),
        is_dip_0024: false,
        blocks: init_mainnet_store()
    });
    let masternode_list_1761054 = masternode_list_from_json("MNLIST_1761054_1666771101.811508_saveMasternodeList.json".to_string());
    let masternode_list_1761048 = masternode_list_from_json("MNLIST_1761048_1666773093.153379_saveMasternodeList.json".to_string());

    let bytes = message_from_file("MNL_1761054_1761100.dat");
    let processor = unsafe {
        &mut *register_processor(
            get_merkle_root_by_hash_default,
            get_block_height_by_hash_from_context,
            get_block_hash_by_height_from_context,
            get_llmq_snapshot_by_block_hash_from_context,
            save_llmq_snapshot_in_cache,
            get_masternode_list_by_block_hash_from_cache,
            masternode_list_save_in_cache,
            masternode_list_destroy_default,
            add_insight_lookup_default,
            hash_destroy_default,
            snapshot_destroy_default,
            should_process_diff_with_range_default,
        )
    };
    processor.opaque_context = context as *mut _ as *mut std::ffi::c_void;
    processor.use_insight_as_backup = true;
    processor.chain_type = context.chain;

    processor.save_masternode_list(masternode_list_1761048.block_hash, &masternode_list_1761048);
    processor.save_masternode_list(masternode_list_1761054.block_hash, &masternode_list_1761054);
    let result = unsafe {
        *process_mnlistdiff_from_message(
            bytes.as_ptr(),
            bytes.len(),
            chain,
            false,
            false,
            70221,
            processor,
            context.cache,
            context as *mut _ as *mut std::ffi::c_void,
        )};
    // println!("{:#?}", result);
    // assert_diff_result(context, result);
}

