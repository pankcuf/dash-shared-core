use rand::thread_rng;
use secp256k1::rand;
use secp256k1::rand::Rng;
use std::cmp::Ordering;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::channel;
use std::{fmt, io, thread};
use std::time::SystemTime;
use byte::ctx::Endian;
use byte::{BytesExt, TryRead, TryWrite};
use crate::consensus::Encodable;
use crate::chain::SyncType;
use crate::chain::chain::Chain;
use crate::chain::common::ChainType;
use crate::chain::ext::Settings;
use crate::chain::network::{BloomFilter, InvType, PeerStatus, PeerType};
use crate::chain::network::peer::{DAYS_3_TIME_INTERVAL, HOURS_3_TIME_INTERVAL, Peer, WEEK_TIME_INTERVAL};
use crate::crypto::{UInt128, UInt256};
use crate::crypto::byte_util::AsBytes;
use crate::manager::peer_manager_desired_state::PeerManagerDesiredState;
use crate::models::MasternodeList;
use crate::storage::{Keychain, UserDefaults};
use crate::{consensus, util};
use crate::util::{Shared, TimeUtil};

pub const PEER_MAX_CONNECTIONS: usize = 5;
pub const SETTINGS_FIXED_PEER_KEY: &str = "SETTINGS_FIXED_PEER";
pub const LAST_SYNCED_GOVERANCE_OBJECTS: &str = "LAST_SYNCED_GOVERANCE_OBJECTS";
pub const LAST_SYNCED_MASTERNODE_LIST: &str = "LAST_SYNCED_MASTERNODE_LIST";


/// services value indicating a node carries full blocks, not just headers
pub const SERVICES_NODE_NETWORK: u64 = 0x01;
/// BIP111: https://github.com/bitcoin/bips/blob/master/bip-0111.mediawiki
pub const SERVICES_NODE_BLOOM: u64 = 0x04;
/// notify user of network problems after this many connect failures in a row
pub const MAX_CONNECT_FAILURES: u32 = 20;

pub const PROTOCOL_TIMEOUT: u64 = 20;
const SERVICES: u64 = SERVICES_NODE_NETWORK | SERVICES_NODE_BLOOM;

#[derive(Debug, Default, PartialEq)]
pub struct PeerInfo {
    pub address: UInt128,
    pub port: u16,
    pub dapi_grpc_port: u32,
    pub dapi_jrpc_port: u32,
}

// impl From<Vec<u8>> for Vec<PeerInfo> {
//     fn from(value: Vec<u8>) -> Self {
//         todo!()
//     }
// }


impl<'a> TryRead<'a, Endian> for PeerInfo {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let address = bytes.read_with::<UInt128>(offset, endian)?;
        let port = bytes.read_with::<u16>(offset, endian)?;
        let dapi_grpc_port = bytes.read_with::<u32>(offset, endian)?;
        let dapi_jrpc_port = bytes.read_with::<u32>(offset, endian)?;
        Ok((Self { address, port, dapi_grpc_port, dapi_jrpc_port }, *offset))
    }
}

impl TryWrite<Endian> for PeerInfo {
    fn try_write(self, bytes: &mut [u8], endian: Endian) -> Result<usize, byte::Error> {
        let offset = &mut 0;
        bytes.write_with(offset, self.address, endian)?;
        bytes.write_with(offset, self.port, endian)?;
        bytes.write_with(offset, self.dapi_grpc_port, endian)?;
        bytes.write_with(offset, self.dapi_jrpc_port, endian)?;
        Ok(*offset)
    }
}

impl Encodable for PeerInfo {
    #[inline]
    fn consensus_encode<S: std::io::Write>(&self, mut s: S) -> Result<usize, std::io::Error> {
        self.address.enc(&mut s);
        self.port.enc(&mut s);
        self.dapi_grpc_port.enc(&mut s);
        self.dapi_jrpc_port.enc(&mut s);
        Ok(std::mem::size_of::<PeerInfo>())
    }
}



#[derive(Debug)]
pub enum Error {
    ServiceNetworkError(Peer),
    BloomFilteringNotSupported(Peer),
    NoPeersFound,
    SyncTimeout,
    Default(String),
    DefaultWithCode(String, u32),
    /// the block's work target is not correct
    SpvBadTarget,
    /// bad proof of work
    SpvBadProofOfWork,
    /// unconnected header chain detected
    UnconnectedHeader,
    /// no chain tip found
    NoTip,
    /// no peers to connect to
    NoPeers,
    /// unknown UTXO referred
    UnknownUTXO,
    /// Merkle root of block does not match the header
    BadMerkleRoot,
    /// downstream error
    Downstream(String),
    /// Network IO error
    IO(io::Error),
    /// Bitcoin util error
    Util(util::Error),
    /// Bitcoin serialize error
    Serialize(consensus::encode::Error),
    // /// Hammersbald error
    // Hammersbald(hammersbald::Error),
    /// Handshake failure
    Handshake,
    HandshakeTimeout,
    /// lost connection
    Lost(String)

}

