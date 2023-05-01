use byte::ctx::Endian;
use byte::{BytesExt, TryRead};

#[repr(u16)]
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Hash, Ord)]
pub enum Identifier {
    #[default]
    Unknown = 0,
    Spork2InstantSendEnabled = 10001,
    Spork3InstantSendBlockFiltering = 10002,
    Spork5InstantSendMaxValue = 10004,
    Spork6NewSigs = 10005,
    Spork8MasternodePaymentEnforcement = 10007,
    Spork9SuperblocksEnabled = 10008,
    Spork10MasternodePayUpdatedNodes = 10009,
    Spork12ReconsiderBlocks = 10011,
    Spork13OldSuperblockFlag = 10012,
    Spork14RequireSentinelFlag = 10013,
    Spork15DeterministicMasternodesEnabled = 10014,
    Spork16InstantSendAutoLocks = 10015,
    Spork17QuorumDKGEnabled = 10016,
    Spork18QuorumDebugEnabled = 10017,
    Spork19ChainLocksEnabled = 10018,
    Spork20InstantSendLLMQBased = 10019,
    Spork21QuorumAllConnected = 10020,
    Spork22PSMoreParticipants = 10021,
    Spork23QuorumPoseConnected = 10022
}

impl Identifier {
    pub fn as_str(&self) -> &str {
        match self {
            Identifier::Spork2InstantSendEnabled => "Instant Send enabled",
            Identifier::Spork3InstantSendBlockFiltering => "Instant Send block filtering",
            Identifier::Spork5InstantSendMaxValue => "Instant Send max value",
            Identifier::Spork6NewSigs => "New Signature/Message Format",
            Identifier::Spork8MasternodePaymentEnforcement => "Masternode payment enforcement",
            Identifier::Spork9SuperblocksEnabled => "Superblocks enabled",
            Identifier::Spork10MasternodePayUpdatedNodes => "Masternode pay updated nodes",
            Identifier::Spork12ReconsiderBlocks => "Reconsider blocks",
            Identifier::Spork13OldSuperblockFlag => "Old superblock flag",
            Identifier::Spork14RequireSentinelFlag => "Require sentinel flag",
            Identifier::Spork15DeterministicMasternodesEnabled => "DML enabled at block",
            Identifier::Spork16InstantSendAutoLocks => "Instant Send auto-locks",
            Identifier::Spork17QuorumDKGEnabled => "Quorum DKG enabled",
            Identifier::Spork18QuorumDebugEnabled => "Quorum debugging enabled",
            Identifier::Spork19ChainLocksEnabled => "Chain locks enabled",
            Identifier::Spork20InstantSendLLMQBased => "LLMQ based Instant Send",
            Identifier::Spork21QuorumAllConnected => "Quorum All connected",
            Identifier::Spork22PSMoreParticipants => "PS More Participants",
            Identifier::Spork23QuorumPoseConnected => "Quorum PoSe connected",
            _ => "Unknown spork"
        }
    }
}
// TODO: write macro
impl From<u16> for Identifier {
    fn from(orig: u16) -> Self {
        match orig {
            10001 => Identifier::Spork2InstantSendEnabled,
            10002 => Identifier::Spork3InstantSendBlockFiltering,
            10004 => Identifier::Spork5InstantSendMaxValue,
            10005 => Identifier::Spork6NewSigs,
            10007 => Identifier::Spork8MasternodePaymentEnforcement,
            10008 => Identifier::Spork9SuperblocksEnabled,
            10009 => Identifier::Spork10MasternodePayUpdatedNodes,
            10011 => Identifier::Spork12ReconsiderBlocks,
            10012 => Identifier::Spork13OldSuperblockFlag,
            10013 => Identifier::Spork14RequireSentinelFlag,
            10014 => Identifier::Spork15DeterministicMasternodesEnabled,
            10015 => Identifier::Spork16InstantSendAutoLocks,
            10016 => Identifier::Spork17QuorumDKGEnabled,
            10017 => Identifier::Spork18QuorumDebugEnabled,
            10018 => Identifier::Spork19ChainLocksEnabled,
            10019 => Identifier::Spork20InstantSendLLMQBased,
            10020 => Identifier::Spork21QuorumAllConnected,
            10021 => Identifier::Spork22PSMoreParticipants,
            10022 => Identifier::Spork23QuorumPoseConnected,
            _ => Identifier::Unknown,
        }
    }
}

impl From<i32> for Identifier {
    fn from(orig: i32) -> Self {
        (orig as u16).into()
    }
}

impl From<Identifier> for u16 {
    fn from(value: Identifier) -> Self {
        match value {
            Identifier::Spork2InstantSendEnabled => 10001,
            Identifier::Spork3InstantSendBlockFiltering => 10002,
            Identifier::Spork5InstantSendMaxValue => 10004,
            Identifier::Spork6NewSigs => 10005,
            Identifier::Spork8MasternodePaymentEnforcement => 10007,
            Identifier::Spork9SuperblocksEnabled => 10008,
            Identifier::Spork10MasternodePayUpdatedNodes => 10009,
            Identifier::Spork12ReconsiderBlocks => 10011,
            Identifier::Spork13OldSuperblockFlag => 10012,
            Identifier::Spork14RequireSentinelFlag => 10013,
            Identifier::Spork15DeterministicMasternodesEnabled => 10014,
            Identifier::Spork16InstantSendAutoLocks => 10015,
            Identifier::Spork17QuorumDKGEnabled => 10016,
            Identifier::Spork18QuorumDebugEnabled => 10017,
            Identifier::Spork19ChainLocksEnabled => 10018,
            Identifier::Spork20InstantSendLLMQBased => 10019,
            Identifier::Spork21QuorumAllConnected => 10020,
            Identifier::Spork22PSMoreParticipants => 10021,
            Identifier::Spork23QuorumPoseConnected => 10022,
            _ => 0
        }
    }
}

impl From<Identifier> for i32 {
    fn from(value: Identifier) -> Self {
        let s_16: u16 = value.into();
        s_16 as i32
    }
}

impl<'a> TryRead<'a, Endian> for Identifier {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let orig = bytes.read_with::<u16>(&mut 0, endian).unwrap();
        let data = Identifier::from(orig);
        Ok((data, std::mem::size_of::<u16>()))
    }
}


// impl Encodable for Identifier {
//     #[inline]
//     fn consensus_encode<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
//         let s_16: u16 = self.into();
//         writer.emit_slice(&s_16.to_le_bytes())?;
//         Ok(2) //u16
//     }
// }
