use crate::common::MasternodeType;
use crate::consensus::Encodable;
use crate::crypto::{
    byte_util::{Reversable, Zeroable},
    UInt256,
};
use crate::tx::CoinbaseTransaction;
use crate::util::data_ops::merkle_root_from_hashes;
use std::cmp::min;
use std::collections::BTreeMap;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub struct MasternodeList {
    pub block_hash: UInt256,
    pub known_height: u32,
    pub masternode_merkle_root: Option<UInt256>,
    pub llmq_merkle_root: Option<UInt256>,
    pub masternodes: BTreeMap<UInt256, crate::models::masternode_entry::MasternodeEntry>,
    pub quorums: BTreeMap<crate::chain::common::llmq_type::LLMQType, BTreeMap<UInt256, crate::models::llmq_entry::LLMQEntry>>,
}

impl Default for MasternodeList {
    fn default() -> Self {
        Self {
            block_hash: UInt256::MAX,
            known_height: 0,
            masternode_merkle_root: None,
            llmq_merkle_root: None,
            masternodes: Default::default(),
            quorums: Default::default(),
        }
    }
}

impl<'a> std::fmt::Debug for MasternodeList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MasternodeList")
            .field("block_hash", &self.block_hash)
            .field("known_height", &self.known_height)
            .field(
                "masternode_merkle_root",
                &self.masternode_merkle_root.unwrap_or(UInt256::MIN),
            )
            .field(
                "llmq_merkle_root",
                &self.llmq_merkle_root.unwrap_or(UInt256::MIN),
            )
            .field("masternodes", &self.masternodes)
            .field("quorums", &self.quorums)
            .finish()
    }
}

impl MasternodeList {
    pub fn new(
        masternodes: BTreeMap<UInt256, crate::models::MasternodeEntry>,
        quorums: BTreeMap<
            crate::chain::common::LLMQType,
            BTreeMap<UInt256, crate::models::LLMQEntry>,
        >,
        block_hash: UInt256,
        block_height: u32,
        quorums_active: bool,
    ) -> Self {
        let mut list = Self {
            quorums,
            block_hash,
            known_height: block_height,
            masternode_merkle_root: None,
            llmq_merkle_root: None,
            masternodes,
        };
        if let Some(hashes) = list.hashes_for_merkle_root(block_height) {
            //println!("MasternodeList: {}:{}: hashes_for_merkle_root: {:#?} masternodes: {:#?}", block_height, block_hash, hashes, list.masternodes);
            list.masternode_merkle_root = merkle_root_from_hashes(hashes);
        }
        if quorums_active {
            let hashes = list.hashes_for_quorum_merkle_root();
            //println!("MasternodeList: {}:{}: hashes_for_quorum_merkle_root: {:#?} quorums: {:#?}", block_height, block_hash, hashes, list.quorums);
            list.llmq_merkle_root = merkle_root_from_hashes(hashes);
        }
        list
    }

    pub fn quorums_count(&self) -> u64 {
        let mut count: u64 = 0;
        for entry in self.quorums.values() {
            count += entry.len() as u64;
        }
        count
    }

    pub fn hashes_for_merkle_root(&self, block_height: u32) -> Option<Vec<UInt256>> {
        (block_height != u32::MAX).then_some({
            let mut pro_tx_hashes = self.reversed_pro_reg_tx_hashes();
            pro_tx_hashes.sort_by(|&s1, &s2| s1.reversed().cmp(&s2.reversed()));
            pro_tx_hashes
                .iter()
                .map(|hash| (&self.masternodes[hash]).entry_hash_at(block_height))
                .collect::<Vec<_>>()
        })
    }

    fn hashes_for_quorum_merkle_root(&self) -> Vec<UInt256> {
        let mut llmq_commitment_hashes = self
            .quorums
            .values()
            .flat_map(|q_map| q_map.values().map(|entry| entry.entry_hash))
            .collect::<Vec<_>>();
        llmq_commitment_hashes.sort();
        llmq_commitment_hashes
    }

    pub fn masternode_for(
        &self,
        registration_hash: UInt256,
    ) -> Option<&crate::models::MasternodeEntry> {
        self.masternodes.get(&registration_hash)
    }

    pub fn has_valid_mn_list_root(&self, tx: &CoinbaseTransaction) -> bool {
        // we need to check that the coinbase is in the transaction hashes we got back
        // and is in the merkle block
        if let Some(mn_merkle_root) = self.masternode_merkle_root {
            //println!("has_valid_mn_list_root: {} == {}", tx.merkle_root_mn_list, mn_merkle_root);
            tx.merkle_root_mn_list == mn_merkle_root
        } else {
            false
        }
    }

    pub fn has_valid_llmq_list_root(&self, tx: &CoinbaseTransaction) -> bool {
        let q_merkle_root = self.llmq_merkle_root;
        let ct_q_merkle_root = tx.merkle_root_llmq_list;
        let has_valid_quorum_list_root = q_merkle_root.is_some()
            && ct_q_merkle_root.is_some()
            && ct_q_merkle_root.unwrap() == q_merkle_root.unwrap();
        if !has_valid_quorum_list_root {
            warn!("LLMQ Merkle root not valid for DML on block {} version {} ({:?} wanted - {:?} calculated)",
                     tx.height,
                     tx.base.version,
                     tx.merkle_root_llmq_list,
                     self.llmq_merkle_root);
        }
        has_valid_quorum_list_root
    }