impl Error {
    pub fn is_app_level(&self) -> bool {
        true
    }
    pub fn code(&self) -> i32 {
        match self {
            Error::NoPeersFound => 1,
            _ => 500
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IO(err)
    }
}

impl From<byte::Error> for Error {
    fn from(err: byte::Error) -> Error {
        Error::Default(format!("{:?}", err))
    }
}

impl From<consensus::encode::Error> for Error {
    fn from(value: consensus::encode::Error) -> Self {
        Error::Serialize(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ServiceNetworkError(peer) => write!(f, "Node at host {} does not service network", peer.host()),
            Error::BloomFilteringNotSupported(peer) => write!(f, "Node at host {} does not support bloom filtering", peer.host()),
            Error::NoPeersFound => write!(f, "No peers found"),
            Error::SyncTimeout => write!(f, "An error message for notifying that chain sync has timed out"),
            Error::Default(message) => write!(f, "{}", message.clone()),
            Error::DefaultWithCode(message, _) => write!(f, "{}", message.clone()),
            Error::SpvBadTarget => write!(f, "bad proof of work target"),
            Error::SpvBadProofOfWork => write!(f, "bad proof of work"),
            Error::UnconnectedHeader => write!(f, "unconnected header"),
            Error::NoTip => write!(f, "no chain tip found"),
            Error::UnknownUTXO => write!(f, "unknown utxo"),
            Error::NoPeers => write!(f, "no peers"),
            Error::BadMerkleRoot =>
                write!(f, "merkle root of header does not match transaction list"),
            Error::Handshake => write!(f, "handshake"),
            Error::HandshakeTimeout => write!(f, "handshake timeout"),
            Error::Lost(ref s) => write!(f, "lost connection: {}", s),
            Error::Downstream(ref s) => write!(f, "downstream error: {}", s),
            // The underlying errors already impl `Display`, so we defer to their implementations.
            Error::IO(ref err) => write!(f, "IO error: {}", err),
            Error::Util(ref err) => write!(f, "Util error: {}", err),
            // Error::Hammersbald(ref err) => write!(f, "Hammersbald error: {}", err),
            Error::Serialize(ref err) => write!(f, "Serialize error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::SpvBadTarget => None,
            Error::SpvBadProofOfWork => None,
            Error::UnconnectedHeader => None,
            Error::NoTip => None,
            Error::NoPeers => None,
            Error::UnknownUTXO => None,
            Error::Downstream(_) => None,
            Error::BadMerkleRoot => None,
            Error::IO(ref err) => Some(err),
            Error::Util(ref err) => Some(err),
            // Error::Hammersbald(ref err) => Some(err),
            Error::Serialize(ref err) => Some(err),
            Error::Handshake => None,
            Error::Lost(_) => None,
            Error::ServiceNetworkError(_) => None,
            Error::BloomFilteringNotSupported(_) => None,
            Error::NoPeersFound => None,
            Error::SyncTimeout => None,
            Error::Default(_) => None,
            Error::DefaultWithCode(_, _) => None,
            Error::HandshakeTimeout => None
        }
    }
}



#[derive(Clone, Copy, Debug, PartialEq)]
enum Command {
    Connect,
    Disconnect,
    NoPeers
}
unsafe impl Send for Command {}
unsafe impl Sync for Command {}

#[derive(Clone, Debug, Default)]
struct Inner {
    pub connected: bool,
    pub connected_peers: Vec<Peer>,
    pub download_peer: Option<Peer>,

    pub peers: Vec<Peer>,
    pub mutable_connected_peers: Vec<Peer>,
    pub mutable_misbehaving_peers: Vec<Peer>,
    pub fixed_peer: Option<Peer>,

