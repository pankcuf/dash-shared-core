use std::cmp;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::Arc;
use crate::chain::block::{IBlock, Kind};
use crate::chain::Chain;
use crate::chain::chain_lock::ChainLock;
use crate::chain::checkpoint::Checkpoint;
use crate::chain::common::ChainType;
use crate::consensus::Encodable;
use crate::crypto::byte_util::{AsBytes, UInt256, Zeroable};
use crate::util::Shared;

pub const BLOCK_UNKNOWN_HEIGHT: i32 = i32::MAX;
pub const DGW_PAST_BLOCKS_MIN: i32 = 24;
pub const DGW_PAST_BLOCKS_MAX: i32 = 24;
/// the furthest in the future a block is allowed to be timestamped
pub const MAX_TIME_DRIFT: u64 = 2 * 60 * 60;

#[derive(Clone, Debug, Default)]
pub struct Block {
    pub block_hash: UInt256,
    pub version: u32,
    pub prev_block: UInt256,
    pub merkle_root: UInt256,
    /// time interval since unix epoch
    pub timestamp: u32,
    pub target: u32,
    pub nonce: u32,
    pub total_transactions: u32,
    pub height: u32,
    pub chain_work: UInt256,
    /// the matched tx hashes in the block
    pub transaction_hashes: Vec<UInt256>,
    pub chain_locked: bool,
    pub has_unverified_chain_lock: bool,

    pub chain_lock_awaiting_processing: Option<Arc<ChainLock>>,
    pub chain_lock_awaiting_saving: Option<Arc<ChainLock>>,

    pub chain_type: ChainType,
    pub chain: Shared<Chain>,
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self == other || self.block_hash == other.block_hash()
        // return self == obj || ([obj isMemberOfClass:[self class]] && uint256_eq([obj blockHash], _blockHash));
    }
}

impl std::hash::Hash for Block {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.block_hash.as_bytes());
    }
}

impl IBlock for Block {
    fn chain(&self) -> Shared<Chain> {
        self.chain.clone()
    }

    fn chain_type(&self) -> ChainType {
        self.chain_type
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn set_height(&mut self, height: u32) {
        self.height = height;
    }

    fn block_hash(&self) -> UInt256 {
        self.block_hash
    }

    fn merkle_root(&self) -> UInt256 {
        self.merkle_root
    }

    fn prev_block(&self) -> UInt256 {
        self.prev_block
    }

    fn target(&self) -> u32 {
        self.target
    }

    fn to_data(&self) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        self.version.enc(&mut writer);
        self.prev_block.enc(&mut writer);
        self.merkle_root.enc(&mut writer);
        self.timestamp.enc(&mut writer);
        self.target.enc(&mut writer);
        self.nonce.enc(&mut writer);
        writer
    }

    fn timestamp(&self) -> u32 {
        self.timestamp
    }

    fn transaction_hashes(&self) -> Vec<UInt256> {
        self.transaction_hashes.clone()
    }

    fn chain_work(&self) -> UInt256 {
        self.chain_work
    }

    fn set_chain_work(&mut self, chain_work: UInt256) {
        self.chain_work = chain_work;
    }

    fn set_chain_locked_with_chain_lock(&mut self, chain_lock: Arc<ChainLock>) {
        self.chain_locked = chain_lock.signature_verified;
        self.has_unverified_chain_lock = !chain_lock.signature_verified;
        self.chain_lock_awaiting_processing = self.has_unverified_chain_lock.then_some(chain_lock.clone());
        // if !chain_lock.saved {
        //     chain_lock.borrow_mut().save_initial();
        //     // chain_lock.save_initial();
        //     if !chain_lock.saved {
        //         // it is still not saved
        //         self.chain_lock_awaiting_saving = Some(chain_lock.clone());
        //     }
        // }
    }

    fn set_chain_locked_with_equivalent_block(&mut self, block: &dyn IBlock) {
        if block.block_hash() == self.block_hash() {
            self.chain_locked |= block.chain_locked();
            self.has_unverified_chain_lock |= block.has_unverified_chain_lock();
            if self.has_unverified_chain_lock() {
                self.chain_lock_awaiting_processing = block.chain_lock_awaiting_processing();
            }
        }
    }

    fn chain_locked(&self) -> bool {
        self.chain_locked
    }

    fn has_unverified_chain_lock(&self) -> bool {
        self.has_unverified_chain_lock
    }

    fn chain_lock_awaiting_processing(&self) -> Option<Arc<ChainLock>> {
        self.chain_lock_awaiting_processing.clone()
    }

