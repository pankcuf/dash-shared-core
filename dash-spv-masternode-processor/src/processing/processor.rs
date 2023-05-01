use std::collections::BTreeMap;
use std::ptr::null;
use crate::{common, models, types};
use crate::chain::common::{ChainType, IHaveChainSettings, LLMQParams, LLMQType};
use crate::crypto::{byte_util::{Reversable, Zeroable}, UInt256};
use crate::ffi::boxer::boxed;
use crate::ffi::callbacks;
use crate::ffi::callbacks::{AddInsightBlockingLookup, GetBlockHashByHeight, GetBlockHeightByHash, GetLLMQSnapshotByBlockHash, HashDestroy, LLMQSnapshotDestroy, MasternodeListDestroy, MasternodeListLookup, MasternodeListSave, MerkleRootLookup, SaveLLMQSnapshot, ShouldProcessDiffWithRange};
use crate::ffi::to::ToFFI;
use crate::processing::{MasternodeProcessorCache, MNListDiffResult, ProcessingError};

// https://github.com/rust-lang/rfcs/issues/2770
#[repr(C)]
pub struct MasternodeProcessor {
    /// External Masternode Manager Diff Message Context
    pub opaque_context: *const std::ffi::c_void,
    pub chain_type: ChainType,
    pub use_insight_as_backup: bool,
    pub get_block_height_by_hash: GetBlockHeightByHash,
    pub get_merkle_root_by_hash: MerkleRootLookup,
    get_block_hash_by_height: GetBlockHashByHeight,
    get_llmq_snapshot_by_block_hash: GetLLMQSnapshotByBlockHash,
    save_llmq_snapshot: SaveLLMQSnapshot,
    get_masternode_list_by_block_hash: MasternodeListLookup,
    save_masternode_list: MasternodeListSave,
    destroy_masternode_list: MasternodeListDestroy,
    add_insight: AddInsightBlockingLookup,
    destroy_hash: HashDestroy,
    destroy_snapshot: LLMQSnapshotDestroy,
    should_process_diff_with_range: ShouldProcessDiffWithRange,
}
impl std::fmt::Debug for MasternodeProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MasternodeProcessor")
            .field("context", &self.opaque_context)
            .finish()
    }
}