    pub connect_failures: u32,
    pub misbehaving_count: usize,
    pub max_connect_count: usize,
    pub desired_state: PeerManagerDesiredState,
    pub chain_type: ChainType,
    pub masternode_list: Option<Shared<MasternodeList>>,
    pub masternode_list_connectivity_nonce: u64,
}

unsafe impl Send for Inner {}
unsafe impl Sync for Inner {}


impl Inner {
    /// number of connected peers
    pub fn connected_peer_count(&self) -> usize {
        self.connected_peers.iter().filter(|peer| peer.status() == PeerStatus::Connected).count()
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn download_peer_name(&self) -> String {
        self.download_peer
            .as_ref()
            .map_or_else(|| String::new(), |peer| format!("{:?}", peer))

    }

    fn dns_seeds(&self) -> Vec<&str> {
        self.chain_type.dns_seeds()
    }

    pub fn is_download_peer(&self, peer: &Peer) -> bool {
        if let Some(dp) = &self.download_peer {
            dp == peer
        } else {
            false
        }
    }

    pub fn peer_misbehaving(&mut self, peer: &mut Peer) {
        self.peers.retain(|p| p != peer);
        self.mutable_misbehaving_peers.push(peer.clone());
        let increased_count = self.misbehaving_count + 1;
        if increased_count >= self.chain_type.peer_misbehaving_threshold() {
            // clear out stored peers so we get a fresh list from DNS for next connect
            self.misbehaving_count = 0;
            self.mutable_misbehaving_peers.clear();
            /*
            self.context.perform_block_and_wait(|context| {
                PeerEntity::delete_all_peers_for_chain(self.chain.r#type(), context)
                    .expect("Can't delete peer entities");
            });*/
            self.clear_peers();
        }
    }

    pub fn remove_misbehaving_peers(&mut self) {
        self.peers.retain(|peer| !self.mutable_misbehaving_peers.contains(peer));
    }

    pub fn remove_peer(&mut self, peer: &Peer) {
        self.peers.retain(|p| p != peer);
        self.connect_failures += 1;
    }


    pub fn add_peers(&mut self, peers: Vec<Peer>) {
        if self.masternode_list.is_some() {
            return;
        }
        self.peers.extend(peers);
        self.remove_misbehaving_peers();
        self.sort_peers();
        // limit total to 2500 peers
        if self.peers.len() > 2500 {
            self.peers.drain(2500..);
        }
        let now = SystemTime::seconds_since_1970();
        // remove peers more than 3 hours old, or until there are only 1000 left
        while let Some(peer) = self.peers.last() {
            if self.peers.len() > 1000 && peer.timestamp() + 3 * 60 * 60 < now {
                self.peers.pop();
            } else {
                break;
            }
        }

    }

    pub fn clear_all_peers(&mut self) {
        // clear out stored peers so we get a fresh list from DNS on next connect attempt
        self.mutable_misbehaving_peers.clear();
        /*self.context.perform_block_and_wait(|context| {
            PeerEntity::delete_all_peers_for_chain(self.chain.r#type(), context)
                .expect("Can't delete stored peers");
        });*/
        self.clear_peers();

    }

    pub fn clear_peers(&mut self) {
        self.peers.clear();
    }

    pub fn sort_peers(&mut self) {
        let three_hours_ago = SystemTime::seconds_since_1970() - HOURS_3_TIME_INTERVAL;
        let (syncs_masternode_list, syncs_governance_objects) = (
            self.chain_type.sync_type().bits() & SyncType::MasternodeList.bits() != 0,
            self.chain_type.sync_type().bits() & SyncType::Governance.bits() != 0
        );
        self.peers.sort_by(|p1, p2| {
            // the following is to make sure we get
            if syncs_masternode_list {
                match (p1.last_requested_masternode_list, p2.last_requested_masternode_list) {
                    (Some(p1_list), Some(p2_list)) if p1_list < three_hours_ago && p2_list > three_hours_ago => { return Ordering::Less; },
                    (None, Some(p2_list)) if p2_list > three_hours_ago => { return Ordering::Less; },
                    (Some(p1_list), Some(p2_list)) if p1_list > three_hours_ago && p2_list < three_hours_ago => { return Ordering::Greater; },
                    (Some(p1_list), None) => { return Ordering::Greater; },
                    _ => {}
                }

                // if (p1.last_requested_masternode_list.is_none() || p1.last_requested_masternode_list.unwrap() < three_hours_ago) &&
                //     p2.last_requested_masternode_list.unwrap() > three_hours_ago {
                //     return Ordering::Less; // NSOrderedDescending
                // }
                // if p1.last_requested_masternode_list.unwrap() > three_hours_ago &&
                //     (p2.last_requested_masternode_list.is_none() || p2.last_requested_masternode_list.unwrap() < three_hours_ago) {
                //     return Ordering::Greater; // NSOrderedAscending
                // }
            }
            if syncs_governance_objects {
                if (p1.last_requested_governance_sync.is_none() || p1.last_requested_governance_sync.unwrap() < three_hours_ago) &&
                    p2.last_requested_governance_sync.unwrap() > three_hours_ago {
                    return Ordering::Less; // NSOrderedDescending
                }
                if p1.last_requested_governance_sync.unwrap() > three_hours_ago &&
                    (p2.last_requested_governance_sync.is_none() || p2.last_requested_governance_sync.unwrap() < three_hours_ago) {
                    return Ordering::Greater; // NSOrderedAscending
                }
                if p1.priority > p2.priority {
                    return Ordering::Greater; // NSOrderedAscending
                } else if p1.priority < p2.priority {
                    return Ordering::Less; // NSOrderedDescending
                }
                if p1.timestamp() > p2.timestamp() {
                    return Ordering::Greater; // NSOrderedAscending
                } else if p1.timestamp() < p2.timestamp() {
                    return Ordering::Less; // NSOrderedDescending
                }
            }
            Ordering::Equal
        });
    }

    pub fn peer_for_location(&self, socket_addr: SocketAddr) -> Option<&Peer> {
        self.peers.iter().find(|peer| peer.socket_addr == socket_addr)
    }

    pub fn registered_devnet_peers(&self, chain: Shared<Chain>) -> Vec<Peer> {
        if let Ok(array) = Keychain::get_array::<PeerInfo>(self.chain_type.registered_peers_key()) {
            let now = SystemTime::seconds_since_1970();
            let weak_ago = now - WEEK_TIME_INTERVAL;
            let timestamp = thread_rng().gen_range(weak_ago..=now);
            array.iter().map(|info| Peer::init_with_address(info.address.to_ip_addr(), info.port, self.chain_type, chain.clone(), timestamp, SERVICES)).collect()
        } else {
            vec![]
        }
    }

    pub fn masternode_list_peers(&self, chain: Shared<Chain>) -> Vec<Peer> {
        if let Some(list) = self.masternode_list.as_ref() {
            list.with(|masternode_list| Self::peers_with_connectivity_nonce(masternode_list, 8, self.masternode_list_connectivity_nonce, self.chain_type, chain))
        } else {
            vec![]
        }
    }

    pub fn registered_devnet_peer_services(&self, chain: Shared<Chain>) -> Vec<String> {
        self.registered_devnet_peers(chain)
            .iter()
            // .filter_map(|peer| (!peer.address.is_zero()).then_some(format!("{}:{}", peer.host(), peer.port)))
            .map(|peer| format!("{:?}", peer))
            .collect()
    }

    pub fn peers_with_connectivity_nonce(masternode_list: &MasternodeList, peer_count: usize, connectivity_nonce: u64, chain_type: ChainType, chain: Shared<Chain>) -> Vec<Peer> {
        let entries = &masternode_list.masternodes;
        let mut sorted_hashes = entries.keys().collect::<Vec<_>>();
        sorted_hashes.sort_by(|&h1, &h2| {
            let mut h1v: Vec<u8> = h1.as_bytes().to_vec();
            connectivity_nonce.enc(&mut h1v);
            let h1ru = UInt256(*blake3::hash(&h1v).as_bytes());
            let mut h2v: Vec<u8> = h2.as_bytes().to_vec();
            connectivity_nonce.enc(&mut h2v);
            let h2ru = UInt256(*blake3::hash(&h2v).as_bytes());
            h1ru.cmp(&h2ru)
        });
        sorted_hashes.into_iter()
            .take(peer_count)
            .filter_map(|hash| entries.get(hash))
            .filter(|masternode| masternode.is_valid)
            .map(|masternode| Peer::init_with_masternode(masternode, chain_type, chain.clone()))
            .collect()
    }

    pub fn peers(&mut self, chain: Shared<Chain>) -> Vec<Peer> {
        if let Some(fixed) = self.fixed_peer.as_ref() {
            return vec![fixed.clone()];
        } else if self.peers.len() >= self.max_connect_count {
            return self.peers.clone();
        }
        self.peers = vec![];
        // todo: load from local DB
        /*self.context.perform_block_and_wait(|context| {
            match PeerEntity::get_all_peers_for_chain(self.chain.r#type(), context) {
                Ok(peers) => {
                    peers.iter().for_each(|peer| {
                        if peer.misbehaving == 0 {
                            self.peers.push(peer.peer(self.chain));
                        } else {
                            self.mutable_misbehaving_peers.insert(peer.peer(self.chain));
                        }
                    });
                },
                Err(err) => println!("Error retrieving peers")
            }
        });*/
        match self.chain_type {
            ChainType::DevNet(..) => {
                self.peers.extend(self.registered_devnet_peers(chain.clone()));
                self.peers.extend(self.masternode_list_peers(chain.clone()));
                self.sort_peers();
            },
            _ => {
                if self.masternode_list.is_some() {
                    self.peers.extend(self.masternode_list_peers(chain.clone()));
                } else {
                    self.sort_peers();
                    // DNS peer discovery
                    let now = SystemTime::seconds_since_1970();
                    let mut peers = Vec::<Vec<Peer>>::new();
                    let dns_seeds = self.dns_seeds();
                    if self.peers.len() < PEER_MAX_CONNECTIONS || self.peers[PEER_MAX_CONNECTIONS - 1].timestamp() + DAYS_3_TIME_INTERVAL < now {
                        peers.resize_with(dns_seeds.len(), || vec![]);
                    }
                    let port = self.chain_type.standard_port();
                    if !peers.is_empty() && !dns_seeds.is_empty() {
                        for (index, dns_seed) in dns_seeds.iter().enumerate() {
                            println!("DNS lookup {}", dns_seed);
                            if let Ok(mut iter) = dns_seed.to_socket_addrs() {
                                while let Some(socket_addr) = iter.next() {
                                    // skipping ipv6 for now
                                    if socket_addr.is_ipv4() {
                                        // add between 3 and 7 days
                                        peers[index].push(Peer::init_with_socket_addr(
                                            socket_addr,
                                            self.chain_type,
                                            chain.clone(),
                                            now - thread_rng().gen_range(DAYS_3_TIME_INTERVAL..=WEEK_TIME_INTERVAL),
                                            SERVICES));
                                    }
                                }
                            }
                        }
                        self.peers.extend(peers.iter().flat_map(|p| p.iter().cloned()));
                        // if DNS peer discovery fails, fall back on a hard coded list of peers (list taken from satoshi client)
                        if self.peers.len() < PEER_MAX_CONNECTIONS {
                            println!("DNS peer discovery failed: will use fixed peers");
                            self.peers.extend(self.chain_type
                                .load_fixed_peer_addresses()
                                .into_iter()
                                .map(|address| Peer::init_with_address(
                                    address,
                                    port,
                                    self.chain_type,
                                    chain.clone(),
                                    now - thread_rng().gen_range(0..WEEK_TIME_INTERVAL),
                                    SERVICES)));
                        }
                    }
                }
             }
        }
        self.sort_peers();
        self.peers.clone()
    }

    pub fn trusted_peer_host(&self) -> Option<String> {
        UserDefaults::string_for_key(self.chain_type.settings_fixed_peer_key())
    }

    pub fn setup_fixed_peer(&mut self, chain: Shared<Chain>) {
        self.fixed_peer = self.trusted_peer_host().and_then(|trusted| Peer::peer_with_host(&trusted, chain));
        self.max_connect_count = self.fixed_peer.as_ref().map_or(1, |_| PEER_MAX_CONNECTIONS);
    }

    pub fn set_trusted_peer_host(&self, host: Option<String>) {
        if let Some(host) = host {
            UserDefaults::set_string(self.chain_type.settings_fixed_peer_key(), host);
        } else {
            UserDefaults::delete(self.chain_type.settings_fixed_peer_key());
        }
    }

}

// #[derive(Debug, Default)]
#[derive(Clone, Debug, Default)]
pub struct PeerManager {
    inner: Arc<RwLock<Inner>>,
    // masternode_list: Option<Shared<MasternodeList>>,
    // masternode_list_connectivity_nonce: u64,

