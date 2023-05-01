use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use crate::chain::{block, block::MAX_TIME_DRIFT, Chain, ChainLock, common::ChainType};
use crate::crypto::UInt256;
use crate::util::{Shared, TimeUtil};

pub trait IBlock: Send + Sync {
    fn chain(&self) -> Shared<Chain>;
    fn chain_type(&self) -> ChainType;
    fn height(&self) -> u32;
    fn set_height(&mut self, height: u32);
    fn block_hash(&self) -> UInt256;
    fn merkle_root(&self) -> UInt256;
    fn prev_block(&self) -> UInt256;
    fn target(&self) -> u32;
    fn to_data(&self) -> Vec<u8>;
    fn timestamp(&self) -> u32;
    fn transaction_hashes(&self) -> Vec<UInt256>;
    fn chain_work(&self) -> UInt256;
    fn set_chain_work(&mut self, chain_work: UInt256);
    fn set_chain_locked_with_chain_lock(&mut self, chain_lock: Arc<ChainLock>);
    // v14
    fn set_chain_locked_with_equivalent_block(&mut self, block: &dyn IBlock);

    fn chain_locked(&self) -> bool;
    fn has_unverified_chain_lock(&self) -> bool;
    fn chain_lock_awaiting_processing(&self) -> Option<Arc<ChainLock>>;
    // true if merkle tree and timestamp are valid
    // NOTE: This only checks if the block difficulty matches the difficulty target in the header. It does not check if the
    // target is correct for the block's height in the chain. Use verifyDifficultyFromPreviousBlock: for that.
    fn is_valid(&self) -> bool {
        if !self.is_merle_tree_valid() {
            return false;
        }
        // check if timestamp is too far in future
        // TODO: use estimated network time instead of system time (avoids timejacking attacks and misconfigured time)
        self.timestamp() <= (SystemTime::seconds_since_1970() + MAX_TIME_DRIFT) as u32
    }
    fn is_merle_tree_valid(&self) -> bool;
    fn can_calculate_difficulty_with_previous_blocks(&self, blocks: &HashMap<UInt256, block::Kind>) -> bool;
    fn verify_difficulty_with_previous_blocks(&self, blocks: &HashMap<UInt256, block::Kind>) -> (bool, u32);
}
