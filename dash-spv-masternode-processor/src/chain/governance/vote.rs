use byte::{BytesExt, LE, TryRead};
use byte::ctx::Bytes;
use crate::consensus::Encodable;
use crate::crypto::UInt256;
// use crate::chain::masternode::MasternodeEntry;
use crate::chain::common::ChainType;
use crate::chain::governance;
use crate::chain::governance::{VoteOutcome, VoteSignal};
use crate::crypto::UTXO;
use crate::models::MasternodeEntry;

#[derive(Debug, Default)]
pub struct Vote {
    pub object: Option<governance::Object>,
    pub masternode: Option<MasternodeEntry>,
    pub outcome: VoteOutcome,
    pub signal: VoteSignal,
    pub created_at: u64,
    pub signature: Vec<u8>,
    pub parent_hash: UInt256,
    pub masternode_utxo: UTXO,
    pub vote_hash: UInt256,
    pub chain_type: ChainType,
}

impl<'a> TryRead<'a, ChainType> for Vote {
    fn try_read(bytes: &'a [u8], chain_type: ChainType) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let masternode_utxo = bytes.read_with::<UTXO>(offset, LE)?;
        if chain_type.protocol_version() < 70209 {
            // switch to outpoint in 70209
            let sigscript_size = bytes.read_with::<u8>(offset, LE)?;
            let _sigscript: &[u8] = bytes.read_with(offset, Bytes::Len(sigscript_size as usize))?;
            let _sequence_number = bytes.read_with::<u32>(offset, LE)?;
        }
        let parent_hash = bytes.read_with::<UInt256>(offset, LE)?;
        let outcome = bytes.read_with::<VoteOutcome>(offset, LE)?;
        let signal = bytes.read_with::<VoteSignal>(offset, LE)?;
        let created_at = bytes.read_with::<u64>(offset, LE)?;
        let message_signature_size = bytes.read_with::<u8>(offset, LE)?;
        let signature: &[u8] = bytes.read_with(offset, Bytes::Len(message_signature_size as usize))?;
        let signal_u32 = u32::from(signal.clone());
        let outcome_u32 = u32::from(outcome.clone());
        Ok((Self {
            object: None,
            masternode: None,
            outcome,
            signal,
            created_at,
            signature: signature.to_vec(),
            parent_hash,
            masternode_utxo,
            vote_hash: Self::hash_with_parent_hash(&parent_hash, created_at, signal_u32, outcome_u32, &masternode_utxo),
            chain_type
        }, *offset))
    }
}


impl Vote {

    fn hash_with_parent_hash(parent_hash: &UInt256, timestamp: u64, signal: u32, outcome: u32, masternode_utxo: &UTXO) -> UInt256 {
        let mut writer: Vec<u8> = Vec::new();
        masternode_utxo.enc(&mut writer);
        0u8.enc(&mut writer);
        u32::MAX.enc(&mut writer);
        parent_hash.enc(&mut writer);
        signal.enc(&mut writer);
        outcome.enc(&mut writer);
        timestamp.enc(&mut writer);
        UInt256::sha256d(writer)
    }

    pub fn data_message(&self) -> Vec<u8> {
        let mut writer: Vec<u8> = Vec::new();
        self.masternode_utxo.enc(&mut writer);
        if self.chain_type.protocol_version() < 70209 {
            // switch to outpoint in 70209
            0u8.enc(&mut writer);
            u32::MAX.enc(&mut writer);
        }
        self.parent_hash.enc(&mut writer);
        u32::from(self.outcome.clone()).enc(&mut writer);
        u32::from(self.signal.clone()).enc(&mut writer);
        self.created_at.enc(&mut writer);
        self.signature.to_vec().enc(&mut writer);
        writer
    }
}