    pub chain: Shared<Chain>,

    // @property (nonatomic, readonly) NSUInteger connectFailures, misbehavingCount, maxConnectCount;

    // sender: Option<Arc<Sender<Command>>>
    // sender: Option<Shared<Sender<Command>>>,
    // sender: Option<Arc<RwLock<Sender<Command>>>>,
    // handle: Option<thread::JoinHandle<()>>,

}

impl PeerManager {
    pub fn new(chain_type: ChainType) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner {
                chain_type,
                // todo: init peer_context in networking queue
                // context: chain.peer_context(),
                max_connect_count: PEER_MAX_CONNECTIONS,
                ..Default::default()
            })),
            ..Default::default()
        }
    }
}


impl PeerManager {
    pub(crate) fn peer_misbehaving(&mut self, peer: &mut Peer, message: String) {
        // todo: multithreading
        peer.misbehaving += 1;
        if let Ok(mut writer) = self.inner.try_write() {
            writer.peer_misbehaving(peer);
        }
        peer.disconnect_with_error(Some(Error::DefaultWithCode(message, 500)));
        self.connect(self.chain.with(|chain| chain.earliest_wallet_creation_time()));
    }
}

impl PeerManager {

    // Managers
    // pub fn masternode_manager(&self) -> &MasternodeManager {
    //     self.chain.masternode_manager()
    // }
    //
    // pub fn transaction_manager(&self) -> &TransactionManager {
    //     self.chain.transaction_manager()
    // }
    //
    // pub fn governance_sync_manager(&self) -> &GovernanceSyncManager {
    //     self.chain.governance_sync_manager()
    // }
    //
    // pub fn spork_manager(&self) -> &spork::Manager {
    //     self.chain.spork_manager()
    // }


    /// Peers
    pub fn remove_trusted_peer_host(&mut self) {
        self.disconnect();
        self.inner.try_write().unwrap().set_trusted_peer_host(None);
    }

    pub fn clear_peers(&mut self) {
        self.disconnect();
        if let Ok(mut writer) = self.inner.try_write() {
            writer.clear_peers();
        }
    }

    pub fn peers(&mut self) -> Vec<Peer> {
        match self.inner.try_write() {
            Ok(mut writer) => writer.peers(self.chain.clone()),
            Err(err) => panic!("Can't lock inner")
        }
    }

    pub fn sort_peers(&mut self) {
        if let Ok(mut writer) = self.inner.try_write() {
            writer.sort_peers();
        }
    }

