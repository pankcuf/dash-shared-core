use std::collections::HashMap;
use std::sync::Arc;
use byte::{BytesExt, TryRead};
use crate::chain::block::{Block, BLOCK_UNKNOWN_HEIGHT, IBlock, Kind};
use crate::chain::{Chain, ChainLock, Checkpoint};
use crate::chain::common::ChainType;
use crate::common::MerkleTree;
use crate::consensus::{Encodable, encode::VarInt};
use crate::crypto::{byte_util::Zeroable, UInt256};
use crate::util::Shared;

#[derive(Clone, Debug, Default)]
pub struct MerkleBlock {
    pub base: Block,
    pub merkle_tree: MerkleTree,
}

impl PartialEq for MerkleBlock {
    fn eq(&self, other: &Self) -> bool {
        self == other || self.block_hash() == other.block_hash()
        // return self == obj || ([obj isMemberOfClass:[self class]] && uint256_eq([obj blockHash], _blockHash));
    }
}

#[derive(Clone)]
pub struct ReadContext(pub ChainType, pub Shared<Chain>);

impl<'a> TryRead<'a, ReadContext> for MerkleBlock {
    fn try_read(bytes: &'a [u8], context: ReadContext) -> byte::Result<(Self, usize)> {
        let offset = &mut 0usize;
        assert!(bytes.len() >= 80, "Merkle block message length less than 80");
        let version = bytes.read_with::<u32>(offset, byte::LE)?;
        let prev_block = bytes.read_with::<UInt256>(offset, byte::LE)?;
        let merkle_root = bytes.read_with::<UInt256>(offset, byte::LE)?;
        let timestamp = bytes.read_with::<u32>(offset, byte::LE)?;
        let target = bytes.read_with::<u32>(offset, byte::LE)?;
        let nonce = bytes.read_with::<u32>(offset, byte::LE)?;
        let merkle_tree = bytes.read_with::<MerkleTree>(offset, byte::LE)?;
        let height = BLOCK_UNKNOWN_HEIGHT as u32;
        let mut data = Vec::<u8>::new();
        version.enc(&mut data);
        prev_block.enc(&mut data);
        merkle_root.enc(&mut data);
        timestamp.enc(&mut data);
        target.enc(&mut data);
        nonce.enc(&mut data);
        let block_hash = UInt256::x11_hash(&data);
        Ok((Self {
            base: Block {
                block_hash,
                version,
                prev_block,
                merkle_root,
                timestamp,
                target,
                nonce,
                total_transactions: merkle_tree.tree_element_count,
                height,
                chain_type: context.0,
                chain: context.1,
                ..Default::default()
            },
            merkle_tree
        }, *offset))
    }
}


impl IBlock for MerkleBlock {
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
        self.base.set_height(height);
    }

    fn block_hash(&self) -> UInt256 {
        self.base.block_hash
    }

    fn merkle_root(&self) -> UInt256 {
        self.base.merkle_root
    }

    fn prev_block(&self) -> UInt256 {
        self.base.prev_block
    }

    fn target(&self) -> u32 {
        self.base.target
    }

    fn to_data(&self) -> Vec<u8> {
        let mut writer: Vec<u8> = self.base.to_data();
        if self.base.total_transactions > 0 {
            self.base.total_transactions.enc(&mut writer);
            VarInt(self.merkle_tree.hashes.len() as u64).enc(&mut writer);
            self.merkle_tree.hashes.iter().for_each(|hash| {
                hash.enc(&mut writer);
            });
            VarInt(self.merkle_tree.flags.len() as u64).enc(&mut writer);
            writer.extend_from_slice(&self.merkle_tree.flags);
        }
        writer
    }

    fn timestamp(&self) -> u32 {
        self.base.timestamp()
    }

    fn transaction_hashes(&self) -> Vec<UInt256> {
        self.base.transaction_hashes()
    }

    fn chain_work(&self) -> UInt256 {
        self.base.chain_work()
    }

    fn set_chain_work(&mut self, chain_work: UInt256) {
        self.base.set_chain_work(chain_work);
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
        self.merkle_tree.has_root(self.merkle_root())
    }

    fn can_calculate_difficulty_with_previous_blocks(&self, blocks: &HashMap<UInt256, Kind>) -> bool {
        self.base.can_calculate_difficulty_with_previous_blocks(blocks)
    }

    fn verify_difficulty_with_previous_blocks(&self, blocks: &HashMap<UInt256, Kind>) -> (bool, u32) {
        self.base.verify_difficulty_with_previous_blocks(blocks)
    }
}

impl MerkleBlock {
    // true if the given tx hash is included in the block
    pub fn contains_tx_hash(&self, tx_hash: UInt256) -> bool {
        self.merkle_tree.contains_hash(tx_hash)
    }

    /// returns an array of the matched tx hashes
    pub fn transaction_hashes(&self) -> Vec<UInt256> {
        self.merkle_tree.element_hashes()
    }

    pub fn is_merkle_tree_valid(&self) -> bool {
        self.merkle_tree.has_root(self.base.merkle_root)
    }

    pub fn init_with_checkpoint(checkpoint: &Checkpoint, chain_type: ChainType, chain: Shared<Chain>) -> Self {
        let base = Block::init_with_version(2, checkpoint.timestamp, checkpoint.height, checkpoint.hash, UInt256::MIN, checkpoint.chain_work, checkpoint.merkle_root, checkpoint.target, chain_type, chain);
        assert!(!checkpoint.chain_work.is_zero(), "Chain work must be set");
        Self { base, ..Default::default() }
    }
    pub fn with(version: u32, block_hash: UInt256, prev_block: UInt256, merkle_root: UInt256, timestamp: u32, target: u32, chain_work: UInt256, nonce: u32, total_transactions: u32, hashes: Vec<UInt256>, flags: Vec<u8>, height: u32, chain_lock: Option<Arc<ChainLock>>, chain_type: ChainType, chain: Shared<Chain>) -> Self {
        let mut base = Block::init_with_version(version, timestamp, height, block_hash, prev_block, chain_work, merkle_root, target, chain_type, chain);
        if let Some(lock) = chain_lock {
            base.set_chain_locked_with_chain_lock(lock);
        }
        Self { base, ..Default::default() }
    }

    /*pub fn new(version: u32,
               block_hash: UInt256,
               prev_block: UInt256,
               merkle_root: UInt256,
               timestamp: u32,
               target: u32,
               chain_work: UInt256,
               nonce: u32,
               total_transactions: u32,
               hashes: Vec<UInt256>,
               flags: Vec<u8>,
               height: u32,
               chain_lock: ChainLock,
               chain_type: ChainType,
               chain: Shared<Chain>) -> Self {
        Self {
            base: Block {
                block_hash,
                version,
                prev_block,
                merkle_root,
                timestamp,
                target,
                nonce,
                total_transactions,
                height,
                chain_type,
                chain,
                chain_work,
                ..Default::default()
            },
            merkle_tree: MerkleTree {
                tree_element_count: total_transactions,
                hashes,
                flags,
                hash_function: MerkleTreeHashFunction::SHA256_2
            }
        }
    }*/
}
