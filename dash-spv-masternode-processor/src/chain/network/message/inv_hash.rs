use byte::ctx::Endian;
use byte::{BytesExt, TryRead};
use crate::chain::network::InvType;
use crate::consensus::Encodable;
use crate::crypto::UInt256;

#[derive(Clone, Debug, PartialEq)]
pub struct InvHash {
    pub r#type: InvType,
    pub hash: UInt256,
}

impl<'a> TryRead<'a, Endian> for InvHash {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let r#type = bytes.read_with::<InvType>(offset, endian)?;
        let hash = bytes.read_with::<UInt256>(offset, endian)?;
        Ok((Self { r#type, hash }, *offset))
    }
}

impl Encodable for InvHash {
    #[inline]
    fn consensus_encode<S: std::io::Write>(&self, mut s: S) -> Result<usize, std::io::Error> {
        let type_u32: u32 = self.r#type.into();
        type_u32.enc(&mut s);
        self.hash.enc(&mut s);
        Ok(std::mem::size_of::<InvHash>())
    }
}
