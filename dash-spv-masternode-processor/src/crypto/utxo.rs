use byte::ctx::Endian;
use byte::{BytesExt, TryRead};
use crate::consensus::Encodable;
use crate::crypto::byte_util::{AsBytes, Zeroable};
use crate::crypto::UInt256;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug/*, FromSqlRow, AsExpression*/)]
// #[diesel(foreign_derive)]
// #[diesel(sql_type = diesel::sql_types::Binary)]
pub struct UTXO {
    pub hash: UInt256,
    pub n: u32
    // use unsigned long instead of uint32_t to avoid trailing struct padding (for NSValue comparisons)
}

impl UTXO {
    pub(crate) fn with_index(index: u32) -> UTXO {
        UTXO { hash: UInt256::MIN, n: index }
    }
}

impl Default for UTXO {
    fn default() -> Self {
        UTXO { hash: UInt256::MIN, n: 0 }
    }
}

impl Zeroable for UTXO {
    fn is_zero(&self) -> bool {
        self.hash.is_zero() && self.n == 0
    }
}

impl<'a> TryRead<'a, Endian> for UTXO {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let offset: &mut usize = &mut 0;
        let hash = bytes.read_with::<UInt256>(offset, endian)?;
        let n = bytes.read_with::<u32>(offset, endian)?;
        let data = UTXO { hash, n };
        Ok((data, std::mem::size_of::<UTXO>()))
    }
}

impl Encodable for UTXO {
    #[inline]
    fn consensus_encode<S: std::io::Write>(&self, mut s: S) -> Result<usize, std::io::Error> {
        self.hash.enc(&mut s);
        self.n.enc(&mut s);
        Ok(std::mem::size_of::<UTXO>())
    }
}

impl AsBytes for UTXO {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((self as *const UTXO) as *const u8, std::mem::size_of::<UTXO>() )}
        // let mut vec = Vec::<u8>::new();
        // self.enc(&mut vec);
        // vec[..]
        // &vec
    }
}

// - (DSUTXO)transactionOutpoint {
// if (self.length < 36) return DSUTXO_ZERO;
// return (DSUTXO){.hash = [self UInt256], .n = *(uint32_t *)(self.bytes + 32)};
// }
