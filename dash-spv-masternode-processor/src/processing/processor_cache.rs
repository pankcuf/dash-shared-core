use std::collections::BTreeMap;
use crate::chain::common;
use crate::crypto::UInt256;
use crate::models;

#[derive(Clone, Default)]
pub struct MasternodeProcessorCache {
    pub llmq_members: BTreeMap<common::LLMQType, BTreeMap<UInt256, Vec<models::MasternodeEntry>>>,
    pub llmq_indexed_members: BTreeMap<common::LLMQType, BTreeMap<models::LLMQIndexedHash, Vec<models::MasternodeEntry>>>,
    pub mn_lists: BTreeMap<UInt256, models::MasternodeList>,
    pub llmq_snapshots: BTreeMap<UInt256, models::LLMQSnapshot>,
    pub needed_masternode_lists: Vec<UInt256>,
}

impl std::fmt::Debug for MasternodeProcessorCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MasternodeProcessorCache")
            .field("llmq_members", &self.llmq_members)
            .field("llmq_indexed_members", &self.llmq_indexed_members)
            .field("llmq_snapshots", &self.llmq_snapshots)
            .field("mn_lists", &self.mn_lists)
            .field("needed_masternode_lists", &self.needed_masternode_lists)
            .finish()
    }
}

impl MasternodeProcessorCache {
    pub fn clear(&mut self) {
        self.llmq_members.clear();
        self.llmq_indexed_members.clear();
        self.mn_lists.clear();
        self.llmq_snapshots.clear();
        self.needed_masternode_lists.clear();
    }
    pub fn add_masternode_list(&mut self, block_hash: UInt256, list: models::MasternodeList) {
        self.mn_lists.insert(block_hash, list);
    }
    pub fn remove_masternode_list(&mut self, block_hash: &UInt256) {
        self.mn_lists.remove(block_hash);
    }
    pub fn add_snapshot(&mut self, block_hash: UInt256, snapshot: models::LLMQSnapshot) {
        self.llmq_snapshots.insert(block_hash, snapshot);
    }
    pub fn remove_snapshot(&mut self, block_hash: &UInt256) {
        self.llmq_snapshots.remove(block_hash);
    }
    pub fn get_quorum_members_of_type(
        &mut self,
        r#type: common::LLMQType,
    ) -> Option<&mut BTreeMap<UInt256, Vec<models::MasternodeEntry>>> {
        self.llmq_members.get_mut(&r#type)
    }

    pub fn get_indexed_quorum_members_of_type(
        &mut self,
        r#type: common::LLMQType,
    ) -> Option<&mut BTreeMap<models::LLMQIndexedHash, Vec<models::MasternodeEntry>>> {
        self.llmq_indexed_members.get_mut(&r#type)
    }

    pub fn get_quorum_members(
        &mut self,
        r#type: common::LLMQType,
        block_hash: UInt256,
    ) -> Option<Vec<models::MasternodeEntry>> {
        let map_by_type_opt = self.get_quorum_members_of_type(r#type);
        if map_by_type_opt.is_some() {
            if let Some(members) = map_by_type_opt.as_ref().unwrap().get(&block_hash) {
                return Some(members.clone());
            }
        }
        None
    }

    pub fn remove_quorum_members(&mut self, block_hash: &UInt256) {
        self.llmq_members.iter_mut().for_each(|(llmq_type, map)| {
            map.remove(block_hash);
        });
        self.llmq_indexed_members.iter_mut().for_each(|(llmq_type, map)| {
            let empties = map
                .iter()
                .filter(|&(&k, _)| k.hash == *block_hash)
                .map(|(k, _)| *k)
                .collect::<Vec<_>>();
            empties.iter().for_each(|h| {
                map.remove(h);
            });
        });
    }
}
