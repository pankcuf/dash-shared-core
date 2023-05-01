use std::collections::HashMap;
use std::sync::Arc;
use crate::chain::block::{Block, FullBlock, IBlock, MerkleBlock};
use crate::chain::{Chain, ChainLock, common::ChainType};
use crate::crypto::UInt256;
use crate::util::Shared;

#[derive(Clone, Debug)]
pub enum Kind {
    Block(Block),
    MerkleBlock(MerkleBlock),
    FullBlock(FullBlock),
}

impl PartialEq for Kind {
    fn eq(&self, other: &Self) -> bool {
        self.height() == other.height() &&
            self.block_hash() == other.block_hash() &&
            self.merkle_root() == other.merkle_root() &&
            self.prev_block() == other.prev_block() &&
            self.target() == other.target() &&
            self.timestamp() == other.timestamp()
    }
}
impl Kind {
    pub fn block_mut(&mut self) -> &mut dyn IBlock {
        match self {
            Kind::Block(block) => block,
            Kind::MerkleBlock(block) => block,
            Kind::FullBlock(block) => block,
        }
    }
    pub fn block(&self) -> &dyn IBlock {
        match self {
            Kind::Block(block) => block,
            Kind::MerkleBlock(block) => block,
            Kind::FullBlock(block) => block,
        }
    }
}

impl IBlock for Kind {
    fn chain(&self) -> Shared<Chain> {
        self.block().chain()
    }

    fn chain_type(&self) -> ChainType {
        self.block().chain_type()
    }

    fn height(&self) -> u32 {
        self.block().height()
    }

    fn set_height(&mut self, height: u32) {
        self.block_mut().set_height(height);
    }

    fn block_hash(&self) -> UInt256 {
        self.block().block_hash()
    }

    fn merkle_root(&self) -> UInt256 {
        self.block().merkle_root()
    }

    fn prev_block(&self) -> UInt256 {
        self.block().prev_block()
    }

    fn target(&self) -> u32 {
        self.block().target()
    }

    fn to_data(&self) -> Vec<u8> {
        self.block().to_data()
    }

    fn timestamp(&self) -> u32 {
        self.block().timestamp()
    }

    fn transaction_hashes(&self) -> Vec<UInt256> {
        self.block().transaction_hashes()
    }

    fn chain_work(&self) -> UInt256 {
        self.block().chain_work()
    }

    fn set_chain_work(&mut self, chain_work: UInt256) {
        self.block_mut().set_chain_work(chain_work);
    }

    fn set_chain_locked_with_chain_lock(&mut self, chain_lock: Arc<ChainLock>) {
        self.block_mut().set_chain_locked_with_chain_lock(chain_lock);
    }

    fn set_chain_locked_with_equivalent_block(&mut self, block: &dyn IBlock) {
        self.block_mut().set_chain_locked_with_equivalent_block(block);
    }

    fn chain_locked(&self) -> bool {
        self.block().chain_locked()
    }

    fn has_unverified_chain_lock(&self) -> bool {
        self.block().has_unverified_chain_lock()
    }

    fn chain_lock_awaiting_processing(&self) -> Option<Arc<ChainLock>> {
        self.block().chain_lock_awaiting_processing()
    }

    fn is_valid(&self) -> bool {
        self.block().is_valid()
    }

    fn is_merle_tree_valid(&self) -> bool {
        self.block().is_merle_tree_valid()
    }

    fn can_calculate_difficulty_with_previous_blocks(&self, blocks: &HashMap<UInt256, Kind>) -> bool {
        self.block().can_calculate_difficulty_with_previous_blocks(blocks)
    }

    fn verify_difficulty_with_previous_blocks(&self, blocks: &HashMap<UInt256, Kind>) -> (bool, u32) {
        self.block().verify_difficulty_with_previous_blocks(blocks)
    }
}
