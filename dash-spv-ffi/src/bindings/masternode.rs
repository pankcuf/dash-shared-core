use std::ptr::null_mut;
use std::slice;
use byte::BytesExt;
use dash_spv_masternode_processor::{models, types};
use dash_spv_masternode_processor::chain::common::{ChainType, IHaveChainSettings, LLMQType};
use dash_spv_masternode_processor::consensus::encode;
use dash_spv_masternode_processor::crypto::{UInt256, byte_util::{BytesDecodable, ConstDecodable}};
use dash_spv_masternode_processor::processing::{MasternodeProcessor, MasternodeProcessorCache, ProcessingError};
use crate::ffi::{boxer::{boxed, boxed_vec}, ByteArray, from::FromFFI, to::ToFFI};
use crate::{unwrap_or_failure, unwrap_or_qr_result_failure};

/// Read and process message received as a response for 'GETMNLISTDIFF' call
/// Here we calculate quorums according to Core v0.17
/// See https://github.com/dashpay/dips/blob/master/dip-0004.md
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn process_mnlistdiff_from_message(
    message_arr: *const u8,
    message_length: usize,
    chain_type: ChainType,
    use_insight_as_backup: bool,
    is_from_snapshot: bool,
    protocol_version: u32,
    processor: *mut MasternodeProcessor,
    cache: *mut MasternodeProcessorCache,
    context: *const std::ffi::c_void,
) -> *mut types::MNListDiffResult {
    let instant = std::time::Instant::now();
    let processor = &mut *processor;
    let cache = &mut *cache;
    println!("process_mnlistdiff_from_message -> {:?} {:p} {:p} {:p}", instant, processor, cache, context);
    processor.opaque_context = context;
    processor.use_insight_as_backup = use_insight_as_backup;
    processor.chain_type = chain_type;
    let message: &[u8] = slice::from_raw_parts(message_arr, message_length as usize);
    let list_diff = unwrap_or_failure!(models::MNListDiff::new(message, &mut 0, |hash| processor
        .lookup_block_height_by_hash(hash), protocol_version));
    if !is_from_snapshot {
        let error = processor
            .should_process_diff_with_range(list_diff.base_block_hash, list_diff.block_hash);
        if error != ProcessingError::None {
            println!("process_mnlistdiff_from_message <- {:?} ms [{:?}]", instant.elapsed().as_millis(), error);
            return boxed(types::MNListDiffResult::default_with_error(error));
        }
    }
    let result = processor.get_list_diff_result_with_base_lookup(list_diff, true, false, false, cache);
    println!("process_mnlistdiff_from_message <- {:?} ms", instant.elapsed().as_millis());
    boxed(result)
}

