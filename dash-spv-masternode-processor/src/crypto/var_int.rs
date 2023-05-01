use byte::ctx::Endian;
use byte::{BytesExt, check_len, LE, TryRead, TryWrite};
use crate::consensus::{Decodable, Encodable};
use crate::consensus::encode::VarInt;
use crate::crypto::byte_util::BytesDecodable;
use crate::impl_bytes_decodable;

impl<'a> TryRead<'a, Endian> for VarInt {
    #[inline]
    fn try_read(bytes: &'a [u8], _endian: Endian) -> byte::Result<(Self, usize)> {
        match VarInt::consensus_decode(bytes) {
            Ok(data) => Ok((data, data.len())),
            Err(_err) => Err(byte::Error::BadInput { err: "Error: VarInt" })
        }
    }
}

impl TryWrite for VarInt {
    #[inline]
    fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
        check_len(bytes, self.len())?;
        Ok(match self.consensus_encode(bytes) {
            Ok(size) => size,
            _ => 0
        })
    }
}

// impl<'a> TryWrite for &'a VarInt {
//     #[inline]
//     fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
//         check_len(bytes, self.len())?;
//         Ok(match self.consensus_encode(bytes) {
//             Ok(size) => size,
//             _ => 0
//         })
//     }
// }

impl_bytes_decodable!(VarInt);