    pub fn save_peers(&self) {
        println!("[PeerManager] save peers");
        todo!()
        // let mut peers = HashSet::from_iter(self.peers.iter().cloned().chain(self.mutable_misbehaving_peers.iter().cloned()));
        // let addrs: Vec<_> = peers.iter().filter_map(|peer| {
        //     if IpAddr::from(peer.address.0).is_ipv4() {
        //         Some(peer.address.ip_address_to_i32())
        //     } else {
        //         None
        //     }
        // }).collect();

        /*self.context.perform_block(|context| {
            ChainEntity::get_chain(self.chain.r#type(), context)
                .and_then(|chain_entity| PeerEntity::delete_peers_except_list(self.chain.r#type(), addrs, context)
                    .and_then(|deleted| PeerEntity::get_peers_with_addresses_for_chain(&chain_entity, addrs, context)
                        .and_then(|peer_entities| {
                            peer_entities.iter().for_each(|peer_entity| {
                                let peer = peer_entity.peer(self.chain);
                                if let Some(p) = peers.get(&peer) {
                                    match peer_entity.update_with_peer(p, context) {
                                        Ok(updated) => {
                                            peers.remove(p);
                                        },
                                        Err(err) => println!("Sqlite Error  {:?}", err)
                                    }
                                } else {
                                    match PeerEntity::delete_by_id(peer_entity.id, context) {
                                        Ok(deleted) => {},
                                        Err(err) => println!("Sqlite Error  {:?}", err)
                                    }
                                }
                            });
                            PeerEntity::create_from_peers(peers, chain_entity.id, context)
                        }))).expect("Can't update peers");
        });*/
    }

    // pub fn peer_for_location(&self, ip_address: UInt128, port: u16) -> Option<&Peer> {
    // pub fn peer_for_location(&self, socket_addr: SocketAddr) -> Option<&Peer> {
    //     self.inner.try_read().map_or(None,|ref reader| reader.peer_for_location(socket_addr))
    // }

    pub fn status_for_location(&self, socket_addr: SocketAddr) -> PeerStatus {
        match self.inner.try_read() {
            Ok(ref reader) => match reader.peer_for_location(socket_addr) {
                Some(peer) if reader.mutable_misbehaving_peers.contains(peer) => PeerStatus::Banned,
                Some(peer) => peer.status.clone(),
                None => PeerStatus::Unknown,
            },
            _ => PeerStatus::Unknown
        }
    }

    pub fn type_for_location(&self, socket_addr: SocketAddr) -> PeerType {
        match self.inner.try_read() {
            Ok(ref reader) => match reader.peer_for_location(socket_addr) {
                Some(..) if self.chain.read(|chain| chain.masternode_manager.has_masternode_at_location(socket_addr)) => PeerType::MasterNode,
                Some(..) => PeerType::FullNode,
                None => PeerType::Unknown,
            },
            _ => PeerType::Unknown
        }
    }


    /// Peer Registration
    pub fn pause_blockchain_synchronization_on_peers(&mut self) {
        if let Ok(mut writer) = self.inner.try_write() {
            if let Some(peer) = writer.download_peer.as_mut() {
                peer.needs_filter_update = true;
            }
        }
    }

    pub fn resume_blockchain_synchronization_on_peers(&mut self) {
        // if let Ok(mut writer) = self.inner.try_write() {
        //     if let Some(peer) = writer.download_peer.as_mut() {
        //         peer.needs_filter_update = false;
        //         self.update_filter_on_peers();
        //     } else {
        //         self.connect();
        //     }
        // }
    }


    pub fn update_filter_on_peers(&mut self) {
        // if let Some(download_peer) = &mut self.inner.download_peer {
        //     if download_peer.needs_filter_update {
        //         return;
        //     }
        //     download_peer.needs_filter_update = true;
        //     println!("filter update needed, waiting for pong");
        //     download_peer.send_ping_message(Arc::new(|success| {
        //         // wait for pong so we include already sent tx
        //         if !success { return; }
        //         // we are on chainPeerManagerQueue
        //         /*self.chain.with(|chain| {
        //             // let tx_manager = chain.transaction_manager;
        //             chain.transaction_manager.clear_transactions_bloom_filter();
        //             if chain.last_sync_block_height() < chain.estimated_block_height() {
        //                 // if we're syncing, only update download peer
        //
        //                 download_peer.send_filterload_message(chain.transaction_manager.transactions_bloom_filter_for_peer_hash(download_peer.hash()).to_data());
        //                 download_peer.send_ping_message(Arc::new(|success| {
        //                     // wait for pong so filter is loaded
        //                     if !success { return; }
        //                     download_peer.needs_filter_update = false;
        //                     download_peer.rerequest_blocks_from(&chain.last_sync_block.unwrap().block_hash());
        //                     download_peer.send_ping_message(Arc::new(|success| {
        //                         // wait for pong so filter is loaded
        //                         if !success || download_peer.needs_filter_update { return; }
        //                         download_peer.send_getblocks_message_with_locators(chain.chain_sync_block_locator_array(), UInt256::MIN);
        //                     }));
        //                 }));
        //             } else {
        //                 self.connected_peers.iter_mut().for_each(|peer| {
        //                     if peer.status() == PeerStatus::Connected {
        //                         peer.send_filterload_message(chain.transaction_manager.transactions_bloom_filter_for_peer_hash(peer.hash()).to_data());
        //                         peer.send_ping_message(Arc::new(|success| {
        //                             // wait for pong so we know filter is loaded
        //                             if !success { return; }
        //                             peer.needs_filter_update = false;
        //                             peer.send_mempool_message(chain.transaction_manager.published_tx_hashes(), None);
        //                         }));
        //                     }
        //                 });
        //             }
        //         });*/
        //         // let tx_manager = self.transaction_manager();
        //         // println!("updating filter with newly created wallet addresses");
        //         // tx_manager.clear_transactions_bloom_filter();
        //         // if self.chain.last_sync_block_height < self.chain.estimated_block_height() {
        //         //     // if we're syncing, only update download peer
        //         //     download_peer.send_filter_load_message(tx_manager.transactions_bloom_filter_for_peer(download_peer).data);
        //         //     download_peer.send_ping_message(|success| {
        //         //         // wait for pong so filter is loaded
        //         //         if !success { return; }
        //         //         download_peer.needs_filter_update = false;
        //         //         download_peer.rerequest_blocks_from(&self.chain.last_sync_block.unwrap().block_hash());
        //         //         download_peer.send_ping_message(|success| {
        //         //             // wait for pong so filter is loaded
        //         //             if !success || download_peer.needs_filter_update { return; }
        //         //             download_peer.send_get_blocks_message_with_locators(self.chain.chain_sync_block_locator_array(), UInt256::MIN);
        //         //         });
        //         //     })
        //         // } else {
        //         //     self.connected_peers.iter_mut().for_each(|peer| {
        //         //         if peer.status() == PeerStatus::Connected {
        //         //             peer.send_filterload_message(self.transaction_manager().transactions_bloom_filter_for_peer(peer).data);
        //         //             peer.send_ping_message(|success| {
        //         //                 // wait for pong so we know filter is loaded
        //         //                 if !success { return; }
        //         //                 peer.needs_filter_update = false;
        //         //                 peer.send_mempool_message(self.transaction_manager().published_tx.keys(), || {});
        //         //             });
        //         //         }
        //         //     });
        //         // }
        //     }));
        // }
    }


