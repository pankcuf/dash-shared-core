use std::io::{Error, Write};
use byte::ctx::Endian;
use byte::{BytesExt, TryRead};
use crate::consensus::Encodable;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Hash, Ord)]
pub enum InvType {
    #[default]
    Error = 0,
    // The hash is a TXID
    Tx = 1,
    // The hash is of a block header
    Block = 2,
    // The hash is of a block header; identical to MSG_BLOCK. When used in a getdata message,
    // this indicates the response should be a merkleblock message rather than a block message
    // (only works if a bloom filter was previously configured). Only for use in getdata messages.
    Merkleblock = 3,
    // The hash is an Instant Send transaction lock request. Transactions received this way are
    // automatically converted to a standard tx message as of Dash Core 0.15.0.
    TxLockRequest = 4,
    // The hash is an Instant Send transaction vote. Deprecated in 0.15.0
    // #[deprecated]
    TxLockVote = 5,
    // The hash is Spork ID
    Spork = 6,
    MasternodePaymentVote = 7,
    MasternodePaymentBlock = 8,
    MasternodeBroadcast = 14,
    MasternodePing = 15,
    // The hash is CoinJoin Broadcast TX
    DSTx = 16,
    // The hash is a Governance Object
    GovernanceObject = 17,
    // The hash is a Governance Object Vote
    GovernanceObjectVote = 18,
    MasternodeVerify = 19,
    // The hash is of a block header; identical to Block. When used in a getdata message,
    // this indicates the response should be a cmpctblock message. Only for use in getdata messages
    CompactBlock = 20, // Defined in BIP152
    // The hash is a long-living masternode quorum final commitment. Added in 0.13.0
    QuorumFinalCommitment = 21,
    DummyCommitment = 22, // only valid on testnet/devnet/regtest
    // The hash is a long-living masternode quorum contribution. Added in 0.14.0
    QuorumContribution = 23,
    // The hash is a long-living masternode quorum complaint. Added in 0.14.0
    QuorumComplaint = 24,
    // The hash is a long-living masternode quorum justification. Added in 0.14.0
    QuorumJustification = 25,
    // The hash is a long-living masternode quorum premature commitment. Added in 0.14.0
    QuorumPrematureCommitment = 26,
    QuorumDebugStatus = 27,
    // The hash is a long-living masternode quorum recovered signature.
    // Note: Only relayed to other masternodes in the same quorum and nodes that have sent a
    // qwatch message as of Dash Core 0.17.0
    QuorumRecoveredSignature = 28,
    // The hash is a ChainLock signature. Added in 0.14.0
    ChainLockSignature = 29,
    // The hash is an LLMQ-based InstantSend lock (DIP10). Added in 0.14.0
    InstantSendLock = 30,
    // The hash is an LLMQ-based deterministic InstantSend lock (DIP22). Added in 0.18.0
    InstantSendDeterministicLock = 31,
}

impl From<InvType> for u32 {
    fn from(value: InvType) -> Self {
        match value {
            InvType::Error => 0,
            InvType::Tx => 1,
            InvType::Block => 2,
            InvType::Merkleblock => 3,
            InvType::TxLockRequest => 4,
            InvType::TxLockVote => 5,
            InvType::Spork => 6,
            InvType::MasternodePaymentVote => 7,
            InvType::MasternodePaymentBlock => 8,
            InvType::MasternodeBroadcast => 14,
            InvType::MasternodePing => 15,
            InvType::DSTx => 16,
            InvType::GovernanceObject => 17,
            InvType::GovernanceObjectVote => 18,
            InvType::MasternodeVerify => 19,
            InvType::CompactBlock => 20,
            InvType::QuorumFinalCommitment => 21,
            InvType::DummyCommitment => 22,
            InvType::QuorumContribution => 23,
            InvType::QuorumComplaint => 24,
            InvType::QuorumJustification => 25,
            InvType::QuorumPrematureCommitment => 26,
            InvType::QuorumDebugStatus => 27,
            InvType::QuorumRecoveredSignature => 28,
            InvType::ChainLockSignature => 29,
            InvType::InstantSendLock => 30,
            InvType::InstantSendDeterministicLock => 31
        }
    }
}

impl From<u32> for InvType {
    fn from(orig: u32) -> Self {
        match orig {
            0 => InvType::Error,
            1 => InvType::Tx,
            2 => InvType::Block,
            3 => InvType::Merkleblock,
            4 => InvType::TxLockRequest,
            5 => InvType::TxLockVote,
            6 => InvType::Spork,
            7 => InvType::MasternodePaymentVote,
            8 => InvType::MasternodePaymentBlock,
            14 => InvType::MasternodeBroadcast,
            15 => InvType::MasternodePing,
            16 => InvType::DSTx,
            17 => InvType::GovernanceObject,
            18 => InvType::GovernanceObjectVote,
            19 => InvType::MasternodeVerify,
            20 => InvType::CompactBlock,
            21 => InvType::QuorumFinalCommitment,
            22 => InvType::DummyCommitment,
            23 => InvType::QuorumContribution,
            24 => InvType::QuorumComplaint,
            25 => InvType::QuorumJustification,
            26 => InvType::QuorumPrematureCommitment,
            27 => InvType::QuorumDebugStatus,
            28 => InvType::QuorumRecoveredSignature,
            29 => InvType::ChainLockSignature,
            30 => InvType::InstantSendLock,
            31 => InvType::InstantSendDeterministicLock,
            _ => InvType::Error
        }
    }
}

impl InvType {
    pub fn name(&self) -> &str {
        match self {
            InvType::Tx => "Tx",
            InvType::Block => "Block",
            InvType::Merkleblock => "Merkleblock",
            InvType::TxLockRequest => "TxLockRequest",
            InvType::TxLockVote => "TxLockVote",
            InvType::Spork => "Spork",
            InvType::MasternodePaymentVote => "MasternodePaymentVote",
            InvType::MasternodePaymentBlock => "MasternodePaymentBlock",
            InvType::MasternodeBroadcast => "MasternodeBroadcast",
            InvType::MasternodePing => "MasternodePing",
            InvType::DSTx => "DSTx",
            InvType::GovernanceObject => "GovernanceObject",
            InvType::GovernanceObjectVote => "GovernanceObjectVote",
            InvType::MasternodeVerify => "MasternodeVerify",
            InvType::Error => "Error",
            InvType::CompactBlock => "CompactBlock",
            InvType::DummyCommitment => "DummyCommitment",
            InvType::QuorumContribution => "QuorumContribution",
            InvType::QuorumFinalCommitment => "QuorumFinalCommitment",
            InvType::ChainLockSignature => "ChainLockSignature",
            InvType::InstantSendLock => "InstantSendLock",
            InvType::InstantSendDeterministicLock => "InstantSendDeterministicLock",
            _ => ""
        }
    }
}

impl<'a> TryRead<'a, Endian> for InvType {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let value = bytes.read_with::<u32>(offset, endian).unwrap();
        Ok((InvType::from(value), std::mem::size_of::<u32>()))
    }
}

impl Encodable for InvType {
    fn consensus_encode<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
        // let uu32: u32 = self.into();
        u32::from(*self).enc(&mut writer);
        Ok(4)
    }
}
