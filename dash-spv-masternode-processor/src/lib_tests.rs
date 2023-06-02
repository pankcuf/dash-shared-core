#[cfg(test)]
#[warn(clippy::manual_map)]
pub mod tests {
    extern crate libc;
    extern crate reqwest;
    use byte::BytesExt;
    use hashes::hex::{FromHex, ToHex};
    use serde::{Deserialize, Serialize};
    use std::io::Read;
    use std::ptr::null_mut;
    use std::{env, fs, slice};
    use crate::bindings::common::{processor_create_cache, register_processor, register_rust_logger};
    use crate::bindings::masternode::{process_mnlistdiff_from_message, process_qrinfo_from_message};
    use crate::ffi::boxer::boxed;
    use crate::ffi::from::FromFFI;
    use crate::ffi::to::ToFFI;
    use crate::ffi::unboxer::unbox_any;
    use crate::chain::common::chain_type::{ChainType, IHaveChainSettings};
    use crate::consensus::encode;
    use crate::crypto::byte_util::{BytesDecodable, Reversable, UInt256, UInt384};
    use crate::models;
    use crate::processing::{MasternodeProcessorCache, MasternodeProcessor, MNListDiffResult, ProcessingError, QRInfoResult};
    use crate::{unwrap_or_diff_processing_failure, unwrap_or_qr_processing_failure, unwrap_or_return, types};
    use crate::tests::block_store::{init_mainnet_store, init_testnet_store};

    // This regex can be used to omit timestamp etc. while replacing after paste from xcode console log
    // So it's bascically cut off such an expression "2022-09-11 15:31:59.445343+0300 DashSync_Example[41749:2762015]"
    // (\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2}):(\d{2}).(\d{6})\+(\d{4}) DashSync_Example\[(\d{5}):(\d{7})\]

    // This regex + replace can be used to transform string like
    // "000000000000001b33b86b6a167d37e3fcc6ba53e02df3cb06e3f272bb89dd7d" => 1092744,
    // into string like
    // ("0000013c21c2dc49704656ffc5adfd9c58506ac4c9556391d6f2d3d8db579233", 796617,),
    // which is very handy
    // ("[0-9A-Fa-f]{64}") => (\d+,)
    // ($1, $2),