    /// Peer Registration
    pub fn clear_registered_peers(&mut self) {
        self.clear_peers();
        Keychain::set_array::<PeerInfo>(vec![], self.inner.try_read().unwrap().chain_type.registered_peers_key(), false)
            .expect("Can't clear peers stored in keychain");
    }

    pub fn register_peer_at_location(&self, ip_address: UInt128, port: u16, dapi_jrpc_port: u32, dapi_grpc_port: u32) {
        let mut registered_peer_infos = Keychain::get_array::<PeerInfo>(self.inner.try_read().unwrap().chain_type.registered_peers_key()).unwrap_or(vec![]);
        let insert_info = PeerInfo { address: ip_address, port, dapi_jrpc_port, dapi_grpc_port };
        if !registered_peer_infos.contains(&insert_info) {
            registered_peer_infos.push(insert_info);
        }
        Keychain::set_array::<PeerInfo>(registered_peer_infos, self.inner.try_read().unwrap().chain_type.registered_peers_key(), false)
            .expect("Can't store peer infos in keychain");
    }



    /// Using Masternode List for connectivitity
    pub fn use_masternode_list_with_connectivity_nonce(&mut self, masternode_list: Shared<MasternodeList>, connectivity_nonce: u64) {
        if let Ok(mut writer) = self.inner.try_write() {
            writer.masternode_list = Some(masternode_list);
            writer.masternode_list_connectivity_nonce = connectivity_nonce;
        }
        // self.masternode_list = Some(masternode_list);
        // self.masternode_list_connectivity_nonce = connectivity_nonce;
        todo!()
        // let connected = self.connected;
        // masternode_list.with(|list| {
        //     let peers = self.peers_with_connectivity_nonce(list, 500, connectivity_nonce);
        //     if !peers.is_empty() {
        //         self.clear_peers();
        //         self.peers = peers;
        //         self.remove_misbehaving_peers();
        //     } else {
        //         self.peers = peers;
        //     }
        // });
        // self.sort_peers();
        // if self.peers.len() > 1 && self.peers.len() < 1000 {
        //     // peer relaying is complete when we receive <1000
        //     self.save_peers();
        // }
        // if connected {
        //     self.connect();
        // }

        /*DispatchContext::main_context().queue(||
            NotificationCenter::post(Notification::PeersDidChange(self.chain)));*/
    }

    /// Peer Registration
    pub fn connect(&mut self, earliest_wallet_creation_time: u64) {
        println!("PeerManager::connect");
        let (sender, receiver) = channel::<Command>();
        // self.sender = Some(shared_sender);
        let shared_chain = self.chain.clone();
        let inner = self.inner.clone();
        // DispatchContext::network_context().queue(|| {
        let handle = thread::spawn(move || {
            // if shared_chain.read(|chain| chain.cant_connect()) {
            //     // check to make sure the wallet has been created if only are a basic wallet with no dash features
            //     return;
            // }
            let mut inner = inner.try_write().unwrap();
            inner.desired_state = PeerManagerDesiredState::Connected;
            if inner.connect_failures >= MAX_CONNECT_FAILURES {
                // this attempt is a manual retry
                inner.connect_failures = 0;
            }
            // if shared_chain.with(|chain| chain.terminal_header_sync_progress() < 1.0) {
            //     shared_chain.with(|chain| chain.reset_terminal_sync_start_height());
            //     // todo: background ops
            // }
            // if shared_chain.with(|chain| chain.chain_sync_progress() < 1.0) {
            //     shared_chain.with(|chain| chain.reset_chain_sync_start_height());
            //     // todo: background ops
            // }

            // remove all disconnected peers
            inner.mutable_connected_peers.retain(|peer| peer.status() != PeerStatus::Disconnected);
            inner.setup_fixed_peer(shared_chain.clone());
            if inner.connected_peers.len() >= inner.max_connect_count {
                // already connected to maxConnectCount peers
                return;
            }
            let peers = inner.peers(shared_chain.clone());
            let mut peers = if peers.len() <= 100 {
                peers.clone()
            } else {
                peers[..100].to_vec()
            };
            if inner.connected_peers.len() < inner.max_connect_count {
                while !peers.is_empty() && inner.connected_peers.len() < inner.max_connect_count {
                    // pick a random peer biased towards peers with more recent timestamps
                    let random_index = ((thread_rng().gen_range(0..peers.len()) as f64).powi(2) / peers.len() as f64) as usize;
                    let peer = peers.iter_mut().nth(random_index).unwrap();
                    if !inner.connected_peers.contains(peer) {
                        // peer.chain_delegate = chain;
                        inner.mutable_connected_peers.push(peer.clone());
                        println!("Will attempt to connect to peer {}", peer.location());
                        peer.connect();
                    }
                    peers.remove(random_index);
                }
            }
            println!("PeerManager::connect: {}", inner.peers.len());
            if inner.connected_peers.is_empty() {
                sender.send(Command::NoPeers).unwrap();
                // this.chain_sync_stopped();
                // DispatchContext::main_context().queue(||
                //     NotificationCenter::post(
                //         Notification::ChainSyncFailed(self.chain, &Some(Error::NoPeersFound))));
            } /*else {
                loop {
                    if let Ok(Command::Disconnect) = receiver.recv() {

                    }
                }
            }*/
            // loop {
                println!("Thread running...");
                if let Ok(Command::Disconnect) = receiver.recv() {
                    println!("Disconnect peer_manager...");
                    // break
                }
                // thread::sleep(Duration::from_millis(10));
            // }

            // Notify the main thread that we're done connecting
        }).join().unwrap();
    }

