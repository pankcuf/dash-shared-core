use byte::ctx::Endian;
use byte::{BytesExt, TryRead};
#[cfg(feature = "generate-dashj-tests")]
use serde::{Serialize, Serializer};
use crate::crypto::byte_util::BytesDecodable;

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Hash, Ord)]
pub enum MasternodeType {
    Regular = 0,
    HighPerformance = 1,
}

#[cfg(feature = "generate-dashj-tests")]
impl Serialize for MasternodeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_u16(u16::from(*self))
    }
}

impl MasternodeType {
    pub fn voting_weight(&self) -> i32 {
        match self {
            MasternodeType::Regular => 1,
            MasternodeType::HighPerformance => 4,
        }
    }
    pub fn collateral_amount(&self) -> u64 {
        match self {
            MasternodeType::Regular => 1000,
            MasternodeType::HighPerformance => 4000,
        }
    }
}

impl From<u16> for MasternodeType {
    fn from(orig: u16) -> Self {
        match orig {
            0 => Self::Regular,
            1 => Self::HighPerformance,
            i => panic!("Unknown MasternodeType {}", i)
        }
    }
}
impl From<MasternodeType> for u16 {
    fn from(orig: MasternodeType) -> Self {
        match orig {
            MasternodeType::Regular => 0,
            MasternodeType::HighPerformance => 1,
        }
    }
}
impl<'a> TryRead<'a, Endian> for MasternodeType {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let orig = bytes.read_with::<u16>(offset, endian).unwrap();
        let masternode_type = MasternodeType::from(orig);
        Ok((masternode_type, 2))
    }
}

impl<'a> BytesDecodable<'a, MasternodeType> for MasternodeType {
    fn from_bytes(bytes: &'a [u8], offset: &mut usize) -> Option<MasternodeType> {
        bytes.read_with::<MasternodeType>(offset, byte::LE).ok()
    }
}