    fn is_merle_tree_valid(&self) -> bool {
        false
    }

    fn can_calculate_difficulty_with_previous_blocks(&self, blocks: &HashMap<UInt256, Kind>) -> bool {
        match blocks.get(&self.prev_block) {
            Some(previous_block) => {
                if self.prev_block.is_zero() ||
                    previous_block.height() == 0 ||
                    previous_block.height() < DGW_PAST_BLOCKS_MIN as u32 + u32::from(self.chain_type.is_devnet_any()) {
                    return true;
                }
                if self.chain_type.allow_min_difficulty_blocks() {
                    // recent block is more than 2 hours old
                    // if (self.timestamp > (previousBlock.timestamp + 2 * 60 * 60)) {
                    //    return TRUE;
                    // }
                    // recent block is more than 10 minutes old
                    if self.timestamp > previous_block.timestamp() + 600 { // 2.5 * 60 * 4
                        return true;
                    }
                }
                let mut current_block: Option<&Kind> = Some(previous_block);
                let mut block_count = 1;
                // loop over the past n blocks, where n == PastBlocksMax
                while current_block.is_some() && current_block.unwrap().height() > 0 && block_count <= DGW_PAST_BLOCKS_MAX {
                    current_block = blocks.get(&current_block.unwrap().prev_block());
                    block_count += 1;
                    if current_block.is_none() {
                        println!("Could not retrieve previous block");
                        return false;
                    }
                }
                true
            },
            None => false
        }
    }

    fn verify_difficulty_with_previous_blocks(&self, blocks: &HashMap<UInt256, Kind>) -> (bool, u32) {
        if self.chain_type.is_devnet_any() {
            return (true, 0);
        }
        let dark_gravity_wave_target = self.dark_gravity_wave_target_with_previous_blocks(blocks);
        let diff = self.target as i32 - dark_gravity_wave_target as i32;
        if i32::abs(diff) > 1 {
            println!("weird difficulty for block at height {} with target ({}) (off by {})", self.height, UInt256::set_compact_be(self.target as i32), diff);

        }
        // the core client is less precise with a rounding error that can sometimes cause a problem. We are very rarely 1 off
        (i32::abs(diff) < 2, dark_gravity_wave_target)
    }
}



impl Block {
    pub fn init_with_version(version: u32,
                             timestamp: u32,
                             height: u32,
                             block_hash: UInt256,
                             prev_block: UInt256,
                             chain_work: UInt256,
                             merkle_root: UInt256,
                             target: u32,
                             chain_type: ChainType,
                             chain: Shared<Chain>) -> Self {
        Self {
            block_hash,
            version,
            prev_block,
            merkle_root,
            timestamp,
            target,
            height,
            chain_type,
            chain,
            chain_work,
            ..Default::default()
        }
    }

    pub fn init_with_checkpoint(checkpoint: &Checkpoint, chain_type: ChainType, chain: Shared<Chain>) -> Self {
        assert!(!checkpoint.chain_work.is_zero(), "Chain work must be set");
        let b = Self::init_with_version(2, checkpoint.timestamp, checkpoint.height, checkpoint.hash, UInt256::MIN, checkpoint.chain_work, checkpoint.merkle_root, checkpoint.target, chain_type, chain);
        assert!(!b.chain_work.is_zero(), "block should have aggregate work set");
        b
    }