    #[derive(Debug)]
    pub struct FFIContext<'a> {
        pub chain: ChainType,
        pub is_dip_0024: bool,
        pub cache: &'a mut MasternodeProcessorCache,
        // TODO:: make it initialized from json file with blocks
        pub blocks: Vec<MerkleBlock>,
    }

    impl<'a> FFIContext<'a> {
        pub fn block_for_hash(&self, hash: UInt256) -> Option<&MerkleBlock> {
            self.blocks.iter().find(|block| block.hash == hash)
        }
        pub fn block_for_height(&self, height: u32) -> Option<&MerkleBlock> {
            self.blocks.iter().find(|block| block.height == height)
        }

        pub fn genesis_as_ptr(&self) -> *const u8 {
            self.chain.genesis_hash().0.as_ptr()
        }
    }

    #[derive(Debug, Copy, Clone)]
    pub struct MerkleBlock {
        pub hash: UInt256,
        pub height: u32,
        pub merkleroot: UInt256,
    }

    impl MerkleBlock {
        pub fn new(height: u32, hash: &str, merkle_root: &str) -> MerkleBlock {
            MerkleBlock {
                height,
                hash: UInt256::from_hex(hash).unwrap(),
                merkleroot: if merkle_root.is_empty() { UInt256::MIN } else { UInt256::from_hex(merkle_root).unwrap() } }
        }

        pub fn reversed(height: u32, hash: &str, merkle_root: &str) -> MerkleBlock {
            MerkleBlock {
                height,
                hash: UInt256::from_hex(hash).unwrap().reverse(),
                merkleroot: UInt256::from_hex(merkle_root).unwrap_or(UInt256::MIN)
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    struct Block {
        pub hash: String,
        pub size: i64,
        pub height: i64,
        pub version: i64,
        pub merkleroot: String,
        pub tx: Vec<String>,
        pub time: i64,
        pub nonce: i64,
        pub bits: String,
        pub difficulty: f64,
        pub chainwork: String,
        pub confirmations: i64,
        pub previousblockhash: String,
        pub nextblockhash: String,
        pub reward: String,
        #[serde(rename = "isMainChain")]
        pub is_main_chain: bool,
        #[serde(rename = "poolInfo")]
        pub pool_info: PoolInfo,
    }
    #[derive(Serialize, Deserialize)]
    struct PoolInfo {}

    pub struct AggregationInfo {
        pub public_key: UInt384,
        pub version: u16,
        pub digest: UInt256,
    }
    pub fn get_block_from_insight_by_hash(hash: UInt256) -> Option<MerkleBlock> {
        let path = format!("https://testnet-insight.dashevo.org/insight-api-dash/block/{}", hash.reversed().0.to_hex().as_str());
        request_block(path)
    }
    pub fn get_block_from_insight_by_height(height: u32) -> Option<MerkleBlock> {
        let path = format!("https://testnet-insight.dashevo.org/insight-api-dash/block/{}", height);
        request_block(path)
    }

    pub fn request_block(path: String) -> Option<MerkleBlock> {
        println!("request_block: {}", path.as_str());
        match reqwest::blocking::get(path.as_str()) {
            Ok(response) => match response.json::<serde_json::Value>() {
                Ok(json) => {
                    let block: Block = serde_json::from_value(json).unwrap();
                    let merkle_block = MerkleBlock {
                        hash: UInt256::from_hex(block.hash.as_str()).unwrap().reverse(),
                        height: block.height as u32,
                        merkleroot: UInt256::from_hex(block.merkleroot.as_str()).unwrap()
                    };
                    println!("request_block: {}", path.as_str());
                    Some(merkle_block)
                },
                Err(err) => {
                    println!("{}", err);
                    None
                },
            },
            Err(err) => {
                println!("{}", err);
                None
            },
        }
    }

    /// This is convenience Core v0.17 method for use in tests which doesn't involve cross-FFI calls
    pub fn process_mnlistdiff_from_message_internal(
        message_arr: *const u8,
        message_length: usize,
        chain_type: ChainType,
        use_insight_as_backup: bool,
        protocol_version: u32,
        // genesis_hash: *const u8,
        processor: *mut MasternodeProcessor,
        cache: *mut MasternodeProcessorCache,
        context: *const std::ffi::c_void,
    ) -> MNListDiffResult {
        let processor = unsafe { &mut *processor };
        let cache = unsafe { &mut *cache };
        println!(
            "process_mnlistdiff_from_message_internal.start: {:?}",
            std::time::Instant::now()
        );
        processor.opaque_context = context;
        processor.use_insight_as_backup = use_insight_as_backup;
        processor.chain_type = chain_type;
        let message: &[u8] = unsafe { slice::from_raw_parts(message_arr, message_length as usize) };
        let list_diff =
            unwrap_or_diff_processing_failure!(models::MNListDiff::new(message, &mut 0, |hash| processor.lookup_block_height_by_hash(hash), protocol_version));
        let result = processor.get_list_diff_result_internal_with_base_lookup(list_diff, true, false, false, cache);
        println!(
            "process_mnlistdiff_from_message_internal.finish: {:?} {:#?}",
            std::time::Instant::now(),
            result
        );
        result
    }

    /// This is convenience Core v0.18 method for use in tests which doesn't involve cross-FFI calls
    pub fn process_qrinfo_from_message_internal(
        message: *const u8,
        message_length: usize,
        chain_type: ChainType,
        use_insight_as_backup: bool,
        is_rotated_quorums_presented: bool,
        protocol_version: u32,
        processor: *mut MasternodeProcessor,
        cache: *mut MasternodeProcessorCache,
        context: *const std::ffi::c_void,
    ) -> QRInfoResult {
        println!("process_qrinfo_from_message: {:?} {:?}", processor, cache);
        let message: &[u8] = unsafe { slice::from_raw_parts(message, message_length as usize) };
        let processor = unsafe { &mut *processor };
        processor.opaque_context = context;
        processor.use_insight_as_backup = use_insight_as_backup;
        processor.chain_type = chain_type;
        let cache = unsafe { &mut *cache };
        println!(
            "process_qrinfo_from_message --: {:?} {:?} {:?}",
            processor, processor.opaque_context, cache
        );
        let offset = &mut 0;
        let read_list_diff =
            |offset: &mut usize| processor.read_list_diff_from_message(message, offset, protocol_version);
        let mut process_list_diff = |list_diff: models::MNListDiff, should_process_quorums: bool| {
            processor.get_list_diff_result_internal_with_base_lookup(list_diff, should_process_quorums, true, is_rotated_quorums_presented, cache)
        };
        let read_snapshot = |offset: &mut usize| models::LLMQSnapshot::from_bytes(message, offset);
        let read_var_int = |offset: &mut usize| encode::VarInt::from_bytes(message, offset);
        let snapshot_at_h_c = unwrap_or_qr_processing_failure!(read_snapshot(offset));
        let snapshot_at_h_2c = unwrap_or_qr_processing_failure!(read_snapshot(offset));
        let snapshot_at_h_3c = unwrap_or_qr_processing_failure!(read_snapshot(offset));
        let diff_tip = unwrap_or_qr_processing_failure!(read_list_diff(offset));
        let diff_h = unwrap_or_qr_processing_failure!(read_list_diff(offset));
        let diff_h_c = unwrap_or_qr_processing_failure!(read_list_diff(offset));
        let diff_h_2c = unwrap_or_qr_processing_failure!(read_list_diff(offset));
        let diff_h_3c = unwrap_or_qr_processing_failure!(read_list_diff(offset));
        let extra_share = message.read_with::<bool>(offset, ()).unwrap_or(false);
        let (snapshot_at_h_4c, diff_h_4c) = if extra_share {
            (
                Some(unwrap_or_qr_processing_failure!(read_snapshot(offset))),
                Some(unwrap_or_qr_processing_failure!(read_list_diff(offset))),
            )
        } else {
            (None, None)
        };
        processor.save_snapshot(diff_h_c.block_hash, snapshot_at_h_c.clone());
        processor.save_snapshot(diff_h_2c.block_hash, snapshot_at_h_2c.clone());
        processor.save_snapshot(diff_h_3c.block_hash, snapshot_at_h_3c.clone());
        if extra_share {
            processor.save_snapshot(
                diff_h_4c.as_ref().unwrap().block_hash,
                snapshot_at_h_4c.as_ref().unwrap().clone(),
            );
        }
        let last_quorum_per_index_count =
            unwrap_or_qr_processing_failure!(read_var_int(offset)).0 as usize;
        let mut last_quorum_per_index: Vec<models::LLMQEntry> =
            Vec::with_capacity(last_quorum_per_index_count);
        for _i in 0..last_quorum_per_index_count {
            let entry = unwrap_or_qr_processing_failure!(models::LLMQEntry::from_bytes(
                message, offset
            ));
            last_quorum_per_index.push(entry);
        }
        let quorum_snapshot_list_count =
            unwrap_or_qr_processing_failure!(read_var_int(offset)).0 as usize;
        let mut quorum_snapshot_list: Vec<models::LLMQSnapshot> =
            Vec::with_capacity(quorum_snapshot_list_count);
        for _i in 0..quorum_snapshot_list_count {
            quorum_snapshot_list.push(unwrap_or_qr_processing_failure!(read_snapshot(offset)));
        }
        let mn_list_diff_list_count =
            unwrap_or_qr_processing_failure!(read_var_int(offset)).0 as usize;
        let mut mn_list_diff_list: Vec<MNListDiffResult> =
            Vec::with_capacity(mn_list_diff_list_count);
        for _i in 0..mn_list_diff_list_count {
            mn_list_diff_list.push(process_list_diff(unwrap_or_qr_processing_failure!(
                read_list_diff(offset)
            ), true));
        }
        // The order is important since the each new one dependent on previous
        #[allow(clippy::manual_map)]
        let result_at_h_4c = if let Some(diff) = diff_h_4c {
            Some(process_list_diff(diff, false))
        } else {
            None
        };
        let result_at_h_3c = process_list_diff(diff_h_3c, false);
        let result_at_h_2c = process_list_diff(diff_h_2c, false);
        let result_at_h_c = process_list_diff(diff_h_c, false);
        let result_at_h = process_list_diff(diff_h, true);
        let result_at_tip = process_list_diff(diff_tip, false);
        QRInfoResult {
            error_status: ProcessingError::None,
            result_at_tip,
            result_at_h,
            result_at_h_c,
            result_at_h_2c,
            result_at_h_3c,
            result_at_h_4c,
            snapshot_at_h_c,
            snapshot_at_h_2c,
            snapshot_at_h_3c,
            snapshot_at_h_4c,
            extra_share,
            last_quorum_per_index,
            quorum_snapshot_list,
            mn_list_diff_list,
        }
    }

    pub fn get_file_as_byte_vec(filename: &str) -> Vec<u8> {
        //println!("get_file_as_byte_vec: {}", filename);
        let mut f = fs::File::open(&filename).expect("no file found");
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read_exact(&mut buffer).expect("buffer overflow");
        buffer
    }

    pub fn register_cache<'a>() -> &'a mut MasternodeProcessorCache {
        let cache = unsafe { &mut *processor_create_cache() };
        cache
    }

    pub fn create_default_context(chain: ChainType, is_dip_0024: bool, cache: &mut MasternodeProcessorCache) -> FFIContext {
        let blocks = match chain {
            ChainType::MainNet => init_mainnet_store(),
            ChainType::TestNet => init_testnet_store(),
            _ => vec![],
        };
        FFIContext { chain, is_dip_0024, cache, blocks }
    }

    pub fn register_logger() {
        unsafe { register_rust_logger(); }
    }

    pub fn register_default_processor() -> *mut MasternodeProcessor {
        unsafe {
            register_processor(
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
        }
    }

    pub fn process_mnlistdiff(bytes: Vec<u8>, processor: *mut MasternodeProcessor, context: &mut FFIContext, version: u32, use_insight: bool, is_from_snapshot: bool) -> types::MNListDiffResult {
        unsafe {
            *process_mnlistdiff_from_message(
                bytes.as_ptr(),
                bytes.len(),
                context.chain,
                use_insight,
                is_from_snapshot,
                version,
                processor,
                context.cache,
                context as *mut _ as *mut std::ffi::c_void,
            )
        }
    }

    pub fn process_qrinfo(bytes: Vec<u8>, processor: *mut MasternodeProcessor, context: &mut FFIContext, version: u32, use_insight: bool, is_from_snapshot: bool) -> types::QRInfoResult {
        unsafe {
            *process_qrinfo_from_message(
                bytes.as_ptr(),
                bytes.len(),
                context.chain,
                use_insight,
                is_from_snapshot,
                true,
                version,
                processor,
                context.cache,
                context as *mut _ as *mut std::ffi::c_void,
            )
        }
    }

    pub fn message_from_file(name: &str) -> Vec<u8> {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let filepath = format!("{}/files/{}", crate_dir, name);
        println!("{:?}", filepath);
        get_file_as_byte_vec(&filepath)
    }

    pub fn assert_diff_result(context: &mut FFIContext, result: types::MNListDiffResult) {
        let masternode_list = unsafe { (*result.masternode_list).decode() };
        //print!("block_hash: {} ({})", masternode_list.block_hash, masternode_list.block_hash.reversed());
        let bh = context.block_for_hash(masternode_list.block_hash).unwrap().height;
        assert!(result.has_found_coinbase, "has no coinbase {}", bh);
        //turned off on purpose as we don't have the coinbase block
        //assert!(result.has_valid_coinbase, "Coinbase not valid at height {}", bh);
        assert!(result.has_valid_mn_list_root, "invalid mnl root {}", bh);
        assert!(result.has_valid_llmq_list_root, "invalid llmq root {}", bh);
        assert!(result.has_valid_quorums, "has invalid llmq height {}", bh);
        println!("Diff is ok at {}", bh);
    }

    pub fn assert_qrinfo_result(context: &mut FFIContext, result: types::QRInfoResult) {
        if result.mn_list_diff_list_count > 0 {
            let diff_result = unsafe { **result.mn_list_diff_list };
            assert_diff_result(context, diff_result);
        }
        if result.extra_share {
            assert_diff_result(context, unsafe { *result.result_at_h_4c });
        }
        assert_diff_result(context, unsafe { *result.result_at_h_3c });
        assert_diff_result(context, unsafe { *result.result_at_h_2c });
        assert_diff_result(context, unsafe { *result.result_at_h_c });
        assert_diff_result(context, unsafe { *result.result_at_h });
        assert_diff_result(context, unsafe { *result.result_at_tip });
    }

    pub unsafe extern "C" fn get_block_height_by_hash_from_context(
        block_hash: *mut [u8; 32],
        context: *const std::ffi::c_void,
    ) -> u32 {
        let data: &mut FFIContext = &mut *(context as *mut FFIContext);
        let block_hash = UInt256(*block_hash);
        let block_hash_reversed = block_hash.reversed();
        let block = data.block_for_hash(block_hash).unwrap_or(&MerkleBlock { hash: UInt256::MIN, height: u32::MAX, merkleroot: UInt256::MIN });
        let height = block.height;
        // println!("get_block_height_by_hash_from_context {}: {} ({})", height, block_hash_reversed, block_hash);
        if height == u32::MAX {
            println!("{}: {},", height, block_hash_reversed);
        }
        height
    }

    pub unsafe extern "C" fn get_block_hash_by_height_default(
        _block_height: u32,
        _context: *const std::ffi::c_void,
    ) -> *mut u8 {
        null_mut()
    }

    pub unsafe extern "C" fn get_block_hash_by_height_from_context(
        block_height: u32,
        context: *const std::ffi::c_void,
    ) -> *mut u8 {
        let data: &mut FFIContext = &mut *(context as *mut FFIContext);
        if let Some(block) = data.block_for_height(block_height) {
            let block_hash = block.hash;
            // println!("get_block_hash_by_height_from_context: {}: {:?}", block_height, block_hash.clone().reversed());
            boxed(block_hash.0) as *mut _
        } else {
            null_mut()
        }
    }

    pub unsafe extern "C" fn get_llmq_snapshot_by_block_height_default(
        _block_height: u32,
        _context: *const std::ffi::c_void,
    ) -> *mut types::LLMQSnapshot {
        null_mut()
    }

    pub unsafe extern "C" fn get_llmq_snapshot_by_block_hash_default(
        _block_hash: *mut [u8; 32],
        _context: *const std::ffi::c_void,
    ) -> *mut types::LLMQSnapshot {
        null_mut()
    }

    pub unsafe extern "C" fn get_llmq_snapshot_by_block_hash_from_context(
        block_hash: *mut [u8; 32],
        context: *const std::ffi::c_void,
    ) -> *mut types::LLMQSnapshot {
        let h = UInt256(*(block_hash));
        let data: &mut FFIContext = &mut *(context as *mut FFIContext);
        if let Some(snapshot) = data.cache.llmq_snapshots.get(&h) {
            //println!("get_llmq_snapshot_by_block_hash_from_context: {}: {:?}", h, snapshot);
            boxed(snapshot.encode())
        } else {
            null_mut()
        }
    }

    pub unsafe extern "C" fn get_masternode_list_by_block_hash_default(
        _block_hash: *mut [u8; 32],
        _context: *const std::ffi::c_void,
    ) -> *mut types::MasternodeList {
        null_mut()
    }

    pub unsafe extern "C" fn get_masternode_list_by_block_hash_from_cache(
        block_hash: *mut [u8; 32],
        context: *const std::ffi::c_void,
    ) -> *mut types::MasternodeList {
        let h = UInt256(*(block_hash));
        let data: &mut FFIContext = &mut *(context as *mut FFIContext);
        //println!("get_masternode_list_by_block_hash_from_cache: {}", h);
        if let Some(list) = data.cache.mn_lists.get(&h) {
            // println!("get_masternode_list_by_block_hash_from_cache: {}: masternodes: {} quorums: {} mn_merkle_root: {:?}, llmq_merkle_root: {:?}", h, list.masternodes.len(), list.quorums.len(), list.masternode_merkle_root, list.llmq_merkle_root);
            let encoded = list.encode();
            // &encoded as *const types::MasternodeList
            boxed(encoded)
        } else {
            println!("missing list: {}: {},", get_block_height_by_hash_from_context(block_hash, context), h.reversed());
            null_mut()
        }
    }

    pub unsafe extern "C" fn masternode_list_save_default(
        _block_hash: *mut [u8; 32],
        _masternode_list: *mut types::MasternodeList,
        _context: *const std::ffi::c_void,
    ) -> bool {
        true
    }
    pub unsafe extern "C" fn masternode_list_save_in_cache(
        block_hash: *mut [u8; 32],
        masternode_list: *mut types::MasternodeList,
        context: *const std::ffi::c_void,
    ) -> bool {
        let h = UInt256(*(block_hash));
        let data: &mut FFIContext = &mut *(context as *mut FFIContext);
        let masternode_list = *masternode_list;
        let masternode_list_decoded = masternode_list.decode();
        //println!("masternode_list_save_in_cache: {}", h);
        data.cache.mn_lists.insert(h, masternode_list_decoded);
        true
    }

    pub unsafe extern "C" fn masternode_list_destroy_default(
        _masternode_list: *mut types::MasternodeList,
    ) {
    }
    pub unsafe extern "C" fn hash_destroy_default(_hash: *mut u8) {}

    pub unsafe extern "C" fn should_process_diff_with_range_default(
        base_block_hash: *mut [u8; 32],
        block_hash: *mut [u8; 32],
        context: *const std::ffi::c_void,
    ) -> ProcessingError {
        ProcessingError::None
    }
    pub unsafe extern "C" fn snapshot_destroy_default(_snapshot: *mut types::LLMQSnapshot) {}
    pub unsafe extern "C" fn add_insight_lookup_default(
        _hash: *mut [u8; 32],
        _context: *const std::ffi::c_void,
    ) {
    }
    pub unsafe extern "C" fn save_llmq_snapshot_default(
        block_hash: *mut [u8; 32],
        snapshot: *mut types::LLMQSnapshot,
        _context: *const std::ffi::c_void,
    ) -> bool {
        true
    }
    pub unsafe extern "C" fn save_llmq_snapshot_in_cache(
        block_hash: *mut [u8; 32],
        snapshot: *mut types::LLMQSnapshot,
        context: *const std::ffi::c_void,
    ) -> bool {
        let h = UInt256(*(block_hash));
        let data: &mut FFIContext = &mut *(context as *mut FFIContext);
        data.cache.add_snapshot(h, (*snapshot).decode());
        true
    }

    pub unsafe extern "C" fn get_merkle_root_by_hash_default(
        block_hash: *mut [u8; 32],
        context: *const std::ffi::c_void,
    ) -> *mut u8 {
        let block_hash = UInt256(*block_hash);
        let data: &mut FFIContext = &mut *(context as *mut FFIContext);
        let block_hash_reversed = block_hash.reversed().0.to_hex();
        let merkle_root = if let Some(block) = data.block_for_hash(block_hash) {
            block.merkleroot.reversed()
        } else {
            UInt256::from_hex("0000000000000000000000000000000000000000000000000000000000000000").unwrap()
        };
        //println!("get_merkle_root_by_hash_default {} ({}) => ({})", block_hash, block_hash_reversed, merkle_root);
        boxed(merkle_root.0) as *mut _
    }

    pub unsafe extern "C" fn get_block_hash_by_height_from_insight(block_height: u32, context: *const std::ffi::c_void) -> *mut u8 {
        let data: &mut FFIContext = &mut *(context as *mut FFIContext);
        match data.blocks.iter().find(|block| block.height == block_height) {
            Some(block) => boxed(block.hash.0) as *mut _,
            None => match get_block_from_insight_by_height(block_height) {
                Some(block) => {
                    data.blocks.push(block);
                    boxed(block.hash.0) as *mut _
                },
                None => null_mut()
            }
        }
    }

    pub unsafe extern "C" fn get_block_height_by_hash_from_insight(block_hash: *mut [u8; 32], context: *const std::ffi::c_void) -> u32 {
        let data: &mut FFIContext = &mut *(context as *mut FFIContext);
        let hash = UInt256(*block_hash);
        match data.blocks.iter().find(|block| block.hash == hash) {
            Some(block) => block.height,
            None => match get_block_from_insight_by_hash(hash) {
                Some(block) => {
                    data.blocks.push(block);
                    block.height
                }
                None => u32::MAX
            }
        }
    }

    pub unsafe extern "C" fn get_merkle_root_by_hash_from_insight(block_hash: *mut [u8; 32], context: *const std::ffi::c_void) -> *mut u8 {
        let data: &mut FFIContext = &mut *(context as *mut FFIContext);
        let hash = UInt256(*block_hash);
        match data.blocks.iter().find(|block| block.hash == hash) {
            Some(block) => boxed(block.merkleroot.reversed().0) as *mut _,
            None => match get_block_from_insight_by_hash(hash) {
                Some(block) => {
                    data.blocks.push(block);
                    boxed(block.merkleroot.reversed().0) as *mut _
                },
                None => boxed(UInt256::MIN.0) as *mut _
            }
        }
    }

    pub fn perform_mnlist_diff_test_for_message(
        hex_string: &str,
        should_be_total_transactions: u32,
        verify_string_hashes: Vec<&str>,
        verify_string_smle_hashes: Vec<&str>,
    ) {
        let bytes = Vec::from_hex(hex_string).unwrap();
        let length = bytes.len();
        let c_array = bytes.as_ptr();
        let message: &[u8] = unsafe { slice::from_raw_parts(c_array, length) };
        let chain = ChainType::TestNet;
        let offset = &mut 0;
        assert!(length - *offset >= 32);
        let base_block_hash = UInt256::from_bytes(message, offset).unwrap();
        assert_ne!(
            base_block_hash,
            UInt256::default(), /*UINT256_ZERO*/
            "Base block hash should NOT be empty here"
        );
        assert!(length - *offset >= 32);
        let _block_hash = UInt256::from_bytes(message, offset).unwrap();
        assert!(length - *offset >= 4);
        let total_transactions = u32::from_bytes(message, offset).unwrap();
        assert_eq!(
            total_transactions, should_be_total_transactions,
            "Invalid transaction count"
        );
        let use_insight_as_backup = false;
        let base_masternode_list_hash: *const u8 = null_mut();
        let context = &mut FFIContext {
            chain,
            is_dip_0024: false,
            cache: &mut MasternodeProcessorCache::default(),
            blocks: init_testnet_store()
        } as *mut _ as *mut std::ffi::c_void;

        let cache = unsafe { processor_create_cache() };
        let processor = unsafe {
            register_processor(
                get_merkle_root_by_hash_default,
                get_block_height_by_hash_from_context,
                get_block_hash_by_height_default,
                get_llmq_snapshot_by_block_hash_default,
                save_llmq_snapshot_default,
                get_masternode_list_by_block_hash_default,
                masternode_list_save_default,
                masternode_list_destroy_default,
                add_insight_lookup_default,
                hash_destroy_default,
                snapshot_destroy_default,
                should_process_diff_with_range_default,
            )
        };

        let result = unsafe { process_mnlistdiff_from_message(
            c_array,
            length,
            chain,
            use_insight_as_backup,
            false,
            70221,
            processor,
            cache,
            context,
        )};
        println!("result: {:?}", result);
        let result = unsafe { unbox_any(result) };
        let masternode_list = unsafe { (*unbox_any(result.masternode_list)).decode() };
        let masternodes = masternode_list.masternodes;
        let mut pro_tx_hashes: Vec<UInt256> = masternodes.clone().into_keys().collect();
        pro_tx_hashes.sort();
        let mut verify_hashes: Vec<UInt256> = verify_string_hashes
            .into_iter()
            .map(|h| {
                Vec::from_hex(h)
                    .unwrap()
                    .read_with::<UInt256>(&mut 0, byte::LE)
                    .unwrap()
                    .reverse()
            })
            .collect();
        verify_hashes.sort();
        assert_eq!(verify_hashes, pro_tx_hashes, "Provider transaction hashes");
        let mut masternode_list_hashes: Vec<UInt256> = pro_tx_hashes
            .clone()
            .iter()
            .map(|hash| masternodes[hash].entry_hash)
            .collect();
        masternode_list_hashes.sort();
        let mut verify_smle_hashes: Vec<UInt256> = verify_string_smle_hashes
            .into_iter()
            .map(|h| {
                Vec::from_hex(h)
                    .unwrap()
                    .read_with::<UInt256>(&mut 0, byte::LE)
                    .unwrap()
            })
            .collect();
        verify_smle_hashes.sort();
        assert_eq!(
            masternode_list_hashes, verify_smle_hashes,
            "SMLE transaction hashes"
        );
        assert!(
            result.has_found_coinbase,
            "The coinbase was not part of provided hashes"
        );
    }
}
