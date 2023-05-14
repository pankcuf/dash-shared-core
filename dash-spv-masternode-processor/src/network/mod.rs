use std::io::{Error, Write};
use std::net::SocketAddr;
use crate::consensus::Encodable;
use crate::crypto::UInt128;

pub mod constants;
pub mod network_context;
pub mod pipeline;
pub mod p2p;
pub mod reachability_manager;


impl Encodable for SocketAddr {
    fn consensus_encode<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
        let ip: UInt128 = self.ip().into();
        ip.enc(&mut writer);
        self.port().swap_bytes().enc(&mut writer);
        Ok(18)
    }
}