    pub fn disconnect(&mut self) {
        if let Ok(mut writer) = self.inner.try_write() {
            writer.desired_state = PeerManagerDesiredState::Disconnected;
            // writer.connect_failures =
        /*DispatchContext::network_context().queue(|| {*/
            writer.connected_peers.iter_mut().for_each(|peer| {
                // prevent further automatic reconnect attempts
                // writer.connect_failures = MAX_CONNECT_FAILURES;
                peer.disconnect();
            });
        }
    }

    pub fn disconnect_download_peer<F: FnMut(bool)>(&mut self, error: Option<Error>, completion: F) {
        if let Ok(mut writer) = self.inner.try_write() {
            if let Some(peer) = &mut writer.download_peer {
                peer.disconnect_with_error(error);
            }
            /*DispatchContext::network_context().queue(|| {
                // disconnect the current download peer so a new random one will be selected
                if let Some(peer) = &self.download_peer {
                    if let Some(pos) = self.peers.iter().position(|x| x == peer) {
                        self.peers.remove(pos);
                    }
                }
                completion(true);
            });*/
        }
    }

    pub fn sync_timeout(&mut self) {
        let now = SystemTime::seconds_since_1970();
        // if now - self.chain.last_relay_time < PROTOCOL_TIMEOUT {
            // TODO: implement cancellation with token
            // the download peer relayed something in time, so restart timer
            // [NSObject cancelPreviousPerformRequestsWithTarget:self selector:@selector(syncTimeout) object:nil];
            // [self performSelector:@selector(syncTimeout)
            // withObject:nil
            // afterDelay:PROTOCOL_TIMEOUT - (now - self.chainManager.lastChainRelayTime)];
            // return;
        // }

        self.disconnect_download_peer(Some(Error::SyncTimeout), |success| {});
    }

    pub fn chain_sync_stopped(&self) {
        todo!()
        /*DispatchContext::main_context().queue(|| {
            // TODO: implement cancellation with token
            // TODO: background ops
            //[NSObject cancelPreviousPerformRequestsWithTarget:self selector:@selector(syncTimeout) object:nil];
            #if TARGET_OS_IOS
            if (self.terminalHeadersSaveTaskId != UIBackgroundTaskInvalid) {
                [[UIApplication sharedApplication] endBackgroundTask:self.terminalHeadersSaveTaskId];
                self.terminalHeadersSaveTaskId = UIBackgroundTaskInvalid;
            }

            if (self.blockLocatorsSaveTaskId != UIBackgroundTaskInvalid) {
                [[UIApplication sharedApplication] endBackgroundTask:self.blockLocatorsSaveTaskId];
                self.blockLocatorsSaveTaskId = UIBackgroundTaskInvalid;
            }
            #endif
        });*/
    }

    pub fn peer_connected(&mut self, peer: &mut Peer) {
        peer.sanitize_timestamp();
        self.inner.try_write().unwrap().connect_failures = 0;
        // println!("{}:{} connected with lastblock {} (our last header {} - last block {})", peer.host, peer.port, peer.last_block_height, self.chain.last_terminal_block_height(), self.chain.last_sync_block_height);

        self.chain.with(|chain| {
                // drop peers that don't carry full blocks, or aren't synced yet
                // TODO: XXXX does this work with 0.11 pruned nodes?
                if peer.doesnt_support_full_blocks() || peer.not_synced_yet(chain.last_sync_block_height()) {
                    peer.disconnect_with_error(Some(Error::ServiceNetworkError(peer.clone())));
                    return;
                }

                // drop peers that don't support SPV filtering
                if peer.doesnt_support_spv_filtering() {
                    peer.disconnect_with_error(Some(Error::BloomFilteringNotSupported(peer.clone())));
                    return;
                }

                if self.inner.try_read().unwrap().connected {
                    if !chain.r#type().syncs_blockchain() { return; }

                    if chain.can_construct_a_filter() {
                        peer.send_filterload_message(chain.transaction_manager.transactions_bloom_filter_for_peer_hash(peer.hash()).to_data());
                        // publish pending tx
                        peer.send_inv_message_for_hashes(chain.transaction_manager.published_callback_hashes(), InvType::Tx);
                    } else {
                        peer.send_filterload_message(BloomFilter::empty_bloom_filter_data());
                    }
                    let (estimated_block_height, last_sync_block_height) = (chain.estimated_block_height(), chain.last_sync_block_height());
                    if estimated_block_height >= peer.last_block_height() || last_sync_block_height >= peer.last_block_height() {
                        if last_sync_block_height < estimated_block_height {
                            // println!("lastSyncBlockHeight {}, estimatedBlockHeight {}", self.chain.last_sync_block_height, self.chain.estimated_block_height());
                            return; // don't get mempool yet if we're syncing
                        }
                        // peer.send_ping_message(Arc::new(|success| {
                        //     if !success {
                        //         println!("[DSTransactionManager] fetching mempool ping on connection failure peer {}", peer.host());
                        //         return;
                        //     }
                        //     println!("[DSTransactionManager] fetching mempool ping on connection success peer {}", peer.host());
                            // let published_tx_hashes = self.chain.with(|chain| chain.transaction_manager.published_tx_hashes());
                            // peer.send_mempool_message(published_tx_hashes, Arc::new(|success, needed, interrupted_by_disconnect| {
                            //     if !success {
                            //         if !needed {
                            //             println!("[DSTransactionManager] fetching mempool message on connection not needed (already happening) peer {}", peer.host());
                            //         } else if interrupted_by_disconnect {
                            //             println!("[DSTransactionManager] fetching mempool message on connection failure peer {}", peer.host());
                            //         } else {
                            //             println!("[DSTransactionManager] fetching mempool message on connection failure disconnect peer {}", peer.host());
                            //         }
                            //         return;
                            //     }
                            //     println!("[DSTransactionManager] fetching mempool message on connection success peer {}", peer.host());
                            //     peer.synced = true;
                            //     self.chain.with(|chain| chain.transaction_manager.remove_unrelayed_transactions_from_peer(peer));
                            //     // self.transaction_manager().remove_unrelayed_transactions_from_peer(peer);
                            //     if self.inner.try_read().unwrap().masternode_list.is_none() {
                            //         // request a list of other dash peers
                            //         peer.send_getaddr_message();
                            //     }
                            //     /*DispatchContext::main_context().queue(||
                            //         NotificationCenter::post(Notification::TransactionStatusDidChange(self.chain)));*/
                            // }))
                        // }));
                        /*DispatchContext::main_context().queue(||
                            NotificationCenter::post(Notification::PeersConnectedDidChange(self.chain)));*/
                        return; // we're already connected to a download peer
                    }
                }
                // select the peer with the lowest ping time to download the chain from if we're behind
                // BUG: XXX a malicious peer can report a higher lastblock to make us select them as the download peer, if two
                // peers agree on lastblock, use one of them instead
                // let really_connected_peers: Vec<_> = self.connected_peers.iter().filter(|peer| peer.status() == PeerStatus::Connected).cloned().collect();
                // if !self.chain_type.is_devnet_any() && really_connected_peers.len() < self.max_connect_count {
                //     // we didn't connect to all connected peers yet
                //     return;
                // }
                // let mut best_peer = peer.clone();
                // really_connected_peers.into_iter().for_each(|peer| {
                //     if (peer.ping_time < best_peer.ping_time &&
                //         peer.last_block_height >= best_peer.last_block_height) ||
                //         peer.last_block_height > best_peer.last_block_height {
                //         best_peer = peer;
                //     }
                //     self.chain.with(|chain| chain.set_estimated_block_height(peer.last_block_height.clone(), &peer, really_connected_peers.len() * 2 / 3));
                // });
                // if let Some(peer) = &mut self.download_peer {
                //     peer.disconnect();
                // }
                // self.download_peer = Some(best_peer);
                // self.connected = true;
                //
                // if self.chain_type.syncs_blockchain() && chain.can_construct_a_filter() {
                //     best_peer.send_filterload_message(chain.transaction_manager.transactions_bloom_filter_for_peer_hash(best_peer.hash()).to_data());
                // }

            // best_peer.current_block_height = self.chain.with(|chain| chain.last_sync_block_height());
                // if self.chain.should_continue_sync_blockchain_after_height(best_peer.last_block_height.clone()) {
                //     // start blockchain sync
                //     self.chain.reset_last_relayed_item_time();
                //     /*DispatchContext::main_context().queue(|| { // setup a timer to detect if the sync stalls
                //         // TODO: implement cancellation with token
                //         // [NSObject cancelPreviousPerformRequestsWithTarget:self selector:@selector(syncTimeout) object:nil];
                //         // [self performSelector:@selector(syncTimeout) withObject:nil afterDelay:PROTOCOL_TIMEOUT];
                //         //
                //         NotificationCenter::post(Notification::TransactionStatusDidChange(self.chain));
                //         self.chain.chain_will_start_syncing_blockchain();
                //         self.chain.chain_should_start_syncing_blockchain(&mut best_peer);
                //     });*/
                // } else {
                //     // we're already synced
                //     self.chain.chain_finished_syncing_transactions_and_blocks(None, true);
                // }
                /*DispatchContext::main_context().queue(|| {
                    NotificationCenter::post(Notification::PeersConnectedDidChange(self.chain));
                    NotificationCenter::post(Notification::PeersDownloadPeerDidChange(self.chain));
            remoe    });*/
        });


    }

