use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use crate::chain::{Chain, tx};
use crate::chain::network::{BloomFilter, Peer};
use crate::chain::common::ChainType;
use crate::crypto::UInt256;
use crate::util;
use crate::util::Shared;

pub trait PublishCallback: Fn(util::Error) + Send + Sync {}
impl<T: Fn(util::Error) + Send + Sync + Clone + 'static> PublishCallback for T {}

#[derive(Clone, Default)]
pub struct TransactionManager {
    pub chain: Shared<Chain>,
    pub chain_type: ChainType,
    published_tx: HashMap<UInt256, tx::Kind>,
    published_callback: HashMap<UInt256, Arc<dyn PublishCallback<Output=()>>>,

}
impl Debug for TransactionManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionManager")
            .field("chain", &self.chain)
            .finish()
    }
}


impl TransactionManager {
    pub fn new(chain_type: ChainType) -> Self {
        Self {
            chain_type,
            ..Default::default()
        }
    }
    // This returns the bloom filter for the peer, currently the filter is only tweaked per peer,
    // and we only cache the filter of the download peer.
    // It makes sense to keep this in this class because it is not a property of the chain, but
    // instead of a ephemeral item used in the synchronization of the chain.
    pub fn transactions_bloom_filter_for_peer_hash(&self, hash: u32) -> BloomFilter {
        todo!()
    }
    // unconfirmed transactions that aren't in the mempools of any of connected peers have likely dropped off the network
    pub fn remove_unrelayed_transactions_from_peer(&self, peer: &Peer) {

    }
}

impl TransactionManager {
    pub fn published_tx_hashes(&self) -> Vec<UInt256> {
        self.published_tx.keys().cloned().collect()
    }
    pub fn published_callback_hashes(&self) -> Vec<UInt256> {
        self.published_callback.keys().cloned().collect()
    }

    pub fn clear_transactions_bloom_filter(&self) {

    }

    pub fn clear_transaction_relays_for_peer(&self, peer: &Peer) {
        // for (NSValue *txHash in self.txRelays.allKeys) {
        // [self.txRelays[txHash] removeObject:peer];
        // }

    }
}