/// Here we read & calculate quorums according to Core v0.18
/// See https://github.com/dashpay/dips/blob/master/dip-0024.md
/// The reason behind we have multiple methods for this is that:
/// in objc we need 2 separate calls to incorporate additional logics between reading and processing
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn process_qrinfo_from_message(
    message: *const u8,
    message_length: usize,
    chain_type: ChainType,
    use_insight_as_backup: bool,
    is_from_snapshot: bool,
    is_rotated_quorums_presented: bool,
    protocol_version: u32,
    processor: *mut MasternodeProcessor,
    cache: *mut MasternodeProcessorCache,
    context: *const std::ffi::c_void,
) -> *mut types::QRInfoResult {
    let instant = std::time::Instant::now();
    let message: &[u8] = slice::from_raw_parts(message, message_length as usize);
    let processor = &mut *processor;
    let cache = &mut *cache;
    processor.opaque_context = context;
    processor.use_insight_as_backup = use_insight_as_backup;
    processor.chain_type = chain_type;
    println!( "process_qrinfo_from_message -> {:?} {:p} {:p} {:p}", instant, processor, cache, context);
    let offset = &mut 0;
    let mut process_list_diff = |list_diff: models::MNListDiff, should_process_quorums: bool| {
        processor.get_list_diff_result_with_base_lookup(list_diff, should_process_quorums, true, is_rotated_quorums_presented, cache)
    };
    let read_list_diff =
        |offset: &mut usize| processor.read_list_diff_from_message(message, offset, protocol_version);
    let read_snapshot = |offset: &mut usize| models::LLMQSnapshot::from_bytes(message, offset);
    let read_var_int = |offset: &mut usize| encode::VarInt::from_bytes(message, offset);
    let mut get_list_diff_result =
        |list_diff: models::MNListDiff, verify_quorums: bool| boxed(process_list_diff(list_diff, verify_quorums));
    let snapshot_at_h_c = unwrap_or_qr_result_failure!(read_snapshot(offset));
    let snapshot_at_h_2c = unwrap_or_qr_result_failure!(read_snapshot(offset));
    let snapshot_at_h_3c = unwrap_or_qr_result_failure!(read_snapshot(offset));
    let diff_tip = unwrap_or_qr_result_failure!(read_list_diff(offset));
    if !is_from_snapshot {
        let error =
            processor.should_process_diff_with_range(diff_tip.base_block_hash, diff_tip.block_hash);
        if error != ProcessingError::None {
            println!("process_qrinfo_from_message <- {:?} ms [{:#?}]", instant.elapsed().as_millis(), error);
            return boxed(types::QRInfoResult::default_with_error(error));
        }
    }
    let diff_h = unwrap_or_qr_result_failure!(read_list_diff(offset));
    let diff_h_c = unwrap_or_qr_result_failure!(read_list_diff(offset));
    let diff_h_2c = unwrap_or_qr_result_failure!(read_list_diff(offset));
    let diff_h_3c = unwrap_or_qr_result_failure!(read_list_diff(offset));
    let extra_share = message.read_with::<bool>(offset, ()).unwrap_or(false);
    let snapshot_at_h_4c = extra_share.then_some(unwrap_or_qr_result_failure!(read_snapshot(offset)));
    let diff_h_4c = extra_share.then_some(unwrap_or_qr_result_failure!(read_list_diff(offset)));
    processor.save_snapshot(diff_h_c.block_hash, snapshot_at_h_c.clone());
    processor.save_snapshot(diff_h_2c.block_hash, snapshot_at_h_2c.clone());
    processor.save_snapshot(diff_h_3c.block_hash, snapshot_at_h_3c.clone());
    if extra_share {
        processor.save_snapshot(
            diff_h_4c.as_ref().unwrap().block_hash,
            snapshot_at_h_4c.clone().unwrap(),
        );
    }

    let last_quorum_per_index_count = unwrap_or_qr_result_failure!(read_var_int(offset)).0 as usize;
    let mut last_quorum_per_index_vec: Vec<*mut types::LLMQEntry> =
        Vec::with_capacity(last_quorum_per_index_count);
    for _i in 0..last_quorum_per_index_count {
        let quorum = unwrap_or_qr_result_failure!(models::LLMQEntry::from_bytes(message, offset));
        last_quorum_per_index_vec.push(boxed(quorum.encode()));
    }
    let quorum_snapshot_list_count = unwrap_or_qr_result_failure!(read_var_int(offset)).0 as usize;
    let mut quorum_snapshot_list_vec: Vec<*mut types::LLMQSnapshot> =
        Vec::with_capacity(quorum_snapshot_list_count);
    let mut snapshots: Vec<models::LLMQSnapshot> = Vec::with_capacity(quorum_snapshot_list_count);
    for _i in 0..quorum_snapshot_list_count {
        let snapshot = unwrap_or_qr_result_failure!(read_snapshot(offset));
        snapshots.push(snapshot);
    }
    let mn_list_diff_list_count = unwrap_or_qr_result_failure!(read_var_int(offset)).0 as usize;
    let mut mn_list_diff_list_vec: Vec<*mut types::MNListDiffResult> =
        Vec::with_capacity(mn_list_diff_list_count);
    assert_eq!(
        quorum_snapshot_list_count, mn_list_diff_list_count,
        "'quorum_snapshot_list_count' must be equal 'mn_list_diff_list_count'"
    );
    for i in 0..mn_list_diff_list_count {
        let list_diff = unwrap_or_qr_result_failure!(read_list_diff(offset));
        let block_hash = list_diff.block_hash;
        mn_list_diff_list_vec.push(get_list_diff_result(list_diff, false));
        let snapshot = snapshots.get(i).unwrap();
        quorum_snapshot_list_vec.push(boxed(snapshot.encode()));
        processor.save_snapshot(block_hash, snapshot.clone());
    }

    let result_at_h_4c = if extra_share {
        get_list_diff_result(diff_h_4c.unwrap(), false)
    } else {
        null_mut()
    };
    let result_at_h_3c = get_list_diff_result(diff_h_3c, false);
    let result_at_h_2c = get_list_diff_result(diff_h_2c, false);
    let result_at_h_c = get_list_diff_result(diff_h_c, false);
    let result_at_h = get_list_diff_result(diff_h, true);
    let result_at_tip = get_list_diff_result(diff_tip, false);
    let result = types::QRInfoResult {
        error_status: ProcessingError::None,
        result_at_tip,
        result_at_h,
        result_at_h_c,
        result_at_h_2c,
        result_at_h_3c,
        result_at_h_4c,
        snapshot_at_h_c: boxed(snapshot_at_h_c.encode()),
        snapshot_at_h_2c: boxed(snapshot_at_h_2c.encode()),
        snapshot_at_h_3c: boxed(snapshot_at_h_3c.encode()),
        snapshot_at_h_4c: if extra_share {
            boxed(snapshot_at_h_4c.unwrap().encode())
        } else {
            null_mut()
        },
        extra_share,
        last_quorum_per_index: boxed_vec(last_quorum_per_index_vec),
        last_quorum_per_index_count,
        quorum_snapshot_list: boxed_vec(quorum_snapshot_list_vec),
        quorum_snapshot_list_count,
        mn_list_diff_list: boxed_vec(mn_list_diff_list_vec),
        mn_list_diff_list_count,
    };

    #[cfg(feature = "generate-dashj-tests")]
    crate::util::java::generate_qr_state_test_file_json(chain_type, result);
    println!("process_qrinfo_from_message <- {:?} ms", instant.elapsed().as_millis());
    boxed(result)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn processor_cache_masternode_list(block_hash: *const u8, list: *const types::MasternodeList, cache: *mut MasternodeProcessorCache) {
    let hash = UInt256::from_const(block_hash).unwrap();
    let list = (*list).decode();
    (&mut *cache).mn_lists.insert(hash, list);
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn validate_masternode_list(list: *const types::MasternodeList, quorum: *const types::LLMQEntry, block_height: u32, chain_type: ChainType) -> bool {
    let list = (*list).decode();
    let mut quorum = (*quorum).decode();
    let is_valid_payload = quorum.validate_payload();
    if !is_valid_payload {
        return false;
    }
    let hpmn_only = quorum.llmq_type == chain_type.platform_type() && !quorum.version.use_bls_legacy();
    let valid_masternodes = models::MasternodeList::get_masternodes_for_quorum(quorum.llmq_type, list.masternodes, quorum.llmq_quorum_hash(), block_height, hpmn_only);
    return quorum.validate(valid_masternodes, block_height);
}


/// # Safety
#[no_mangle]
pub extern "C" fn quorum_size_for_type(llmq_type: LLMQType) -> u32 {
    llmq_type.size()
}

/// # Safety
#[no_mangle]
pub extern "C" fn quorum_threshold_for_type(llmq_type: LLMQType) -> u32 {
    llmq_type.threshold()
}

/// # Safety
#[no_mangle]
pub extern "C" fn quorum_build_llmq_hash(llmq_type: LLMQType, quorum_hash: *const u8) -> ByteArray {
    models::LLMQEntry::build_llmq_quorum_hash(llmq_type, UInt256::from_const(quorum_hash).unwrap()).into()
}

/// # Safety
#[no_mangle]
pub extern "C" fn masternode_hash_confirmed_hash(confirmed_hash: *const u8, pro_reg_tx_hash: *const u8) -> ByteArray {
    let confirmed_hash = UInt256::from_const(confirmed_hash).unwrap_or(UInt256::MIN);
    let pro_reg_tx_hash = UInt256::from_const(pro_reg_tx_hash).unwrap_or(UInt256::MIN);
    models::MasternodeEntry::hash_confirmed_hash(confirmed_hash, pro_reg_tx_hash).into()
}

/// # Safety
#[no_mangle]
pub extern "C" fn quorum_type_for_is_locks(chain_type: ChainType) -> LLMQType {
    chain_type.is_llmq_type()
}

/// # Safety
#[no_mangle]
pub extern "C" fn quorum_type_for_isd_locks(chain_type: ChainType) -> LLMQType {
    chain_type.isd_llmq_type()
}

/// # Safety
#[no_mangle]
pub extern "C" fn quorum_type_for_chain_locks(chain_type: ChainType) -> LLMQType {
    chain_type.chain_locks_type()
}

/// # Safety
#[no_mangle]
pub extern "C" fn quorum_type_for_platform(chain_type: ChainType) -> LLMQType {
    chain_type.platform_type()
}

/// # Safety
#[no_mangle]
pub extern "C" fn quorum_should_process_type_for_chain(llmq_type: LLMQType, chain_type: ChainType) -> bool {
    chain_type.should_process_llmq_of_type(llmq_type)
}