impl MasternodeProcessor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        get_merkle_root_by_hash: MerkleRootLookup,
        get_block_height_by_hash: GetBlockHeightByHash,
        get_block_hash_by_height: GetBlockHashByHeight,
        get_llmq_snapshot_by_block_hash: GetLLMQSnapshotByBlockHash,
        save_llmq_snapshot: SaveLLMQSnapshot,
        get_masternode_list_by_block_hash: MasternodeListLookup,
        save_masternode_list: MasternodeListSave,
        destroy_masternode_list: MasternodeListDestroy,
        add_insight: AddInsightBlockingLookup,
        destroy_hash: HashDestroy,
        destroy_snapshot: LLMQSnapshotDestroy,
        should_process_diff_with_range: ShouldProcessDiffWithRange,
    ) -> Self {
        Self {
            get_merkle_root_by_hash,
            get_block_height_by_hash,
            get_block_hash_by_height,
            get_llmq_snapshot_by_block_hash,
            save_llmq_snapshot,
            get_masternode_list_by_block_hash,
            save_masternode_list,
            destroy_masternode_list,
            add_insight,
            destroy_hash,
            destroy_snapshot,
            should_process_diff_with_range,
            opaque_context: null(),
            chain_type: ChainType::MainNet,
            use_insight_as_backup: false,
        }
    }

    pub(crate) fn find_masternode_list(
        &self,
        block_hash: UInt256,
        cached_lists: &BTreeMap<UInt256, models::MasternodeList>,
        unknown_lists: &mut Vec<UInt256>,
    ) -> Option<models::MasternodeList> {
        let genesis_hash = self.chain_type.genesis_hash();
        if block_hash.is_zero() {
            // If it's a zero block we don't expect models list here
            // println!("find {}: {} EMPTY BLOCK HASH -> None", self.lookup_block_height_by_hash(block_hash), block_hash);
            None
        } else if block_hash.eq(&genesis_hash) {
            // If it's a genesis block we don't expect models list here
            // println!("find {}: {} It's a genesis -> Some(EMPTY MNL)", self.lookup_block_height_by_hash(block_hash), block_hash);
            Some(models::MasternodeList::new(BTreeMap::default(), BTreeMap::default(), block_hash, self.lookup_block_height_by_hash(block_hash), false))
            // None
        } else if let Some(cached) = cached_lists.get(&block_hash) {
            // Getting it from local cache stored as opaque in FFI context
            // println!("find_masternode_list (cache) {}: {} -> Some({:?})", self.lookup_block_height_by_hash(block_hash), block_hash, cached);
            Some(cached.clone())
        } else if let Some(looked) = self.lookup_masternode_list(block_hash) {
            // Getting it from FFI directly
            // println!("find_masternode_list {}: {} (ffi) -> Some({:?})", self.lookup_block_height_by_hash(block_hash), block_hash, looked);
            Some(looked)
        } else {
            // println!("find {}: {} Unknown -> None", self.lookup_block_height_by_hash(block_hash), block_hash);
            if self.lookup_block_height_by_hash(block_hash) != u32::MAX {
                unknown_lists.push(block_hash);
            } else if self.use_insight_as_backup {
                self.add_insight(block_hash);
                if self.lookup_block_height_by_hash(block_hash) != u32::MAX {
                    unknown_lists.push(block_hash);
                }
            }
            None
        }
    }
    pub(crate) fn find_snapshot(
        &self,
        block_hash: UInt256,
        cached_snapshots: &BTreeMap<UInt256, models::LLMQSnapshot>,
    ) -> Option<models::LLMQSnapshot> {
        if let Some(cached) = cached_snapshots.get(&block_hash) {
            // Getting it from local cache stored as opaque in FFI context
            Some(cached.clone())
        } else {
            self.lookup_snapshot_by_block_hash(block_hash)
        }
    }

    pub(crate) fn get_list_diff_result_with_base_lookup(
        &self,
        list_diff: models::MNListDiff,
        should_process_quorums: bool,
        is_dip_0024: bool,
        is_rotated_quorums_presented: bool,
        cache: &mut MasternodeProcessorCache,
    ) -> types::MNListDiffResult {
        let base_block_hash = list_diff.base_block_hash;
        let base_list = self.find_masternode_list(
            base_block_hash,
            &cache.mn_lists,
            &mut cache.needed_masternode_lists,
        );
        self.get_list_diff_result(base_list, list_diff, should_process_quorums, is_dip_0024, is_rotated_quorums_presented, cache)
    }

    pub(crate) fn get_list_diff_result_internal_with_base_lookup(
        &self,
        list_diff: models::MNListDiff,
        should_process_quorums: bool,
        is_dip_0024: bool,
        is_rotated_quorums_presented: bool,
        cache: &mut MasternodeProcessorCache,
    ) -> MNListDiffResult {
        let base_list = self.find_masternode_list(
            list_diff.base_block_hash,
            &cache.mn_lists,
            &mut cache.needed_masternode_lists,
        );
        self.get_list_diff_result_internal(base_list, list_diff, should_process_quorums, is_dip_0024, is_rotated_quorums_presented, cache)
    }

    pub(crate) fn get_list_diff_result(
        &self,
        base_list: Option<models::MasternodeList>,
        list_diff: models::MNListDiff,
        should_process_quorums: bool,
        is_dip_0024: bool,
        is_rotated_quorums_presented: bool,
        cache: &mut MasternodeProcessorCache,
    ) -> types::MNListDiffResult {
        let result = self.get_list_diff_result_internal(base_list, list_diff, should_process_quorums, is_dip_0024, is_rotated_quorums_presented, cache);
        // println!("get_list_diff_result: {:#?}", result);
        result.encode()
    }

    fn cache_masternode_list(
        &self,
        block_hash: UInt256,
        list: models::MasternodeList,
        cache: &mut MasternodeProcessorCache,
    ) {
        // It's good to cache lists to use it inside processing session
        // Here we use opaque-like pointer which we initiate on the C-side to sync its lifetime with runtime
        cache.add_masternode_list(block_hash, list);
        // Here we just store it in the C-side ()
        // self.save_masternode_list(block_hash, &masternode_list);
    }

    pub(crate) fn get_list_diff_result_internal(
        &self,
        base_list: Option<models::MasternodeList>,
        list_diff: models::MNListDiff,
        should_process_quorums: bool,
        is_dip_0024: bool,
        is_rotated_quorums_presented: bool,
        cache: &mut MasternodeProcessorCache,
    ) -> MNListDiffResult {
        let skip_removed_masternodes = list_diff.version >= 2;
        let base_block_hash = list_diff.base_block_hash;
        let block_hash = list_diff.block_hash;
        let block_height = list_diff.block_height;
        let (base_masternodes, base_quorums) = match base_list {
            Some(list) => (list.masternodes, list.quorums),
            None => (BTreeMap::new(), BTreeMap::new()),
        };
        let mut coinbase_transaction = list_diff.coinbase_transaction;
        let quorums_active = coinbase_transaction.coinbase_transaction_version >= 2;
        let (added_masternodes, modified_masternodes, masternodes) = self.classify_masternodes(
            base_masternodes,
            list_diff.added_or_modified_masternodes,
            list_diff.deleted_masternode_hashes,
            block_height,
            block_hash,
        );
        let (added_quorums, quorums, has_valid_quorums) = self.classify_quorums(
            base_quorums,
            list_diff.added_quorums,
            list_diff.deleted_quorums,
            should_process_quorums,
            skip_removed_masternodes,
            is_dip_0024,
            is_rotated_quorums_presented,
            cache,
        );
        let masternode_list = models::MasternodeList::new(
            masternodes,
            quorums,
            block_hash,
            block_height,
            quorums_active,
        );
        let merkle_tree = common::MerkleTree {
            tree_element_count: list_diff.total_transactions,
            hashes: list_diff.merkle_hashes,
            flags: list_diff.merkle_flags,
            hash_function: Default::default()
        };
        self.cache_masternode_list(block_hash, masternode_list.clone(), cache);
        let needed_masternode_lists = cache.needed_masternode_lists.clone();
        cache.needed_masternode_lists.clear();
        let has_found_coinbase = coinbase_transaction.has_found_coinbase(&merkle_tree.hashes);
        let desired_merkle_root = self.lookup_merkle_root_by_hash(block_hash).unwrap_or(UInt256::MIN);
        let has_valid_coinbase = merkle_tree.has_root(desired_merkle_root);
        let has_valid_mn_list_root = masternode_list.has_valid_mn_list_root(&coinbase_transaction);
        let has_valid_llmq_list_root = !quorums_active || masternode_list.has_valid_llmq_list_root(&coinbase_transaction);
        let result = MNListDiffResult {
            error_status: ProcessingError::None,
            base_block_hash,
            block_hash,
            has_found_coinbase,
            has_valid_coinbase,
            has_valid_mn_list_root,
            has_valid_llmq_list_root,
            has_valid_quorums,
            masternode_list,
            added_masternodes,
            modified_masternodes,
            added_quorums,
            needed_masternode_lists,
        };
        info!("••• RESULT ({}{}{}) •••", u8::from(should_process_quorums), u8::from(is_dip_0024), u8::from(is_rotated_quorums_presented));
        info!("{:?}", result);
        info!("••••••••••••••");
        result
    }

    pub fn classify_masternodes(
        &self,
        base_masternodes: BTreeMap<UInt256, models::MasternodeEntry>,
        added_or_modified_masternodes: BTreeMap<UInt256, models::MasternodeEntry>,
        deleted_masternode_hashes: Vec<UInt256>,
        block_height: u32,
        block_hash: UInt256,
    ) -> (
        BTreeMap<UInt256, models::MasternodeEntry>,
        BTreeMap<UInt256, models::MasternodeEntry>,
        BTreeMap<UInt256, models::MasternodeEntry>,
    ) {
        let added_masternodes = added_or_modified_masternodes
            .iter()
            .filter(|(k, _)| !base_masternodes.contains_key(k))
            .map(|(k, v)| (*k, v.clone()))
            .collect::<BTreeMap<_, _>>();

        let mut modified_masternodes = added_or_modified_masternodes
            .iter()
            .filter(|(k, _)| base_masternodes.contains_key(k))
            .map(|(k, v)| (*k, v.clone()))
            .collect::<BTreeMap<_, _>>();

        let mut masternodes = if !base_masternodes.is_empty() {
            let mut old_masternodes = base_masternodes;
            for hash in deleted_masternode_hashes {
                old_masternodes.remove(&hash.reversed());
            }
            old_masternodes.extend(added_masternodes.clone());
            old_masternodes
        } else {
            added_masternodes.clone()
        };

        for (hash, modified) in &mut modified_masternodes {
            if let Some(old) = masternodes.get_mut(hash) {
                if old.update_height < modified.update_height {
                    modified.update_with_previous_entry(old, block_height, block_hash);
                    if !old.confirmed_hash.is_zero() &&
                        old.known_confirmed_at_height.is_some() &&
                        old.known_confirmed_at_height.unwrap() > block_height {
                        old.known_confirmed_at_height = Some(block_height);
                    }
                }
                masternodes.insert(*hash, modified.clone());
            }
        }
        (added_masternodes, modified_masternodes, masternodes)
    }

    #[allow(clippy::type_complexity)]
    pub fn classify_quorums(
        &self,
        mut base_quorums: BTreeMap<LLMQType, BTreeMap<UInt256, models::LLMQEntry>>,
        mut added_quorums: BTreeMap<LLMQType, BTreeMap<UInt256, models::LLMQEntry>>,
        deleted_quorums: BTreeMap<LLMQType, Vec<UInt256>>,
        should_process_quorums: bool,
        skip_removed_masternodes: bool,
        is_dip_0024: bool,
        is_rotated_quorums_presented: bool,
        cache: &mut MasternodeProcessorCache,
    ) -> (
        BTreeMap<LLMQType, BTreeMap<UInt256, models::LLMQEntry>>,
        BTreeMap<LLMQType, BTreeMap<UInt256, models::LLMQEntry>>,
        bool,
    ) {
        let mut has_valid_quorums = true;
        if should_process_quorums {
            for (&llmq_type, llmqs_of_type) in &mut added_quorums {
                if self.should_process_quorum(llmq_type, is_dip_0024, is_rotated_quorums_presented) {
                    for (&llmq_block_hash, quorum) in llmqs_of_type {
                        // if llmq_type == LLMQType::Llmqtype60_75 {
                        //     println!("//////////// validate_quorum -> {:?}: {}: {}", llmq_type, self.lookup_block_height_by_hash(llmq_block_hash), llmq_block_hash.reversed());
                        // }
                        if let Some(models::MasternodeList { masternodes, .. }) = self.find_masternode_list(llmq_block_hash, &cache.mn_lists, &mut cache.needed_masternode_lists) {
                            let valid = self.validate_quorum(quorum, skip_removed_masternodes, llmq_block_hash, masternodes, cache);
                            // TMP Testnet Platform LLMQ fail verification
                            if llmq_type != LLMQType::Llmqtype25_67 {
                                has_valid_quorums &= valid;
                            }
                        }
                    }
                }
            }
        }
        for (llmq_type, keys_to_delete) in &deleted_quorums {
            if let Some(llmq_map) = base_quorums.get_mut(llmq_type) {
                for key in keys_to_delete {
                    llmq_map.remove(key);
                }
            }
        }
        for (llmq_type, keys_to_add) in added_quorums.iter() {
            base_quorums
                .entry(*llmq_type)
                .or_insert_with(BTreeMap::new)
                .extend(keys_to_add.clone());
        }
        (added_quorums, base_quorums, has_valid_quorums)
    }

    pub fn validate_quorum(
        &self,
        quorum: &mut models::LLMQEntry,
        skip_removed_masternodes: bool,
        block_hash: UInt256,
        masternodes: BTreeMap<UInt256, models::MasternodeEntry>,
        cache: &mut MasternodeProcessorCache,
    ) -> bool {
        let block_height = self.lookup_block_height_by_hash(block_hash);
        // if quorum.llmq_type == LLMQType::Llmqtype60_75 {
        //     println!("//////////// validate_quorum {:?}: {}: {} ({}): {:?}", quorum.llmq_type, block_height, block_hash, block_hash.reversed(), quorum);
        // }
        // java::generate_masternode_list_from_map(&masternodes);
        let valid_masternodes = if quorum.index.is_some() {
            self.get_rotated_masternodes_for_quorum(
                quorum.llmq_type,
                block_hash,
                block_height,
                &mut cache.llmq_members,
                &mut cache.llmq_indexed_members,
                &cache.mn_lists,
                &cache.llmq_snapshots,
                &mut cache.needed_masternode_lists,
                skip_removed_masternodes,
            )
        } else {
            models::MasternodeList::get_masternodes_for_quorum(
                quorum.llmq_type,
                masternodes,
                quorum.llmq_quorum_hash(),
                block_height)
        };

        // info!("••• validate_quorum ({}: {:?}: {:?}) •••", block_height, quorum, valid_masternodes);
        // TMP Generate test for verification with this data
        // info!("#[test]");
        // info!("fn test_{}() {{", thread_rng().gen_range(0..8184));
        // info!("  let block_height = {};", block_height);
        // info!("  let mut quorum = {:?};", quorum);
        // info!("  let valid_masternodes = vec!{:?};", valid_masternodes);
        // info!("  assert!(quorum.validate(valid_masternodes, block_height));");
        // info!("}}");
        // generate_test(|| {
        //     println!("Context context = new Context(TestNet3Params.get());");
        //     println!("context.initDash(true, true);");
        //     println!("NetworkParameters params = context.getParams();");
        //     println!("params.setBasicBLSSchemeActivationHeight(40000);");
        //     println!("BLSScheme.setLegacyDefault({});", quorum.version.use_bls_legacy());
        //     println!("int height = {};", block_height);
        //     println!("Sha256Hash blockHash = Sha256Hash.wrap(\"{}\");", block_hash);
        //     println!("Sha256Hash blockHashReversed = Sha256Hash.wrap(\"{}\");", block_hash.reversed());
        //     println!("StoredBlock storedBlock = new StoredBlock(");
        //     println!("  new Block(params, 536870912, Sha256Hash.wrap(Sha256Hash.wrap(\"0000010080ade24d4fdde00be3b1c58dac97f62ece74b311286ec126e4310a28\").getReversedBytes()), Sha256Hash.wrap(\"afbf4427b30c57535fd8df5eb8750e7176057d5bcdb907274553aa7dfc4c55ad\"), 1681681442, 0, 678112, new ArrayList<>()),");
        //     println!("  new BigInteger(Hex.decode(\"00000000000000000000000000000000000000000000000002d68cc84bf201da\")), height);");
        //     java::generate_final_commitment(&quorum);
        //     java::generate_masternode_list(&valid_masternodes);
        //     println!("boolean verified = finalCommitment.verify(storedBlock, nodes, true);");
        //     println!("assertTrue(verified);");
        // });
        // if quorum.llmq_type == LLMQType::Llmqtype60_75 {
        //     println!("//////////// validate_quorum {} ////////////////", block_height);
        //     // println!("{:#?}", valid_masternodes);
        //     println!("{:#?}", valid_masternodes.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>());
        // }

        quorum.verify(valid_masternodes, block_height)
    }

    fn sort_scored_masternodes(scored_masternodes: BTreeMap<UInt256, models::MasternodeEntry>) -> Vec<models::MasternodeEntry> {
        // let mut v: Vec<_> = scored_masternodes.clone().into_iter().collect::<Vec<_>>();
        // v.sort_unstable_by_key(|(s, _)| std::cmp::Reverse(s.reversed()));
        // v.into_iter().map(|(_, node)| node).collect()
        let mut v = Vec::from_iter(scored_masternodes);
        v.sort_by(|(s1, _), (s2, _)| s2.reversed().cmp(&s1.reversed()));
        v.into_iter().map(|(s, node)| node).collect()
    }

    pub fn valid_masternodes_for_rotated_quorum_map(
        masternodes: Vec<models::MasternodeEntry>,
        quorum_modifier: UInt256,
        block_height: u32,
    ) -> Vec<models::MasternodeEntry> {
        let scored_masternodes = masternodes
            .into_iter()
            .filter_map(|entry| models::MasternodeList::masternode_score(&entry, quorum_modifier, block_height)
                .map(|score| (score, entry)))
            .collect::<BTreeMap<_, _>>();
        Self::sort_scored_masternodes(scored_masternodes)
    }

    // Reconstruct quorum members at index from snapshot
    pub fn quorum_quarter_members_by_snapshot(
        &self,
        llmq_params: LLMQParams,
        quorum_base_block_height: u32,
        cached_lists: &BTreeMap<UInt256, models::MasternodeList>,
        cached_snapshots: &BTreeMap<UInt256, models::LLMQSnapshot>,
        unknown_lists: &mut Vec<UInt256>,
    ) -> Vec<Vec<models::MasternodeEntry>> {
        let work_block_height = quorum_base_block_height - 8;
        let llmq_type = llmq_params.r#type;
        let quorum_count = llmq_params.signing_active_quorum_count as usize;
        let quorum_size = llmq_params.size;
        let quarter_size = (quorum_size / 4) as usize;
        // Quorum members dichotomy in snapshot
        match self.lookup_block_hash_by_height(work_block_height) {
            None => panic!("MISSING: block for height: {}", work_block_height),
            Some(work_block_hash) => {
                if let Some(masternode_list) = self.find_masternode_list(work_block_hash, cached_lists, unknown_lists) {
                    if let Some(snapshot) = self.find_snapshot(work_block_hash, cached_snapshots) {
                        let mut i: u32 = 0;
                        // println!("•••• quorum_quarter_members_by_snapshot: {:?}: {:?}: {}: {}", llmq_type, snapshot.skip_list_mode, work_block_height, work_block_hash.reversed());
                        // println!("{:#?}", masternode_list);
                        // println!("••••");
                        // java::generate_snapshot(&snapshot, work_block_height);
                        // java::generate_llmq_hash(llmq_type, work_block_hash.reversed());
                        // java::generate_masternode_list_from_map(&masternode_list.masternodes);
                        let quorum_modifier = models::LLMQEntry::build_llmq_quorum_hash(llmq_type, work_block_hash);
                        // println!("quorum_modifier: {}", quorum_modifier);
                        // println!("snapshot: {:?}", snapshot);
                        let scored_masternodes = models::MasternodeList::score_masternodes_map(masternode_list.masternodes, quorum_modifier, work_block_height);
                        // java::generate_masternode_list_from_map(&scored_masternodes);
                        let sorted_scored_masternodes = Self::sort_scored_masternodes(scored_masternodes);
                        // println!("//////////////////sorted_scored_masternodes////////////////////");
                        // println!("{:#?}", sorted_scored_masternodes.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>());
                        let (used_at_h, unused_at_h) = sorted_scored_masternodes
                            .into_iter()
                            .partition(|_| {
                                let is_true = snapshot.member_is_true_at_index(i);
                                i += 1;
                                is_true
                            });
                        // java::generate_masternode_list(&used_at_h);
                        // java::generate_masternode_list(&unused_at_h);
                        // println!("//////////////////////////////////////");
                        let sorted_used_at_h = Self::valid_masternodes_for_rotated_quorum_map(
                            used_at_h,
                            quorum_modifier,
                            work_block_height,
                        );
                        // println!("////////////sorted_used_at_h////////////////");
                        // println!("{:#?}", sorted_used_at_h.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>());
                        let sorted_unused_at_h = Self::valid_masternodes_for_rotated_quorum_map(
                            unused_at_h,
                            quorum_modifier,
                            work_block_height,
                        );
                        // println!("////////////sorted_unused_at_h////////////////");
                        // println!("{:#?}", sorted_unused_at_h.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>());
                        let mut sorted_combined_mns_list = sorted_unused_at_h;
                        sorted_combined_mns_list.extend(sorted_used_at_h);
                        // println!("////////////sorted_combined_mns_list////////////////");
                        // println!("{:#?}", sorted_combined_mns_list.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>());
                        snapshot.apply_skip_strategy(sorted_combined_mns_list, quorum_count, quarter_size)
                    } else {
                        info!("MISSING: snapshot for block at height: {}: {}", work_block_height, work_block_hash);
                        vec![]
                    }
                } else {
                    info!("MISSING: masternode_list for block at height: {}: {}", work_block_height, work_block_hash);
                    vec![]
                }
            }
        }
    }

    // fn log_masternodes(vec: &Vec<models::MasternodeEntry>, prefix: String) {
    //     info!("{}", prefix);
    //     vec.iter().for_each(|m| info!("{:?}", m.provider_registration_transaction_hash.reversed()));
    // }

    // Determine quorum members at new index
    pub fn new_quorum_quarter_members(
        &self,
        params: LLMQParams,
        quorum_base_block_height: u32,
        previous_quarters: [&Vec<Vec<models::MasternodeEntry>>; 3],
        cached_lists: &BTreeMap<UInt256, models::MasternodeList>,
        unknown_lists: &mut Vec<UInt256>,
        skip_removed_masternodes: bool,
    ) -> Vec<Vec<models::MasternodeEntry>> {
        let quorum_count = params.signing_active_quorum_count as usize;
        let mut quarter_quorum_members = vec![Vec::<models::MasternodeEntry>::new(); quorum_count];
        let quorum_size = params.size as usize;
        let quarter_size = quorum_size / 4;
        let work_block_height = quorum_base_block_height - 8;
        match self.lookup_block_hash_by_height(work_block_height) {
            None => panic!("missing block for height: {}", work_block_height),
            Some(work_block_hash) => {
                if let Some(masternode_list) = self.find_masternode_list(work_block_hash, cached_lists, unknown_lists) {
                    //java::generate_masternode_list_from_map(&masternode_list.masternodes);
                    // println!("•••• new_quorum_quarter_members: {:?}: (skip_removed: {}) {}: {}", params.r#type, skip_removed_masternodes, work_block_height, work_block_hash.reversed());
                    // println!("{:#?}", masternode_list);
                    // println!("••••");
                    if masternode_list.masternodes.len() < quarter_size {
                        println!("models list at {}: {} has less masternodes ({}) then required for quarter size: ({})", work_block_height, work_block_hash, masternode_list.masternodes.len(), quarter_size);
                        quarter_quorum_members
                    } else {
                        let mut used_at_h_masternodes = Vec::<models::MasternodeEntry>::new();
                        let mut unused_at_h_masternodes = Vec::<models::MasternodeEntry>::new();
                        let mut used_at_h_indexed_masternodes = vec![Vec::<models::MasternodeEntry>::new(); quorum_count];
                        for i in 0..quorum_count {
                            // for quarters h - c, h -2c, h -3c
                            for quarter in &previous_quarters {
                                if let Some(quarter_nodes) = quarter.get(i) {
                                    //Self::log_masternodes(quarter_nodes, format!("••••• PREV QUARTER {} ••••••• ", i));
                                    for node in quarter_nodes {
                                        let hash = node.provider_registration_transaction_hash;
                                        // let skip = skip_removed_masternodes && !masternode_list.has_masternode(node.provider_registration_transaction_hash);
                                        if (!skip_removed_masternodes || masternode_list.has_masternode(hash)) &&
                                            masternode_list.has_valid_masternode(hash) {
                                            // node.is_valid {
                                            if !used_at_h_masternodes.iter().any(|m| m.provider_registration_transaction_hash == hash) {
                                                used_at_h_masternodes.push(node.clone());
                                            }
                                            if !used_at_h_indexed_masternodes[i].iter().any(|m| m.provider_registration_transaction_hash == hash) {
                                                used_at_h_indexed_masternodes[i].push(node.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        masternode_list.masternodes.values().for_each(|mn| {
                            if mn.is_valid && !used_at_h_masternodes.iter()
                                .any(|node| mn.provider_registration_transaction_hash == node.provider_registration_transaction_hash) {
                                unused_at_h_masternodes.push(mn.clone());
                            }
                        });
                        //Self::log_masternodes(&used_at_h_masternodes, format!("••••• USED AT H {} ••••••• ", work_block_height));
                        //Self::log_masternodes(&unused_at_h_masternodes, format!("••••• UNUSED AT H {} •••••••", work_block_height));
                        let quorum_modifier = models::LLMQEntry::build_llmq_quorum_hash(params.r#type, work_block_hash);
                        let sorted_used_mns_list = Self::valid_masternodes_for_rotated_quorum_map(used_at_h_masternodes, quorum_modifier, work_block_height);
                        let sorted_unused_mns_list = Self::valid_masternodes_for_rotated_quorum_map(unused_at_h_masternodes, quorum_modifier, work_block_height);
                        //Self::log_masternodes(&sorted_unused_mns_list, format!("••••• SORTED UNUSED AT H {} ••••••• ", work_block_height));
                        //Self::log_masternodes(&sorted_used_mns_list, format!("••••• SORTED USED AT H {} ••••••• ", work_block_height));
                        let mut sorted_combined_mns_list = sorted_unused_mns_list;
                        sorted_combined_mns_list.extend(sorted_used_mns_list);
                        // println!("••••• SORTED COMBINED AT H {} •••••••", work_block_height);
                        // println!("{:#?}", sorted_combined_mns_list.iter().map(|m|m.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>());

                        let mut skip_list = Vec::<i32>::new();
                        let mut first_skipped_index = 0i32;
                        let mut idx = 0i32;
                        for i in 0..quorum_count {
                            let masternodes_used_at_h_indexed_at_i = used_at_h_indexed_masternodes.get_mut(i).unwrap();
                            let used_mns_count = masternodes_used_at_h_indexed_at_i.len();
                            let sorted_combined_mns_list_len = sorted_combined_mns_list.len();
                            let mut updated = false;
                            let initial_loop_idx = idx;
                            while quarter_quorum_members[i].len() < quarter_size && used_mns_count + quarter_quorum_members[i].len() < sorted_combined_mns_list_len {
                                let mn = sorted_combined_mns_list.get(idx as usize).unwrap();
                                // TODO: replace masternodes with smart pointers to avoid cloning
                                if masternodes_used_at_h_indexed_at_i.iter().any(|node| mn.provider_registration_transaction_hash == node.provider_registration_transaction_hash) {
                                    let skip_index = idx - first_skipped_index;
                                    if first_skipped_index == 0 {
                                        first_skipped_index = idx;
                                    }
                                    skip_list.push(idx);
                                } else {
                                    masternodes_used_at_h_indexed_at_i.push(mn.clone());
                                    quarter_quorum_members[i].push(mn.clone());
                                    updated = true;
                                }
                                idx += 1;
                                if idx == sorted_combined_mns_list_len as i32 {
                                    idx = 0;
                                }
                                if idx == initial_loop_idx {
                                    if !updated {
                                        println!("there are not enough MNs {}: {} then required for quarter size: ({})", work_block_height, work_block_hash, quarter_size);
                                        return vec![Vec::<models::MasternodeEntry>::new(); quorum_count];
                                    }
                                    updated = false;
                                }
                            }
                        }
                        // println!("••••• QUARTER MEMBERS •••••••");
                        // quarter_quorum_members.iter().enumerate().for_each(|(index, members)| {
                        //     Self::log_masternodes(&members, format!("••••• INDEX {} ••••••• ", index));
                        // });
                        // println!("•••••");
                        quarter_quorum_members
                    }
                } else {
                    println!("missing models list for height: {}: {}", work_block_height, work_block_hash);
                    quarter_quorum_members
                }
            }
        }
    }

    fn add_quorum_members_from_quarter(
        quorum_members: &mut Vec<Vec<models::MasternodeEntry>>,
        quarter: &[Vec<models::MasternodeEntry>],
        index: usize,
    ) {
        if let Some(indexed_quarter) = quarter.get(index) {
            quorum_members.resize_with(index + 1, Vec::new);
            quorum_members[index].extend(indexed_quarter.iter().cloned());
        }
    }

    fn rotate_members(
        &self,
        cycle_quorum_base_block_height: u32,
        llmq_params: LLMQParams,
        cached_lists: &BTreeMap<UInt256, models::MasternodeList>,
        cached_snapshots: &BTreeMap<UInt256, models::LLMQSnapshot>,
        unknown_lists: &mut Vec<UInt256>,
        skip_removed_masternodes: bool,
    ) -> Vec<Vec<models::MasternodeEntry>> {
        let num_quorums = llmq_params.signing_active_quorum_count as usize;
        let cycle_length = llmq_params.dkg_params.interval;
        // println!("/////////////////////// rotate_members {}: {} /////////", cycle_quorum_base_block_height, cycle_length);
        let quorum_base_block_height = cycle_quorum_base_block_height - cycle_length;
        let prev_q_h_m_c = self.quorum_quarter_members_by_snapshot(llmq_params, quorum_base_block_height, cached_lists, cached_snapshots, unknown_lists);
        // println!("/////////////////////// prev_q_h_m_c : {} /////////", quorum_base_block_height);
        // println!("{:#?}", prev_q_h_m_c.iter().map(|p| p.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>()).collect::<Vec<_>>());
        let quorum_base_block_height = cycle_quorum_base_block_height - 2 * cycle_length;
        let prev_q_h_m_2c = self.quorum_quarter_members_by_snapshot(llmq_params, quorum_base_block_height, cached_lists, cached_snapshots, unknown_lists);
        // println!("/////////////////////// prev_q_h_m_2c : {} /////////", quorum_base_block_height);
        // println!("{:#?}", prev_q_h_m_2c.iter().map(|p| p.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>()).collect::<Vec<_>>());
        let quorum_base_block_height = cycle_quorum_base_block_height - 3 * cycle_length;
        let prev_q_h_m_3c = self.quorum_quarter_members_by_snapshot(llmq_params, quorum_base_block_height, cached_lists, cached_snapshots, unknown_lists);
        // println!("/////////////////////// prev_q_h_m_3c : {} /////////", quorum_base_block_height);
        // println!("{:#?}", prev_q_h_m_3c.iter().map(|p| p.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>()).collect::<Vec<_>>());

        let mut rotated_members =
            Vec::<Vec<models::MasternodeEntry>>::with_capacity(num_quorums);
        let new_quarter_members = self.new_quorum_quarter_members(
            llmq_params,
            cycle_quorum_base_block_height,
            [
                &prev_q_h_m_c,
                &prev_q_h_m_2c,
                &prev_q_h_m_3c,
            ],
            cached_lists,
            unknown_lists,
            skip_removed_masternodes,
        );
        // println!("/////////////////////// new_quarter_members : {} /////////", cycle_quorum_base_block_height);
        // println!("{:#?}", new_quarter_members.iter().map(|p| p.iter().map(|n| n.provider_registration_transaction_hash.reversed()).collect::<Vec<_>>()).collect::<Vec<_>>());

        (0..num_quorums).for_each(|i| {
            Self::add_quorum_members_from_quarter(&mut rotated_members, &prev_q_h_m_3c, i);
            Self::add_quorum_members_from_quarter(&mut rotated_members, &prev_q_h_m_2c, i);
            Self::add_quorum_members_from_quarter(&mut rotated_members, &prev_q_h_m_c, i);
            Self::add_quorum_members_from_quarter(&mut rotated_members, &new_quarter_members, i);
        });
        rotated_members
    }

    /// Determine masternodes which is responsible for signing at this quorum index
    #[allow(clippy::too_many_arguments)]
    pub fn get_rotated_masternodes_for_quorum(
        &self,
        llmq_type: LLMQType,
        block_hash: UInt256,
        block_height: u32,
        cached_llmq_members: &mut BTreeMap<LLMQType, BTreeMap<UInt256, Vec<models::MasternodeEntry>>>,
        cached_llmq_indexed_members: &mut BTreeMap<LLMQType, BTreeMap<models::LLMQIndexedHash, Vec<models::MasternodeEntry>>>,
        cached_mn_lists: &BTreeMap<UInt256, models::MasternodeList>,
        cached_llmq_snapshots: &BTreeMap<UInt256, models::LLMQSnapshot>,
        cached_needed_masternode_lists: &mut Vec<UInt256>,
        skip_removed_masternodes: bool,
    ) -> Vec<models::MasternodeEntry> {
        let map_by_type_opt = cached_llmq_members.get_mut(&llmq_type);
        if map_by_type_opt.is_some() {
            if let Some(members) = map_by_type_opt.as_ref().unwrap().get(&block_hash) {
                return members.clone();
            }
        } else {
            cached_llmq_members.insert(llmq_type, BTreeMap::new());
        }
        let map_by_type = cached_llmq_members.get_mut(&llmq_type).unwrap();
        let llmq_params = llmq_type.params();
        let quorum_index = block_height % llmq_params.dkg_params.interval;
        let cycle_base_height = block_height - quorum_index;
        // println!("/////////////////////get_rotated_masternodes_for_quorum {} {} {} {}", block_height, llmq_params.dkg_params.interval, quorum_index, cycle_base_height);
        match self.lookup_block_hash_by_height(cycle_base_height) {
            None => panic!("missing hash for block at height: {}", cycle_base_height),
            Some(cycle_base_hash) => {
                let map_by_type_indexed_opt = cached_llmq_indexed_members.get_mut(&llmq_type);
                if map_by_type_indexed_opt.is_some() {
                    if let Some(members) = map_by_type_indexed_opt
                        .as_ref()
                        .unwrap()
                        .get(&(cycle_base_hash, quorum_index).into())
                    {
                        map_by_type.insert(block_hash, members.clone());
                        return members.clone();
                    }
                } else {
                    cached_llmq_indexed_members.insert(llmq_type, BTreeMap::new());
                }
                let rotated_members = self.rotate_members(
                    cycle_base_height,
                    llmq_params,
                    cached_mn_lists,
                    cached_llmq_snapshots,
                    cached_needed_masternode_lists,
                    skip_removed_masternodes,
                );
                let map_indexed_quorum_members_of_type =
                    cached_llmq_indexed_members.get_mut(&llmq_type).unwrap();
                rotated_members.iter().enumerate().for_each(|(i, members)| {
                    map_indexed_quorum_members_of_type.insert((cycle_base_hash, i).into(), members.clone());
                });
                if let Some(members) = rotated_members.get(quorum_index as usize) {
                    map_by_type.insert(block_hash, members.clone());
                    return members.clone();
                }
                vec![]
            }
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////
    /// FFI-callbacks
    ///////////////////////////////////////////////////////////////////////////////////////////

    pub fn lookup_masternode_list(
        &self,
        block_hash: UInt256,
    ) -> Option<models::MasternodeList> {
        // First look at the local cache
        callbacks::lookup_masternode_list(
            block_hash,
            |h: UInt256| unsafe {
                (self.get_masternode_list_by_block_hash)(boxed(h.0), self.opaque_context)
            },
            |list: *mut types::MasternodeList| unsafe { (self.destroy_masternode_list)(list) },
        )
    }

    pub fn save_masternode_list(
        &self,
        block_hash: UInt256,
        masternode_list: &models::MasternodeList,
    ) -> bool {
        unsafe {
            (self.save_masternode_list)(
                boxed(block_hash.0),
                boxed(masternode_list.encode()),
                self.opaque_context,
            )
        }
    }

    pub fn lookup_block_hash_by_height(&self, block_height: u32) -> Option<UInt256> {
        callbacks::lookup_block_hash_by_height(
            block_height,
            |h: u32| unsafe { (self.get_block_hash_by_height)(h, self.opaque_context) },
            |hash: *mut u8| unsafe { (self.destroy_hash)(hash) },
        )
    }

    pub fn lookup_block_height_by_hash(&self, block_hash: UInt256) -> u32 {
        unsafe { (self.get_block_height_by_hash)(boxed(block_hash.0), self.opaque_context) }
    }

    pub fn lookup_snapshot_by_block_hash(&self, block_hash: UInt256) -> Option<models::LLMQSnapshot> {
        callbacks::lookup_snapshot_by_block_hash(
            block_hash,
            |h: UInt256| unsafe {
                (self.get_llmq_snapshot_by_block_hash)(boxed(h.0), self.opaque_context)
            },
            |snapshot: *mut types::LLMQSnapshot| unsafe { (self.destroy_snapshot)(snapshot) },
        )
    }

    pub fn save_snapshot(&self, block_hash: UInt256, snapshot: models::LLMQSnapshot) -> bool {
        unsafe {
            (self.save_llmq_snapshot)(
                boxed(block_hash.0),
                boxed(snapshot.encode()),
                self.opaque_context,
            )
        }
    }

    pub fn lookup_merkle_root_by_hash(&self, block_hash: UInt256) -> Option<UInt256> {
        callbacks::lookup_merkle_root_by_hash(
            block_hash,
            |h: UInt256| unsafe { (self.get_merkle_root_by_hash)(boxed(h.0), self.opaque_context) },
            |hash: *mut u8| unsafe { (self.destroy_hash)(hash) },
        )
    }

    pub fn should_process_quorum(&self, llmq_type: LLMQType, is_dip_0024: bool, is_rotated_quorums_presented: bool) -> bool {
        // TODO: what we really wants here for platform quorum type?
        if self.chain_type.isd_llmq_type() == llmq_type {
            is_dip_0024 && is_rotated_quorums_presented
        } else if is_dip_0024 { /*skip old quorums here for now*/
            false
        } else {
            self.chain_type.should_process_llmq_of_type(llmq_type)
        }
    }

    pub fn should_process_diff_with_range(
        &self,
        base_block_hash: UInt256,
        block_hash: UInt256,
    ) -> ProcessingError {
        unsafe {
            (self.should_process_diff_with_range)(
                boxed(base_block_hash.0),
                boxed(block_hash.0),
                self.opaque_context,
            )
        }
    }

    pub fn add_insight(&self, block_hash: UInt256) {
        unsafe { (self.add_insight)(boxed(block_hash.0), self.opaque_context) }
    }

    pub fn read_list_diff_from_message<'a>(
        &self,
        message: &'a [u8],
        offset: &mut usize,
        protocol_version: u32
    ) -> Option<models::MNListDiff> {
        models::MNListDiff::new(message, offset, |block_hash| self.lookup_block_height_by_hash(block_hash), protocol_version)
    }
}
