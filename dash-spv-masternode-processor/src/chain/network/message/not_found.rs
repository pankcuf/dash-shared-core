use std::net::SocketAddr;
use byte::{BytesExt, TryRead};
use crate::chain::network::InvType;
use crate::chain::network::message::inv_hash::InvHash;
use crate::consensus::encode::VarInt;
use crate::crypto::UInt256;

#[derive(Clone, Debug, Default)]
pub struct NotFound {
    pub tx_hashes: Vec<UInt256>,
    pub tx_lock_request_hashes: Vec<UInt256>,
    pub block_hashes: Vec<UInt256>,
}

impl<'a> TryRead<'a, SocketAddr> for NotFound {
    fn try_read(bytes: &'a [u8], socket_addr: SocketAddr) -> byte::Result<(Self, usize)> {
        let mut offset = 0usize;
        let count = bytes.read_with::<VarInt>(&mut offset, byte::LE).unwrap();
        let l = count.len();
        let size = count.0 as usize;
        if l == 0 || bytes.len() < l + size * 36 {
            println!("malformed notfound message, length is {}, should be {} for {} items", bytes.len(), if l == 0 { 1 } else { l } + size * 36, size);
            return Err(byte::Error::Incomplete);
        }
        println!("{} got notfound with {} item{}", socket_addr, size, if size == 1 { "" } else { "s" });
        let mut tx_hashes = Vec::<UInt256>::new();
        let mut tx_lock_request_hashes = Vec::<UInt256>::new();
        let mut block_hashes = Vec::<UInt256>::new();
        while let Some(data) = bytes.read_iter::<InvHash>(&mut offset, byte::LE).next() {
            match data.r#type {
                InvType::Tx => tx_hashes.push(data.hash),
                InvType::TxLockRequest => tx_lock_request_hashes.push(data.hash),
                InvType::Merkleblock => block_hashes.push(data.hash),
                _ => {}
            }
        }
        Ok((Self { tx_hashes, tx_lock_request_hashes, block_hashes }, offset))
    }
}
