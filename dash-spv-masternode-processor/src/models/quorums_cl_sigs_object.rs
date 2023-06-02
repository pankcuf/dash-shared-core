use byte::ctx::Endian;
use byte::{BytesExt, LE, TryRead};
use crate::consensus::encode::VarInt;
use crate::crypto::UInt768;

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct QuorumsCLSigsObject {
    pub signature: UInt768,
    pub index_set: Vec<u16>,
}

impl std::fmt::Debug for QuorumsCLSigsObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuorumsCLSigsObject")
            .field("signature", &self.signature)
            .field("index_set", &self.index_set)
            .finish()
    }
}

impl<'a> TryRead<'a, Endian> for QuorumsCLSigsObject {
    fn try_read(bytes: &'a [u8], _ctx: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let signature = bytes.read_with::<UInt768>(offset, LE)?;
        let index_set_length = bytes.read_with::<VarInt>(offset, LE)?.0 as usize;
        let mut index_set = Vec::with_capacity(index_set_length);
        for _i in 0..index_set_length {
            index_set.push(bytes.read_with::<u16>(offset, LE)?);
        }
        let entry = Self {
            signature,
            index_set
        };
        Ok((entry, *offset))
    }
}
