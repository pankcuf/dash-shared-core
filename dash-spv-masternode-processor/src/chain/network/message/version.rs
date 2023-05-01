use std::io::Read;
use byte::ctx::Str;
use byte::{BytesExt, TryRead};
use crate::chain::common::ChainType;
use crate::consensus::encode::{Decodable, Error, ReadExt, VarInt};
use crate::crypto::UInt128;
use crate::manager::peer_manager::{SERVICES_NODE_BLOOM, SERVICES_NODE_NETWORK};

#[derive(Clone, Debug, Default)]
pub struct Version {
    /// The P2P network protocol version
    pub version: u32,
    /// A bitmask describing the services supported by this node
    pub services: u64,
    /// The time at which the `version` message was sent
    pub timestamp: u64,
    /// A string describing the peer's software
    pub useragent: String,
    /// The height of the maximum-work blockchain that the peer is aware of
    pub last_block_height: u32,

    pub nonce: u64,

    pub addr_recv_services: u64,
    pub addr_recv_address: UInt128,
    pub addr_recv_port: u16,

    pub addr_trans_services: u64,
    pub addr_trans_address: UInt128,
    pub addr_trans_port: u16,
}

impl Version {
    pub fn doesnt_support_spv_filtering(&self) -> bool {
        self.version >= 70206 && self.services & SERVICES_NODE_BLOOM == 0
    }

    // drop peers that don't carry full blocks, or aren't synced yet
    // TODO: XXXX does this work with 0.11 pruned nodes?
    pub fn doesnt_support_full_blocks(&self) -> bool {
        self.services & SERVICES_NODE_NETWORK == 0
    }

    pub fn not_synced_yet(&self, height: u32) -> bool {
        self.last_block_height + 10 < height
    }
}

impl Decodable for Version {
    fn consensus_decode<D: Read>(mut d: D) -> Result<Self, Error> {
        // let bytes = d.take(MAX_MSG_LENGTH as u64).by_ref();
        let version = d.read_u32()?;
        let services = d.read_u64()?;
        let timestamp = d.read_u64()?;
        let addr_recv_services = d.read_u64()?;
        let addr_recv_address = UInt128::consensus_decode(&mut d)?;
        let addr_recv_port = d.read_u16()?.swap_bytes();
        let addr_trans_services = d.read_u64()?;
        let addr_trans_address = UInt128::consensus_decode(&mut d)?;
        let addr_trans_port = d.read_u16()?.swap_bytes();
        let nonce = d.read_u64()?;
        let useragent = Decodable::consensus_decode(&mut d)?;
        let last_block_height = d.read_u32()?;
        Ok(Version {
            version,
            services,
            timestamp,
            useragent,
            last_block_height,
            nonce,
            addr_recv_services,
            addr_recv_address,
            addr_recv_port,
            addr_trans_services,
            addr_trans_address,
            addr_trans_port
        })
    }
}


impl<'a> TryRead<'a, ChainType> for Version {
    fn try_read(bytes: &'a [u8], chain_type: ChainType) -> byte::Result<(Self, usize)> {
        if bytes.len() < 85 {
            println!("malformed version message, length is {}, should be > 84", bytes.len());
            return Err(byte::Error::Incomplete);
        }
        let offset = &mut 0;
        let version = bytes.read_with::<u32>(offset, byte::LE)?;
        if version < chain_type.min_protocol_version() {
            return Err(byte::Error::BadInput { err: "protocol version not supported" });
        }
        let services = bytes.read_with::<u64>(offset, byte::LE)?;
        let timestamp = bytes.read_with::<u64>(offset, byte::LE)?;

        let addr_recv_services = bytes.read_with::<u64>(offset, byte::LE)?;
        let addr_recv_address = bytes.read_with::<UInt128>(offset, byte::LE)?;
        let addr_recv_port = bytes.read_with::<u16>(offset, byte::BE)?;
        let addr_trans_services = bytes.read_with::<u64>(offset, byte::LE)?;
        let addr_trans_address = bytes.read_with::<UInt128>(offset, byte::LE)?;
        let addr_trans_port = bytes.read_with::<u16>(offset, byte::BE)?;

        let nonce = bytes.read_with::<u64>(offset, byte::LE)?;

        let useragent_len = bytes.read_with::<VarInt>(offset, byte::LE)?.0 as usize;
        let useragent = bytes.read_with::<&str>(offset, Str::Len(useragent_len))?.to_string();
        if bytes.len() < 80 + *offset + std::mem::size_of::<u32>() {
            println!("malformed version message, length is {}, should be {}", bytes.len(), 80 + useragent_len + 4);
            return Err(byte::Error::Incomplete);
        }
        let last_block_height = bytes.read_with::<u32>(offset, byte::LE)?;
        // let relay = bytes.read_with::<bool>(offset, ())?;
        Ok((Self {
            version,
            services,
            timestamp,
            useragent,
            last_block_height,
            addr_recv_services,
            addr_recv_address,
            addr_recv_port,
            addr_trans_services,
            addr_trans_address,
            addr_trans_port,
            nonce,
        }, *offset))
    }
}

