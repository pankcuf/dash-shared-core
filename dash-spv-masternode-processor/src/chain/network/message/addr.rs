use std::net::{IpAddr, SocketAddr, SocketAddrV4};
use std::time::SystemTime;
use byte::{BytesExt, TryRead};
use crate::consensus::encode::VarInt;
use crate::crypto::UInt128;
use crate::manager::peer_manager::SERVICES_NODE_NETWORK;
use crate::util::TimeUtil;

#[derive(Clone, Debug)]
pub struct AddrInfo {
    pub socket_addr: SocketAddr,
    pub timestamp: u64,
    pub services: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Addr {
    pub addresses: Vec<AddrInfo>,
}

impl<'a> TryRead<'a, u64> for AddrInfo {
    fn try_read(bytes: &'a [u8], now: u64) -> byte::Result<(Self, usize)> {
        let mut offset = 0usize;
        let mut timestamp = bytes.read_with::<u32>(&mut offset, byte::LE).unwrap() as u64;
        let services = bytes.read_with::<u64>(&mut offset, byte::LE).unwrap();
        let address = bytes.read_with::<UInt128>(&mut offset, byte::LE).unwrap();
        let port = bytes.read_with::<u16>(&mut offset, byte::BE).unwrap();
        // if (address.u64[0] != 0 || address.u32[2] != CFSwapInt32HostToBig(0xffff)) continue; // ignore IPv6 for now
        if services & SERVICES_NODE_NETWORK != 0 && IpAddr::from(address.0).is_ipv4() {
            // if address time is more than 10 min in the future or older than reference date, set to 5 days old
            if timestamp > now + 600 /*|| timestamp < 0*/ {
                timestamp = now - 5 * 24 * 60 * 60;
            }
            Ok((Self { socket_addr: SocketAddr::V4(SocketAddrV4::new(address.to_ipv4_addr(), port)), timestamp, services }, offset))
        } else {
            // skip peers that don't carry full blocks
            // ignore IPv6 for now
            Err(byte::Error::BadInput { err: "It's ipv6" })
        }
    }
}

// impl Decodable for Addr {
//     fn consensus_decode<D: Read>(d: D) -> Result<Self, Error> {
//         d.read_u8()?
//     }
// }


impl<'a> TryRead<'a, bool> for Addr {
    fn try_read(bytes: &'a [u8], sent_getaddr: bool) -> byte::Result<(Self, usize)> {
        // let socket_addr = peer_info.0;
        if bytes.len() > 0 && bytes.read_with::<u8>(&mut 0, byte::LE).unwrap() == 0 {
            // println!("{} got addr with 0 addresses", socket_addr);
            return Ok((Self::default(), 0));
        } else if bytes.len() < 5 {
            println!("malformed addr message, length {} is too short", bytes.len());
            return Err(byte::Error::Incomplete);
        } else if !sent_getaddr {
            // simple anti-tarpitting tactic, don't accept unsolicited addresses
            return Ok((Self::default(), 0));
        }
        let offset = &mut 0;
        let count = bytes.read_with::<VarInt>(offset, byte::LE).unwrap();
        let size = count.len() + count.0 as usize * 30;
        if count.0 > 1000 {
            // println!("{} dropping addr message, {} is too many addresses (max 1000)", socket_addr, count);
            Ok((Self::default(), 0))
        } else if bytes.len() < size {
            println!("malformed addr message, length is {}, should be {} for {} addresses", bytes.len(), size, count.0);
            Err(byte::Error::Incomplete)
        } else {
            // println!("{} got addr with {} addresses", socket_addr, count.0);
            let now = SystemTime::seconds_since_1970();
            let addresses = (count.len()..size)
                .step_by(30)
                .filter_map(|mut off| bytes.read_with::<AddrInfo>(&mut off, now).ok())
                .collect();
            Ok((Self { addresses }, *offset))
        }
    }
}

