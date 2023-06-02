use std::collections::BTreeMap;
use std::ptr::null_mut;
use bls_signatures::G1Element;
use hashes::hex::FromHex;
use crate::{models, types};
use crate::bindings::common::{processor_create_cache, register_processor};
use crate::bindings::masternode::process_mnlistdiff_from_message;
use crate::chain::common::{ChainType, DevnetType};
use crate::crypto::{UInt256, UInt384};
use crate::ffi::{boxer::boxed, to::ToFFI};
use crate::lib_tests::tests::{add_insight_lookup_default, assert_diff_result, FFIContext, get_block_height_by_hash_from_context, get_block_hash_by_height_from_context, get_llmq_snapshot_by_block_hash_default, get_merkle_root_by_hash_default, hash_destroy_default, masternode_list_destroy_default, masternode_list_save_in_cache, MerkleBlock, message_from_file, save_llmq_snapshot_in_cache, should_process_diff_with_range_default, snapshot_destroy_default};
use crate::models::OperatorPublicKey;
use crate::tests::block_store::init_testnet_store;

unsafe extern "C" fn get_merkle_root_for_chacha(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> *mut u8 {
    let h = UInt256(*(block_hash));
    // for block_hash '9993903c63b96f9a3846692535a11da2525561f0d61c7d31b7222bfddf020000':
    let merkle_root =
        UInt256::from_hex("42a84456a608ade07581c35e1087634743f6293c56dbdc01930ad97df0f08b2e")
            .unwrap();
    println!("get_merkle_root_for_chacha: {}: {}", h, merkle_root);
    boxed(merkle_root.0) as *mut _
}

unsafe extern "C" fn get_block_height_by_hash_chacha(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> u32 {
    let h = UInt256(*(block_hash));
    let orig_s = h.clone().to_string();
    match orig_s.as_str() {
        "8862eca4bdb5255b51dc72903b8a842f6ffe7356bc40c7b7a7437b8e4556e220" => 1,
        "3a29104cd905e2a584d5f1844a0f421a2af0fd9ad7840a0ebb097a628d1b0000" => 9192,
        "9993903c63b96f9a3846692535a11da2525561f0d61c7d31b7222bfddf020000" => 9247,
        _ => u32::MAX,
    }
}
unsafe extern "C" fn get_block_hash_by_height_chacha(
    block_height: u32,
    _context: *const std::ffi::c_void,
) -> *mut u8 {
    match block_height {
        9192 => boxed(UInt256::from_hex("3a29104cd905e2a584d5f1844a0f421a2af0fd9ad7840a0ebb097a628d1b0000").unwrap().0) as *mut _,
        9184 => boxed(UInt256::from_hex("3a29104cd905e2a584d5f1844a0f421a2af0fd9ad7840a0ebb097a628d1b0000").unwrap().0) as *mut _,
        9160 => boxed(UInt256::from_hex("3a29104cd905e2a584d5f1844a0f421a2af0fd9ad7840a0ebb097a628d1b0000").unwrap().0) as *mut _,
        9136 => boxed(UInt256::from_hex("3a29104cd905e2a584d5f1844a0f421a2af0fd9ad7840a0ebb097a628d1b0000").unwrap().0) as *mut _,
        9120 => boxed(UInt256::from_hex("3a29104cd905e2a584d5f1844a0f421a2af0fd9ad7840a0ebb097a628d1b0000").unwrap().0) as *mut _,
        // 9112 => boxed(UInt256::from_hex("3a29104cd905e2a584d5f1844a0f421a2af0fd9ad7840a0ebb097a628d1b0000").unwrap().0) as *mut _,
        _ => null_mut()
    }
}

pub unsafe extern "C" fn get_masternode_list_at_9192(
    block_hash: *mut [u8; 32],
    context: *const std::ffi::c_void,
) -> *mut types::MasternodeList {
    let h = UInt256(*(block_hash));
    let nodes = BTreeMap::new();
    let quorums = BTreeMap::new();
    let list = models::MasternodeList::new(nodes, quorums, h, 9192, true);
    let encoded = list.encode();
    boxed(encoded)
}


#[test]
fn test_basic_bls_scheme() {
    let chain = ChainType::DevNet(DevnetType::Chacha);
    let processor = unsafe {
        register_processor(
            get_merkle_root_for_chacha,
            get_block_height_by_hash_chacha,
            get_block_hash_by_height_chacha,
            get_llmq_snapshot_by_block_hash_default,
            save_llmq_snapshot_in_cache,
            get_masternode_list_at_9192,
            masternode_list_save_in_cache,
            masternode_list_destroy_default,
            add_insight_lookup_default,
            hash_destroy_default,
            snapshot_destroy_default,
            should_process_diff_with_range_default,
        )
    };
    let cache = unsafe { &mut *processor_create_cache() };
    let context = &mut (FFIContext {
        chain,
        is_dip_0024: false,
        cache,
        blocks: vec![]
    }) as *mut _ as *mut std::ffi::c_void;
    let bytes = message_from_file("MNL_1_9247.dat");
    let result = unsafe { process_mnlistdiff_from_message(
        bytes.as_ptr(),
        bytes.len(),
        chain,
        false,
        true,
        70225,
        processor,
        cache,
        context,
    )};
    println!("Result: {:#?}", &result);
}

unsafe extern "C" fn get_merkle_root_for_mojito(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> *mut u8 {
    let h = UInt256(*(block_hash));
    // 720ea2e4e7f6b31debe9bb852e1e4cdfdf10bed9827f0ef6527cfa0261010000 -> f0597c739df147363e06988fb4132dde4fbc66418b28a4e5d74e552ad2d555d0
    let merkle_root = UInt256::from_hex("f0597c739df147363e06988fb4132dde4fbc66418b28a4e5d74e552ad2d555d0")
            .unwrap();
    println!("get_merkle_root_for_mojito: {}: {}", h, merkle_root);
    boxed(merkle_root.0) as *mut _
}
unsafe extern "C" fn get_block_height_by_hash_mojito(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> u32 {
    let h = UInt256(*(block_hash));
    let orig_s = h.clone().to_string();
    match orig_s.as_str() {
        "739507391fa00da48a2ecae5df3b5e40b4432243603db6dafe33ca6b4966e357" => 1,
        "720ea2e4e7f6b31debe9bb852e1e4cdfdf10bed9827f0ef6527cfa0261010000" => 4450,
        _ => u32::MAX,
    }
}
unsafe extern "C" fn get_block_hash_by_height_mojito(
    block_height: u32,
    _context: *const std::ffi::c_void,
) -> *mut u8 {
    match block_height {
        1 => boxed(UInt256::from_hex("739507391fa00da48a2ecae5df3b5e40b4432243603db6dafe33ca6b4966e357").unwrap().0) as *mut _,
        4450 => boxed(UInt256::from_hex("720ea2e4e7f6b31debe9bb852e1e4cdfdf10bed9827f0ef6527cfa0261010000").unwrap().0) as *mut _,
        _ => null_mut()
    }
}
pub unsafe extern "C" fn get_masternode_list_mojito(
    block_hash: *mut [u8; 32],
    context: *const std::ffi::c_void,
) -> *mut types::MasternodeList {
    null_mut()
    // let h = UInt256(*(block_hash));
    // let nodes = BTreeMap::new();
    // let quorums = BTreeMap::new();
    // let list = models::MasternodeList::new(nodes, quorums, h, 9192, true);
    // let encoded = list.encode();
    // boxed(encoded)
}

unsafe extern "C" fn get_merkle_root_for_white_russian(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> *mut u8 {
    let h = UInt256(*(block_hash));
    // 720ea2e4e7f6b31debe9bb852e1e4cdfdf10bed9827f0ef6527cfa0261010000 -> f0597c739df147363e06988fb4132dde4fbc66418b28a4e5d74e552ad2d555d0
    let merkle_root = UInt256::from_hex("f0597c739df147363e06988fb4132dde4fbc66418b28a4e5d74e552ad2d555d0")
        .unwrap();
    println!("get_merkle_root_for_white_russian: {}: {}", h, merkle_root);
    boxed(merkle_root.0) as *mut _
}

unsafe extern "C" fn get_block_hash_by_height_white_russian(
    block_height: u32,
    _context: *const std::ffi::c_void,
) -> *mut u8 {
    println!("get_block_hash_by_height_white_russian: {}", block_height);
    match block_height {
        1 => boxed(UInt256::from_hex("739507391fa00da48a2ecae5df3b5e40b4432243603db6dafe33ca6b4966e357").unwrap().0) as *mut _,
        4450 => boxed(UInt256::from_hex("720ea2e4e7f6b31debe9bb852e1e4cdfdf10bed9827f0ef6527cfa0261010000").unwrap().0) as *mut _,
        _ => null_mut()
    }
}

#[test]
fn test_core_19_beta_6() {
    let chain = ChainType::DevNet(DevnetType::WhiteRussian);
    let processor = unsafe {
        register_processor(
            get_merkle_root_by_hash_default,
            get_block_height_by_hash_from_context,
            get_block_hash_by_height_from_context,
            get_llmq_snapshot_by_block_hash_default,
            save_llmq_snapshot_in_cache,
            get_masternode_list_mojito,
            masternode_list_save_in_cache,
            masternode_list_destroy_default,
            add_insight_lookup_default,
            hash_destroy_default,
            snapshot_destroy_default,
            should_process_diff_with_range_default,
        )
    };
    let cache = unsafe { &mut *processor_create_cache() };
    let context = &mut (FFIContext {
        chain,
        is_dip_0024: false,
        cache,
        blocks: vec![
            MerkleBlock::new(1, "9163d6958065ca5e73c36f0f2474ce618846260c215f5cba633bd0003585cb35", "dede0aec9516671ae39789b532e3bd08d7cf2f950d8559b963757578a19e65b6"),
            MerkleBlock::new(4765, "574e8d4a407fd54b9c0ec1ec0eb76e0a9ebc39cf4846356d8b32aab3e5000000", "888e68ec280bfbc012d7cce43e34faf790165f75e6c77ada040f20657e2c97db"),
        ]
    });
    let bytes = message_from_file("MNL_1_4765.dat");
    let result = unsafe { process_mnlistdiff_from_message(
        bytes.as_ptr(),
        bytes.len(),
        chain,
        false,
        true,
        70227,
        processor,
        context.cache,
        context as *mut _ as *mut std::ffi::c_void,
    )};
    let result = unsafe { *result };
    assert_diff_result(context, result);
}


#[test]
fn test_core_19_rc_2_testnet() {
    let chain = ChainType::TestNet;
    let processor = unsafe {
        register_processor(
            get_merkle_root_by_hash_default,
            get_block_height_by_hash_from_context,
            get_block_hash_by_height_from_context,
            get_llmq_snapshot_by_block_hash_default,
            save_llmq_snapshot_in_cache,
            get_masternode_list_mojito,
            masternode_list_save_in_cache,
            masternode_list_destroy_default,
            add_insight_lookup_default,
            hash_destroy_default,
            snapshot_destroy_default,
            should_process_diff_with_range_default,
        )
    };
    let cache = unsafe { &mut *processor_create_cache() };
    let context = &mut (FFIContext {
        chain,
        is_dip_0024: false,
        cache,
        blocks: init_testnet_store()
    });
    let bytes = message_from_file("MNL_TESTNET_CORE_19.dat");
    let result = unsafe { process_mnlistdiff_from_message(
        bytes.as_ptr(),
        bytes.len(),
        chain,
        false,
        true,
        70223,
        processor,
        context.cache,
        context as *mut _ as *mut std::ffi::c_void,
    )};
    let result = unsafe { *result };
    println!("Result: {:#?}", &result);
    // todo: need add new blocks to the testnet store
    //assert_diff_result(context, result);
}

#[test]
fn test_legacy_basic_conversion() {
    let chain_type = ChainType::TestNet;
    let block_height = 530000;
    let legacy_key = OperatorPublicKey {
        data: UInt384::from_hex("16ca29d03ef4897a22fe467bb58c52448c63bb29534502305e8ff142ac03907fae0851ff2528e4878ef51bfa3d5a1f22").unwrap(),
        version: 0
    };
    let basic_key = OperatorPublicKey {
        data: UInt384::from_hex("96ca29d03ef4897a22fe467bb58c52448c63bb29534502305e8ff142ac03907fae0851ff2528e4878ef51bfa3d5a1f22").unwrap(),
        version: 2,
    };
    assert_eq!(UInt384(*G1Element::from_bytes(&basic_key.data.0).unwrap().serialize_legacy()), legacy_key.data);

    let bk1 = UInt384::from_hex("981ab9848a9eba75643cde7f3ae8c2d3ba1efe36ba9dbbd2162437780f35493f9ed327220a5a0e60d5ae2793f5a75525").unwrap();
    println!("{}", UInt384(*G1Element::from_bytes(&bk1.0).unwrap().serialize_legacy()));
    // println!("{}", UInt384(*G1Element::from_bytes(&bk1.0).unwrap().serialize()));
    // println!("{}", UInt384(*G1Element::from_bytes_legacy(&bk1.0).unwrap().serialize()));
    println!("{}", UInt384(*G1Element::from_bytes_legacy(&bk1.0).unwrap().serialize_legacy()));

    let bk1 = UInt384::from_hex("158367af44572fbd35b475ca6259e1c499eefcbd5573ded52917c45cd2c8a0aa2e4ac9fd25ecdf1ef548750d2caf3ee3").unwrap();
    // println!("{}", UInt384(*G1Element::from_bytes(&bk1.0).unwrap().serialize_legacy()));
    // println!("{}", UInt384(*G1Element::from_bytes(&bk1.0).unwrap().serialize()));
    println!("{}", UInt384(*G1Element::from_bytes_legacy(&bk1.0).unwrap().serialize()));
    println!("{}", UInt384(*G1Element::from_bytes_legacy(&bk1.0).unwrap().serialize_legacy()));


}
