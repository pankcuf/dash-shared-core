use std::collections::HashMap;
use std::sync::Arc;
use crate::chain::block::{Block, IBlock, Kind};
use crate::chain::Chain;
use crate::chain::chain_lock::ChainLock;
use crate::chain::common::ChainType;
use crate::crypto::UInt256;
use crate::util::data_ops::merkle_root_from_hashes;
use crate::util::Shared;

#[derive(Clone, Debug, Default)]
pub struct FullBlock {
    pub base: Block,
    // pub transactions: Vec<&'static dyn ITransaction>,
}

impl PartialEq for FullBlock {
    fn eq(&self, other: &Self) -> bool {
        self == other || self.block_hash() == other.block_hash()
        // return self == obj || ([obj isMemberOfClass:[self class]] && uint256_eq([obj blockHash], _blockHash));
    }
}

impl IBlock for FullBlock {
    fn chain(&self) -> Shared<Chain> {
        self.base.chain()
    }

    fn chain_type(&self) -> ChainType {
        self.base.chain_type()
    }

    fn height(&self) -> u32 {
        self.base.height()
    }

    fn set_height(&mut self, height: u32) {
        self.base.set_height(height)
    }

    fn block_hash(&self) -> UInt256 {
        self.base.block_hash()
    }

    fn merkle_root(&self) -> UInt256 {
        self.base.merkle_root()
    }

    fn prev_block(&self) -> UInt256 {
        self.base.prev_block()
    }

    fn target(&self) -> u32 {
        self.base.target
    }

    fn to_data(&self) -> Vec<u8> {
        self.base.to_data()
    }

    fn timestamp(&self) -> u32 {
        self.base.timestamp()
    }

    fn transaction_hashes(&self) -> Vec<UInt256> {
        todo!()
        // self.transactions.iter().map(|tx| tx.tx_hash()).collect()
    }

    fn chain_work(&self) -> UInt256 {
        self.base.chain_work()
    }

    fn set_chain_work(&mut self, chain_work: UInt256) {
        self.base.set_chain_work(chain_work)
    }

    fn set_chain_locked_with_chain_lock(&mut self, chain_lock: Arc<ChainLock>) {
        self.base.set_chain_locked_with_chain_lock(chain_lock);
    }

    fn set_chain_locked_with_equivalent_block(&mut self, block: &dyn IBlock) {
        self.base.set_chain_locked_with_equivalent_block(block);
    }

    fn chain_locked(&self) -> bool {
        self.base.chain_locked()
    }

    fn has_unverified_chain_lock(&self) -> bool {
        self.base.has_unverified_chain_lock()
    }

    fn chain_lock_awaiting_processing(&self) -> Option<Arc<ChainLock>> {
        self.base.chain_lock_awaiting_processing()
    }

    fn is_merle_tree_valid(&self) -> bool {
        merkle_root_from_hashes(self.transaction_hashes())
            .map_or(false, |root| self.base.total_transactions == 0 || root == self.base.merkle_root())
    }

    fn can_calculate_difficulty_with_previous_blocks(&self, blocks: &HashMap<UInt256, Kind>) -> bool {
        self.base.can_calculate_difficulty_with_previous_blocks(blocks)
    }

    fn verify_difficulty_with_previous_blocks(&self, blocks: &HashMap<UInt256, Kind>) -> (bool, u32) {
        self.base.verify_difficulty_with_previous_blocks(blocks)
    }
}
