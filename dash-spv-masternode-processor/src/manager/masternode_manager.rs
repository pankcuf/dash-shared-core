use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Weak;
use crate::chain::chain::Chain;
use crate::chain::common::ChainType;
use crate::crypto::UInt256;
// use crate::chain::masternode::{LLMQEntry, MasternodeList};
// use crate::chain::tx::ITransaction;
use crate::chain::masternode::local_masternode::LocalMasternode;
// use crate::chain::masternode::store::Store;
use crate::chain::network::Peer;
// use crate::chain::SyncType;
use crate::chain::tx::provider_registration_transaction::ProviderRegistrationTransaction;
use crate::models::{LLMQEntry, MasternodeList};
use crate::util::Shared;

pub trait PeerMasternodeDelegate: Sync + Debug {
    fn peer_relayed_masternode_diff_message(&self, peer: &Peer, message: &[u8]);
    fn peer_relayed_quorum_rotation_info_message(&self, peer: &Peer, message: &[u8]);
}

#[derive(Clone, Debug, Default)]
pub struct MasternodeManager {
    // pub store: Store,
    pub local_masternodes_dictionary_by_registration_transaction_hash: HashMap<UInt256, LocalMasternode>,
    pub chain: Shared<Chain>,
    pub chain_type: ChainType,
    pub current_masternode_list: Option<Shared<MasternodeList>>,

}
// impl<'a> Default for &'a MasternodeManager {
//     fn default() -> Self {
//         &MasternodeManager::default()
//     }
// }

impl MasternodeManager {
    pub(crate) fn has_masternode_list_currently_being_saved(&self) -> bool {
        todo!()
    }
}

impl MasternodeManager {
    pub(crate) fn load_file_distributed_masternode_lists(&self) {
        todo!()
    }
}

impl MasternodeManager {
    // the safety delay checks to see if this was called in the last n seconds.
    pub(crate) fn get_current_masternode_list_with_safety_delay(&self, delay: i32) {
        // self.timeIntervalForMasternodeRetrievalSafetyDelay = [[NSDate date] timeIntervalSince1970];
        // dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(safetyDelay * NSEC_PER_SEC)), self.chain.networkingQueue, ^{
        //     NSTimeInterval timeElapsed = [[NSDate date] timeIntervalSince1970] - self.timeIntervalForMasternodeRetrievalSafetyDelay;
        //     if (timeElapsed > safetyDelay) {
        //         [self getRecentMasternodeList];
        //     }
        // });
    }
}

impl MasternodeManager {
    pub fn new(chain_type: ChainType) -> Self {
        Self {
            chain_type,
            // chain,
            // store: Store::new(chain),
            ..Default::default()
        }
    }
    pub(crate) fn quorum_entry_for_chain_lock_request_id(&self, request_id: Option<UInt256>, block_height_offset: u32) -> Option<Weak<LLMQEntry>> {
        todo!()
    }

    pub(crate) fn quorum_entry_for_instant_send_request_id(&self, request_id: &UInt256, block_height_offset: u32) -> Option<Weak<LLMQEntry>> {
        todo!()
    }
}

impl MasternodeManager {
    pub(crate) fn start_sync(&self) {
        todo!()
    }

    pub(crate) fn last_masternode_list_block_height(&self) -> u32 {
        todo!()
    }

    pub(crate) fn wipe_masternode_info(&self) {
        todo!()
    }

    pub(crate) fn wipe_local_masternode_info(&self) {
        todo!()
    }

    pub fn recent_masternode_lists(&self) -> Vec<MasternodeList> {
        todo!()
        // self.store.masternode_lists_by_block_hash.into_values().collect()
    }

    pub fn estimated_masternode_lists_to_sync(&mut self) -> u32 {
        todo!()
        // if self.chain_type.sync_type.bits() & SyncType::MasternodeList.bits() != 0 {
        //     0
        // } else if self.masternode_list_retrieval_queue_max_amount() == 0 || self.store.masternode_lists_by_block_hash.len() <= 1 {
        //     // 1 because there might be a default
        //     self.store.masternode_lists_to_sync()
        // } else {
        //     self.masternode_list_retrieval_queue_count() as u32
        // }
    }

    pub fn masternode_list_retrieval_queue_count(&self) -> usize {
        todo!("impl list diff service")
        //return [self.masternodeListDiffService retrievalQueueCount] + [self.quorumRotationService retrievalQueueCount];
    }

    pub fn masternode_list_retrieval_queue_max_amount(&self) -> usize {
        todo!("impl list diff service")
        //return [self.masternodeListDiffService retrievalQueueMaxAmount] + [self.quorumRotationService retrievalQueueMaxAmount];
    }

    pub fn masternode_list_and_quorums_sync_progress(&mut self) -> f64 {
        todo!()
        // let amount_left = self.masternode_list_retrieval_queue_count();
        // let max_amount = self.masternode_list_retrieval_queue_max_amount() as f64;
        // if amount_left == 0 {
        //     self.store.masternode_lists_and_quorums_is_synced() as i32 as f64
        // } else {
        //     0f64.max(1f64.min(1f64 - (amount_left as f64 / max_amount)))
        // }
    }

    pub fn local_masternode_from_provider_registration_transaction(&self, transaction: &ProviderRegistrationTransaction, save: bool) -> Option<&LocalMasternode> {
        // First check to see if we have a local masternode for this provider registration hash
        todo!()
        /*let tx_hash = transaction.tx_hash();
        if let Some(local_masternode) = self.local_masternodes_dictionary_by_registration_transaction_hash.get(&tx_hash) {
            //todo Update keys
            return Option::from(local_masternode)
        }
        let local_masternode = LocalMasternode::init_with_provider_registration_transaction(transaction);
        if local_masternode.no_local_wallet() {
            return None;
        }
        self.local_masternodes_dictionary_by_registration_transaction_hash.insert(tx_hash, local_masternode);
        if save {
            local_masternode.save();
        }
        Some(&local_masternode)*/
    }

    pub fn local_masternode_having_provider_registration_transaction_hash(&self, hash: &UInt256) -> Option<&LocalMasternode> {
        self.local_masternodes_dictionary_by_registration_transaction_hash.get(hash)
    }


    pub fn has_masternode_at_location(&self, socket_addr: SocketAddr) -> bool {
        todo!()
    }
}