    pub fn masternode_score(
        entry: &crate::models::MasternodeEntry,
        modifier: UInt256,
        block_height: u32,
    ) -> Option<UInt256> {
        if !entry.is_valid_at(block_height)
            || entry.confirmed_hash.is_zero()
            || entry.confirmed_hash_at(block_height).is_none()
        {
            return None;
        }
        let mut buffer: Vec<u8> = Vec::new();
        if let Some(hash) = entry.confirmed_hash_hashed_with_pro_reg_tx_hash_at(block_height) {
            hash.enc(&mut buffer);
        }
        modifier.enc(&mut buffer);
        let score = UInt256::sha256(&buffer);
        (!score.is_zero() && !score.0.is_empty()).then_some(score)
    }

    pub fn quorum_entry_for_platform_with_quorum_hash(
        &self,
        quorum_hash: UInt256,
        llmq_type: crate::chain::common::LLMQType,
    ) -> Option<&crate::models::LLMQEntry> {
        self.quorums
            .get(&llmq_type)?
            .values()
            .find(|&entry| entry.llmq_hash == quorum_hash)
    }

    pub fn quorum_entry_for_lock_request_id(
        &self,
        request_id: UInt256,
        llmq_type: crate::chain::common::LLMQType,
    ) -> Option<&crate::models::LLMQEntry> {
        let mut first_quorum: Option<&crate::models::LLMQEntry> = None;
        let mut lowest_value = UInt256::MAX;
        self.quorums.get(&llmq_type)?.values().for_each(|entry| {
            let ordering_hash = entry
                .ordering_hash_for_request_id(request_id, llmq_type)
                .reverse();
            if lowest_value > ordering_hash {
                lowest_value = ordering_hash;
                first_quorum = Some(entry);
            }
        });
        first_quorum
    }
    pub fn reversed_pro_reg_tx_hashes(&self) -> Vec<&UInt256> {
        self.masternodes.keys().collect::<Vec<&UInt256>>()
    }

    pub fn sorted_reversed_pro_reg_tx_hashes(&self) -> Vec<&UInt256> {
        let mut hashes = self.reversed_pro_reg_tx_hashes();
        hashes.sort_by(|&s1, &s2| s2.reversed().cmp(&s1.reversed()));
        hashes
    }

    pub fn has_masternode(&self, provider_registration_transaction_hash: UInt256) -> bool {
        // self.masternodes.contains_key(provider_registration_transaction_hash)
        self.masternodes.values().any(|node| {
            node.provider_registration_transaction_hash == provider_registration_transaction_hash
        })
    }

    pub fn has_valid_masternode(&self, provider_registration_transaction_hash: UInt256) -> bool {
        self.masternodes
            .values()
            .find(|node| {
                node.provider_registration_transaction_hash
                    == provider_registration_transaction_hash
            })
            .map_or(false, |node| node.is_valid)
        // self.masternodes.values().any(|node| node.provider_registration_transaction_hash == provider_registration_transaction_hash)
    }
}

impl MasternodeList {
    pub fn score_masternodes_map(
        masternodes: BTreeMap<UInt256, crate::models::MasternodeEntry>,
        quorum_modifier: UInt256,
        block_height: u32,
        hpmn_only: bool,
    ) -> BTreeMap<UInt256, crate::models::MasternodeEntry> {
        masternodes
            .into_iter()
            .filter_map(|(_, entry)| {
                if !hpmn_only || entry.mn_type == MasternodeType::HighPerformance {
                    Self::masternode_score(&entry, quorum_modifier, block_height)
                        .map(|score| (score, entry))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_masternodes_for_quorum(
        llmq_type: crate::chain::common::LLMQType,
        masternodes: BTreeMap<UInt256, crate::models::MasternodeEntry>,
        quorum_modifier: UInt256,
        block_height: u32,
        hpmn_only: bool,
    ) -> Vec<crate::models::MasternodeEntry> {
        let quorum_count = llmq_type.size();
        let masternodes_in_list_count = masternodes.len();
        let mut score_dictionary =
            Self::score_masternodes_map(masternodes, quorum_modifier, block_height, hpmn_only);
        let mut scores: Vec<UInt256> = score_dictionary.clone().into_keys().collect();
        scores.sort_by(|&s1, &s2| s2.reversed().cmp(&s1.reversed()));
        let mut valid_masternodes: Vec<crate::models::MasternodeEntry> = Vec::new();
        let count = min(masternodes_in_list_count, scores.len());
        for score in scores.iter().take(count) {
            if let Some(masternode) = score_dictionary.get_mut(score) {
                if (*masternode).is_valid_at(block_height) {
                    valid_masternodes.push((*masternode).clone());
                }
            }
            if valid_masternodes.len() == quorum_count as usize {
                break;
            }
        }
        valid_masternodes
    }
}
