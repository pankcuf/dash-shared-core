use std::io;
use byte::{BytesExt, TryRead};
use crate::consensus::Encodable;
use crate::crypto::UInt128;

#[repr(C)]
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub struct SocketAddress {
    pub ip_address: UInt128, //v6, but only v4 supported
    pub port: u16,
}

impl std::fmt::Display for SocketAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip_address, self.port)?;
        Ok(())
    }
}

impl Encodable for SocketAddress {
    #[inline]
    fn consensus_encode<S: io::Write>(&self, mut s: S) -> Result<usize, io::Error> {
        self.ip_address.enc(&mut s);
        self.port.swap_bytes().enc(&mut s);
        Ok(18)
    }
}

impl<'a> TryRead<'a, ()> for SocketAddress {
    fn try_read(bytes: &'a [u8], context: ()) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let ip_address = bytes.read_with::<UInt128>(offset, byte::LE)?;
        let port = bytes.read_with::<u16>(offset, byte::BE)?;
        Ok((SocketAddress { ip_address, port }, *offset))
    }
}