    // current difficulty formula, darkcoin - based on DarkGravity v3
    // original work done by evan duffield, modified for iOS
    fn dark_gravity_wave_target_with_previous_blocks(&self, blocks: &HashMap<UInt256, Kind>) -> u32 {
        match blocks.get(&self.prev_block()) {
            Some(previous_block) => {
                let mut actual_timespan = 0u32;
                let mut last_block_time = 0i64;
                let mut sum_targets = UInt256::MIN;
                let allow_min_difficulty_blocks = self.chain_type.allow_min_difficulty_blocks();
                let max_proof_of_work = self.chain_type.max_proof_of_work();
                let max_proof_of_work_target = self.chain_type.max_proof_of_work_target();
                if self.prev_block.is_zero() ||
                    previous_block.height() == 0 ||
                    previous_block.height() < if self.chain_type.is_devnet_any() { DGW_PAST_BLOCKS_MIN + 1 } else { DGW_PAST_BLOCKS_MIN } as u32 {
                    // This is the first block or the height is < PastBlocksMin
                    // Return minimal required work. (1e0ffff0)
                    return max_proof_of_work_target
                }
                if allow_min_difficulty_blocks {
                    // recent block is more than 2 hours old
                    if self.timestamp > previous_block.timestamp() + 2 * 60 * 60 {
                        println!("Our block is way ahead of previous block {} > {}", self.timestamp, previous_block.timestamp());
                        return max_proof_of_work_target;
                    }
                    // recent block is more than 10 minutes old
                    if self.timestamp > previous_block.timestamp() + 600 {
                        let previous_target = UInt256::set_compact_le(previous_block.target() as i32);
                        let new_target = previous_target.multiply_u32_le(10);
                        return cmp::min(new_target.get_compact_le() as u32, max_proof_of_work_target);
                    }
                }
                let mut current_block: Option<&Kind> = Some(previous_block);
                let mut block_count = 1u32;
                // loop over the past n blocks, where n == PastBlocksMax
                while current_block.is_some() && current_block.unwrap().height() > 0 && block_count <= DGW_PAST_BLOCKS_MAX as u32 {
                    let c_block = current_block.unwrap();
                    // Calculate average difficulty based on the blocks we iterate over in this for loop
                    if block_count <= DGW_PAST_BLOCKS_MIN as u32 {
                        let current_target = UInt256::set_compact_le(c_block.target() as i32);
                        sum_targets = (if block_count == 1 { current_target } else { sum_targets }).add_le(current_target);
                    }
                    let current_block_time = c_block.timestamp();
                    if last_block_time > 0 {
                        // Calculate time difference between previous block and current block
                        // Increment the actual timespan
                        actual_timespan += last_block_time as u32 - current_block_time;
                    }
                    // Set lastBlockTime to the block time for the block in current iteration
                    last_block_time = current_block_time as i64;
                    // if previous_block == nil {
                    //     assert!(current_block);
                    //     break;
                    // }
                    let old_current_block = c_block;
                    if let Some(prev) = blocks.get(&c_block.prev_block()) {
                        current_block = Some(prev);
                    } else {
                        println!("Block {} missing for dark gravity wave calculation", old_current_block.height() - 1);
                        current_block = None;
                    }
                    block_count += 1;
                }
                let block_count_256 = UInt256::from(block_count);
                println!("SumTargets for block {} is {}, blockCount is {}", self.height, sum_targets, block_count_256);
                // dark_target is the difficulty
                let mut dark_target = sum_targets.divide_le(block_count_256);
                // target_timespan is the time that the CountBlocks should have taken to be generated.
                let target_timespan = ((block_count - 1) * 150) as u64;
                println!("Original dark target for block {} is {}", self.height, dark_target);
                println!("Max proof of work is {}", max_proof_of_work);
                // Limit the re-adjustment to 3x or 0.33x
                // We don't want to increase/decrease diff too much.
                let span_d_3 = (target_timespan / 3) as u32;
                if actual_timespan < span_d_3 {
                    actual_timespan = span_d_3;
                }
                let span_m_3 = target_timespan as u32 * 3;
                if actual_timespan > span_m_3 {
                    actual_timespan = span_m_3;
                }
                dark_target = dark_target.multiply_u32_le(actual_timespan);
                let target_timespan_256 = UInt256::from(target_timespan);
                println!("Middle dark target for block {} is {}", self.height, dark_target);
                println!("target_timespan_256 for block {} is {}", self.height, target_timespan_256);
                // Calculate the new difficulty based on actual and target timespan.
                dark_target = dark_target.divide_le(target_timespan_256);
                println!("Final dark target for block {} is {}", self.height, dark_target);
                // If calculated difficulty is lower than the minimal diff, set the new difficulty to be the minimal diff.
                if dark_target.sup(&max_proof_of_work) {
                    println!("Found a block with minimum difficulty");
                    return max_proof_of_work_target;
                }
                // Return the new diff.
                dark_target.get_compact_le() as u32
            },
            _ => 0
        }
    }

    pub fn has_chain_lock_awaiting_saving(&self) -> bool {
        self.chain_lock_awaiting_saving.is_some()
    }

    // pub fn save_associated_chain_lock(&mut self) -> bool {
    //     self.chain_lock_awaiting_saving.take()
    //         .map_or(true, |mut awaiting_lock|
    //             if awaiting_lock.saved {
    //                 true
    //             } else {
    //                 awaiting_lock.save_initial();
    //                 if awaiting_lock.saved {
    //                     self.chain_lock_awaiting_saving = None;
    //                     true
    //                 } else {
    //                     false
    //                 }
    //             }
    //         )
    // }
}
