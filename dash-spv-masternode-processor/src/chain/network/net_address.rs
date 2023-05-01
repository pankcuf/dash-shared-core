use crate::consensus::Encodable;

#[derive(Debug, Default)]
pub struct NetAddress {
    pub address: u32,
    pub port: u16,
    pub services: u64,
}

impl NetAddress {
    pub fn new(address: u32, port: u16, services: u64) -> Self {
        Self { address, port, services }
    }
}

// impl<'a> byte::TryWrite<byte::ctx::Endian> for NetAddress {
//     fn try_write(self, bytes: &mut [u8], _endian: byte::ctx::Endian) -> byte::Result<usize> {
//         let offset: &mut usize = &mut 0;
//         *offset += self.services.enc(bytes);
//         *offset += b"\0\0\0\0\0\0\0\0\0\0\xFF\xFF".enc(bytes); // IPv4 mapped IPv6 header
//         *offset += self.address.to_be_bytes().enc(bytes);
//         *offset += self.port.to_be_bytes().enc(bytes);
//         Ok(*offset)
//     }
// }

impl Encodable for NetAddress {
    #[inline]
    fn consensus_encode<S: std::io::Write>(&self, mut s: S) -> Result<usize, std::io::Error> {
        self.services.enc(&mut s);
        b"\0\0\0\0\0\0\0\0\0\0\xFF\xFF".enc(&mut s);
        self.address.to_be_bytes().enc(&mut s);
        self.port.to_be_bytes().enc(&mut s);
        Ok(std::mem::size_of::<NetAddress>())
    }
}