    pub fn peer_disconnected_with_error(&mut self, peer: &mut Peer, error: Option<Error>) {
        // println!("{}:{} disconnected{}{}", peer.host(), peer.port, if error.is_some() { ", " } else { "" }, if error.is_some() { error.unwrap().message() } else { "" });
        if let Some(err) = error {
            if err.is_app_level() {
                self.peer_misbehaving(peer, err.to_string());
            } else if let Ok(mut writer) = self.inner.try_write() {
                writer.remove_peer(peer);
            }
        }
        self.chain.with(|chain| chain.transaction_manager.clear_transaction_relays_for_peer(peer));
        if let Ok(mut writer) = self.inner.try_write() {
            if let Some(d_peer) = writer.download_peer.take() {
                if d_peer == *peer {
                    // download peer disconnected
                    writer.connected = false;
                    self.chain.with(|chain| chain.remove_estimated_block_heights_of_peer(peer));
                    writer.download_peer = None;
                    if writer.connect_failures > MAX_CONNECT_FAILURES {
                        writer.connect_failures = MAX_CONNECT_FAILURES;
                    }
                }
            }

            if !writer.connected && writer.connect_failures == MAX_CONNECT_FAILURES {
                self.chain_sync_stopped();
                writer.clear_all_peers()
                /*DispatchContext::main_context().queue(||
                    NotificationCenter::post(
                        Notification::ChainSyncFailed(self.chain, &error)))*/
            } else if writer.connect_failures < MAX_CONNECT_FAILURES {
                /*DispatchContext::main_context().queue(|| {
                    // TODO: impl background ops
                    // #if TARGET_OS_IOS
                    // if ((self.desiredState == DSPeerManagerDesiredState_Connected) && (self.terminalHeadersSaveTaskId != UIBackgroundTaskInvalid ||
                    //     [UIApplication sharedApplication].applicationState != UIApplicationStateBackground)) {
                    // [self connect]; // try connecting to another peer
                    // }
                    // #else
                    // if (self.desiredState == DSPeerManagerDesiredState_Connected) {
                    //     [self connect]; // try connecting to another peer
                    // }
                    // #endif

                });*/
            }
            /*DispatchContext::main_context().queue(|| {
                NotificationCenter::post(Notification::PeersConnectedDidChange(self.chain));
                NotificationCenter::post(Notification::PeersDownloadPeerDidChange(self.chain));
            });*/
        }

    }

    pub fn peers_relayed(&mut self, peer: &Peer, peers: Vec<Peer>) {
        println!("{:?} relayed {} peer(s)", peer, peers.len());
        if let Ok(mut writer) = self.inner.try_write() {
            let num_peers = peers.len();
            writer.add_peers(peers);
            if (2..1000).contains(&num_peers) {
                // peer relaying is complete when we receive <1000
                self.save_peers();
            }
            /*DispatchContext::main_context().queue(||
                NotificationCenter::post(
                    Notification::PeersDidChange(self.chain)));*/
        }


    }
}
