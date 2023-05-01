use byte::BytesExt;
use hashes::hex::ToHex;
use std::collections::BTreeMap;
use crate::chain::common::LLMQType;
use crate::consensus::encode::VarInt;
use crate::crypto::byte_util::{BytesDecodable, Reversable};
use crate::crypto::var_array::VarArray;
use crate::crypto::UInt256;
use crate::models::{LLMQEntry, MasternodeEntry};
use crate::models::masternode_entry::MasternodeReadContext;
use crate::tx::CoinbaseTransaction;

#[derive(Clone)]
pub struct MNListDiff {
    pub base_block_hash: UInt256,
    pub block_hash: UInt256,
    pub total_transactions: u32,
    pub merkle_hashes: Vec<UInt256>,
    pub merkle_flags: Vec<u8>,
    pub coinbase_transaction: CoinbaseTransaction,
    pub deleted_masternode_hashes: Vec<UInt256>,
    pub added_or_modified_masternodes: BTreeMap<UInt256, MasternodeEntry>,
    pub deleted_quorums: BTreeMap<LLMQType, Vec<UInt256>>,
    pub added_quorums: BTreeMap<LLMQType, BTreeMap<UInt256, LLMQEntry>>,
    pub base_block_height: u32,
    pub block_height: u32,
    // 0: protocol_version < 20225
    // 1: all pubKeyOperator of all CSimplifiedMNListEntry are serialised using legacy BLS scheme
    // 2: all pubKeyOperator of all CSimplifiedMNListEntry are serialised using basic BLS scheme
    pub version: u16,
}

impl std::fmt::Debug for MNListDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MNListDiff")
            .field("base_block_hash", &self.base_block_hash)
            .field("block_hash", &self.block_hash)
            .field("total_transactions", &self.total_transactions)
            .field("merkle_hashes", &self.merkle_hashes)
            .field("merkle_flags", &self.merkle_flags.to_hex())
            .field("merkle_flags_count", &self.merkle_flags.len())
            .field("coinbase_transaction", &self.coinbase_transaction)
            .field("deleted_masternode_hashes", &self.deleted_masternode_hashes)
            .field("added_or_modified_masternodes", &self.added_or_modified_masternodes)
            .field("deleted_quorums", &self.deleted_quorums)
            .field("added_quorums", &self.added_quorums)
            .field("base_block_height", &self.base_block_height)
            .field("block_height", &self.block_height)
            .field("version", &self.version)
            .finish()
    }
}

impl MNListDiff {
    pub fn new<F: Fn(UInt256) -> u32>(
        message: &[u8],
        offset: &mut usize,
        block_height_lookup: F,
        protocol_version: u32,
    ) -> Option<Self> {
        let base_block_hash = UInt256::from_bytes(message, offset)?;
        let block_hash = UInt256::from_bytes(message, offset)?;
        let base_block_height = block_height_lookup(base_block_hash);
        let block_height = block_height_lookup(block_hash);
        let total_transactions = u32::from_bytes(message, offset)?;
        let merkle_hashes = VarArray::<UInt256>::from_bytes(message, offset)?;
        let merkle_flags_count = VarInt::from_bytes(message, offset)?.0 as usize;
        let merkle_flags: &[u8] = match message.read_with(offset, byte::ctx::Bytes::Len(merkle_flags_count)) {
            Ok(data) => data,
            Err(_err) => { return None; },
        };
        let coinbase_transaction = CoinbaseTransaction::from_bytes(message, offset)?;
        let version = if protocol_version >= 70225 {
            // BLS Basic
            u16::from_bytes(message, offset)?
        } else {
            // BLS Legacy
            0
        };
        let masternode_read_ctx = MasternodeReadContext(block_height, version);
        let deleted_masternode_count = VarInt::from_bytes(message, offset)?.0;
        let mut deleted_masternode_hashes: Vec<UInt256> =
            Vec::with_capacity(deleted_masternode_count as usize);
        for _i in 0..deleted_masternode_count {
            deleted_masternode_hashes.push(UInt256::from_bytes(message, offset)?);
        }
        let added_masternode_count = VarInt::from_bytes(message, offset)?.0;
        let added_or_modified_masternodes: BTreeMap<UInt256, MasternodeEntry> = (0..added_masternode_count)
            .fold(BTreeMap::new(), |mut acc, _| {
                if let Ok(entry) = message.read_with::<MasternodeEntry>(offset, masternode_read_ctx) {
                    acc.insert(entry.provider_registration_transaction_hash.reversed(), entry);
                }
                acc
            });

        let mut deleted_quorums: BTreeMap<LLMQType, Vec<UInt256>> = BTreeMap::new();
        let mut added_quorums: BTreeMap<LLMQType, BTreeMap<UInt256, LLMQEntry>> = BTreeMap::new();
        let quorums_active = coinbase_transaction.coinbase_transaction_version >= 2;
        if quorums_active {
            let deleted_quorums_count = VarInt::from_bytes(message, offset)?.0;
            for _i in 0..deleted_quorums_count {
                let llmq_type = LLMQType::from_bytes(message, offset)?;
                let llmq_hash = UInt256::from_bytes(message, offset)?;
                deleted_quorums
                    .entry(llmq_type)
                    .or_insert_with(Vec::new)
                    .push(llmq_hash);
            }
            let added_quorums_count = VarInt::from_bytes(message, offset)?.0;
            for _i in 0..added_quorums_count {
                if let Some(entry) = LLMQEntry::from_bytes(message, offset) {
                    added_quorums
                        .entry(entry.llmq_type)
                        .or_insert_with(BTreeMap::new)
                        .insert(entry.llmq_hash, entry);
                }
            }
        }
        Some(Self {
            base_block_hash,
            block_hash,
            total_transactions,
            merkle_hashes: merkle_hashes.1,
            merkle_flags: merkle_flags.to_vec(),
            coinbase_transaction,
            deleted_masternode_hashes,
            added_or_modified_masternodes,
            deleted_quorums,
            added_quorums,
            base_block_height,
            block_height,
            version
        })
    }
}
