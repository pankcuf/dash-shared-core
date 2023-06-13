use std::io;
use std::io::{Error, Write};
use byte::ctx::Endian;
use byte::{BytesExt, TryRead, TryWrite};
#[cfg(feature = "generate-dashj-tests")]
use serde::{Serialize, Serializer};
use crate::consensus::{Decodable, Encodable, encode};
use crate::crypto::byte_util::BytesDecodable;

#[warn(non_camel_case_types)]
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Hash, Ord)]
pub enum LLMQVersion {
    Default = 1,
    Indexed = 2,
    BLSBasicDefault = 3,
    BLSBasicIndexed = 4,
}

#[cfg(feature = "generate-dashj-tests")]
impl Serialize for LLMQVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_u16(u16::from(*self))
    }
}

impl LLMQVersion {
    pub fn use_bls_legacy(&self) -> bool {
        *self == Self::Default || *self == Self::Indexed
    }
    pub fn use_rotated_quorums(&self) -> bool {
        *self == Self::Indexed || *self == Self::BLSBasicIndexed
    }
}

impl From<u16> for LLMQVersion {
    fn from(orig: u16) -> Self {
        match orig {
            1 => LLMQVersion::Default,
            2 => LLMQVersion::Indexed,
            3 => LLMQVersion::BLSBasicDefault,
            4 => LLMQVersion::BLSBasicIndexed,
            _ => LLMQVersion::Default,
        }
    }
}

impl From<LLMQVersion> for u16 {
    fn from(value: LLMQVersion) -> Self {
        match value {
            LLMQVersion::Default => 1,
            LLMQVersion::Indexed => 2,
            LLMQVersion::BLSBasicDefault => 3,
            LLMQVersion::BLSBasicIndexed => 4,
        }
    }
}

impl Encodable for LLMQVersion {
    fn consensus_encode<W: Write>(&self, writer: W) -> Result<usize, Error> {
        u16::from(*self)
            .consensus_encode(writer)
    }
}

impl Decodable for LLMQVersion {
    fn consensus_decode<D: io::Read>(mut d: D) -> Result<Self, encode::Error> {
        u16::consensus_decode(&mut d)
            .map(Into::into)
    }
}


impl<'a> TryRead<'a, Endian> for LLMQVersion {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let orig = bytes.read_with::<u16>(offset, endian).unwrap();
        Ok((LLMQVersion::from(orig), 2))
    }
}

impl<'a> TryWrite<Endian> for LLMQVersion {
    fn try_write(self, bytes: &mut [u8], _endian: Endian) -> byte::Result<usize> {
        let orig: u16 = self.into();
        orig.consensus_encode(bytes).unwrap();
        Ok(2)
    }
}
impl<'a> BytesDecodable<'a, LLMQVersion> for LLMQVersion {
    fn from_bytes(bytes: &'a [u8], offset: &mut usize) -> Option<LLMQVersion> {
        match bytes.read_with::<LLMQVersion>(offset, byte::LE) {
            Ok(data) => Some(data),
            Err(_err) => None,
        }
    }
}
