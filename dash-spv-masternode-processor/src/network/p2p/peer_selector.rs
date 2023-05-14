use std::collections::HashSet;
use std::net::{Shutdown, SocketAddr, ToSocketAddrs};
use std::sync::Mutex;
use crate::chain::common::ChainType;
use crate::manager::peer_manager;
use crate::network::p2p::{Peer, PeerId, PeerMap, VersionCarrier};

pub(crate) const BAN :u32 = 100;
pub(crate) const PEER_MAX_CONNECTIONS: usize = 1;

pub struct PeerSelector {
    pub peers: PeerMap,
    pub max_connect_count: usize,
    pub chain_type: ChainType,
}

impl PeerSelector {
    pub fn new(chain_type: ChainType) -> Self {
        Self { peers: PeerMap::new(), chain_type, max_connect_count: PEER_MAX_CONNECTIONS }
    }

    pub fn connected_peers(&self) -> Vec<SocketAddr> {
        self.peers.values()
            .filter_map(|peer| peer.lock().unwrap().stream.peer_addr().ok())
            .collect()
    }

    pub fn shutdown(&mut self, pid: PeerId) {
        // let mut peers = self.peers.write().unwrap();
        debug!("shutdown: {:?}", pid);
        if let Some(peer) = self.peers.remove(&pid) {
            peer.lock().unwrap().stream.shutdown(Shutdown::Both).unwrap_or(());
        }
    }

    pub fn peer_by_id(&self, pid: PeerId) -> Option<&Mutex<Peer>> {
        self.peers.get(&pid)
    }


    pub fn has_peer_with_address(&self, address: SocketAddr) -> bool {
        self.peers.values().any(|peer| peer.lock().unwrap().stream.peer_addr().map_or(false, |addr| address.ip() == addr.ip()))
    }

    pub fn add(&mut self, peer: Peer) -> Result<(), peer_manager::Error> {
        let is_outgoing = peer.outgoing;
        let pid = peer.pid;
        self.peers.insert(pid, Mutex::new(peer));
        let stored_peer = self.peers.get(&pid).unwrap();
        if is_outgoing {
            stored_peer.lock().unwrap().register_write()
        } else {
            stored_peer.lock().unwrap().register_read()
        }
    }

    pub fn ban(&self, pid: PeerId, increment: u32) -> bool {
        let mut disconnected = false;
        if let Some(peer) = self.peer_by_id(pid) {
            let mut locked_peer = peer.lock().unwrap();
            locked_peer.ban += increment;
            if locked_peer.ban >= BAN {
                disconnected = true;
            }
        }
        disconnected
    }

    pub fn peer_version(&self, pid: PeerId) -> Option<VersionCarrier> {
        if let Some(peer) = self.peer_by_id(pid) {
            let locked_peer = peer.lock().unwrap();
            return locked_peer.version.clone();
        }
        None
    }

    // pub fn peers(&self) -> Vec<Mutex<Peer>> {
    //     self.peers.values()
    // }
    //
    pub fn peer_ids(&self) -> Vec<PeerId> {
        self.peers.keys().cloned().collect::<Vec<_>>()
    }

    pub fn len(&self) -> usize {
        self.peers.len()
    }

    pub fn select_peers(&self) -> Vec<SocketAddr> {
        match self.chain_type {
            ChainType::DevNet(_) => {
                self.chain_type.load_fixed_peer_addresses()
                    .iter()
                    .map(|addr| SocketAddr::new(*addr, self.chain_type.standard_port()))
                    .collect()
            },
            ChainType::TestNet => {
                // ["85.209.243.60:19999", "54.214.59.174:19999", "174.34.233.110:19999", "54.68.48.149:19999", "35.90.217.208:19999"]
                ["85.209.243.86:19999"]
                    .iter().map(|address| address.parse().unwrap()).collect()
            }
            _ => {
                let dns_seeds = self.chain_type.dns_seeds();
                let mut peers = vec![Vec::<SocketAddr>::new(); dns_seeds.len()];

                for (index, dns_seed) in dns_seeds.iter().enumerate() {
                    println!("DNS lookup {}", dns_seed);
                    if let Ok(mut iter) = (dns_seed.to_string(), self.chain_type.standard_port()).to_socket_addrs() {
                        while let Some(socket_addr) = iter.next() {
                            // skipping ipv6 for now
                            if socket_addr.is_ipv4() {
                                // add between 3 and 7 days
                                peers[index].push(socket_addr);
                            }
                        }
                    }
                }
                peers.iter().flat_map(|p| p.iter().cloned()).collect()
            }
        }
    }

    pub fn select_more_peers(&self) -> HashSet<SocketAddr> {
        let mut earlier = HashSet::new();
        if self.len() < self.max_connect_count {
            let mut eligible = self.select_peers()
                .iter()
                .cloned()
                .filter(|a| !earlier.contains(a))
                .collect::<Vec<_>>();
            if !eligible.is_empty() {
                earlier.extend(eligible.drain(..self.max_connect_count));
            }
        }
        earlier
    }
}
