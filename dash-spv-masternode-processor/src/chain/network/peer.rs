use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::Hasher;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;
use byte::{BytesExt, Iter};
use byte::ctx::Bytes;
use secp256k1::rand::{Rng, thread_rng};
use crate::chain::block::{IBlock, merkle_block, MerkleBlock};
use crate::chain::{Chain, governance, SyncCountInfo, SyncType};
use crate::chain::{chain_lock, spork};
use crate::chain::chain_lock::ChainLock;
use crate::chain::common::ChainType;
use crate::chain::ext::governance::PeerGovernanceDelegate;
use crate::chain::ext::masternodes::PeerMasternodeDelegate;
use crate::chain::ext::peers::PeerChainDelegate;
use crate::chain::ext::transactions::PeerTransactionDelegate;
use crate::chain::network::governance_request_state::GovernanceRequestState;
use crate::chain::network::message::inv_hash::InvHash;
use crate::chain::network::message::inv_type::InvType;
use crate::chain::network::message::message::{MessageType, Payload};
use crate::chain::network::message::request::Request;
use crate::chain::network::peer_type::PeerType;
use crate::chain::network::PeerStatus;
use crate::chain::spork::manager::PeerSporkDelegate;
use crate::chain::spork::Spork;
use crate::chain::tx;
use crate::chain::tx::instant_send_lock;
use crate::chain::tx::{InstantSendLock, ITransaction};
use crate::chain::tx::protocol::ReadContext;
use crate::consensus::Encodable;
use crate::consensus::encode::VarInt;
use crate::crypto::{UInt128, UInt256};
use crate::crypto::byte_util::{AsBytes, Zeroable};
use crate::crypto::data_ops::{extend_unique, extract_new_and_unique};
use crate::manager::peer_manager::Error;
use crate::models::MasternodeEntry;
use crate::chain::network::message::addr::{Addr, AddrInfo};
use crate::chain::network::message::not_found::NotFound;
use crate::chain::network::message::reject::Reject;
use crate::chain::network::message::version::Version;
use crate::util::{Shared, TimeUtil};
use crate::util::data_ops::{hex_with_data, short_hex_string_from};

pub const WEEK_TIME_INTERVAL: u64 = 604800; //7*24*60*60
pub const DAY_TIME_INTERVAL: u64 = 86400;   //24*60*60
pub const DAYS_3_TIME_INTERVAL: u64 = 86400 * 3;
pub const WEAK_TIME_INTERVAL: u64 = 86400 * 7;
pub const HOUR_TIME_INTERVAL: u64 = 3600;
pub const HOURS_3_TIME_INTERVAL: u64 = 10800;
/// we don't provide full blocks to remote nodes
pub const ENABLED_SERVICES: u64 = 0;
pub const LOCAL_HOST: u32 = 0x7f000001;
pub const MAX_MSG_LENGTH: usize = 0x02000000;
pub const MAX_GETDATA_HASHES: usize = 50000;
pub const MEMPOOL_TIMEOUT: u64 = 2;

// pub type PongCallback = Box<dyn Fn(bool) + Send + Sync>;
// pub type MempoolTransactionCallback = Box<dyn Fn(bool, bool, bool) + Send + Sync>;

pub trait PongCallback: FnMut(bool) + Send + Sync {}
impl<T: FnMut(bool) + Send + Sync + Clone + 'static> PongCallback for T {}
pub trait MempoolTransactionCallback: FnMut(bool, bool, bool) + Send + Sync {}
impl<T: FnMut(bool, bool, bool) + Send + Sync + Clone + 'static> MempoolTransactionCallback for T {}


const FNV32_PRIME: u32 = 0x01000193;
const FNV32_OFFSET: u32 = 0x811C9dc5;

// const SERVER: Token = Token(0);
// const CLIENT: Token = Token(1);



#[derive(Clone)]
pub struct Peer {
    pub socket_addr: SocketAddr,
    version: Version,
    pub priority: u32,
    pub status: PeerStatus,
    pub last_block_hash: UInt256,
    pub misbehaving: i16,
    pub low_preference_till: u64,

    pub ping_time: u64,
    local_nonce: u64,
    ping_start_time: u64,
    /// headers or block->totalTx per second being relayed
    relay_speed: u64,
    relay_start_time: u64,

    pub last_requested_masternode_list: Option<u64>,
    pub last_requested_governance_sync: Option<u64>,
    /// set this to the timestamp when the wallet was created to improve initial sync time (interval since reference date)
    pub earliest_key_time: u64,

    /// set this when wallet addresses need to be added to bloom filter
    pub needs_filter_update: bool,
    /// set this to local block height (helps detect tarpit nodes)
    pub current_block_height: u32,
    /// use this to keep track of peer state
    pub synced: bool,

    /// minimum tx fee rate peer will accept
    pub fee_per_byte: u64,

    pub(crate) sent_getaddr: bool,
    sent_getdatatxblocks: bool,
    sent_getdatamasternode: bool,
    sent_filter: bool,
    sent_getblocks: bool,
    sent_getheaders: bool,
    sent_mempool: bool,
    sent_verack: bool,
    got_verack: bool,


    received_orphan_count: u32,

    pong_handlers: Vec<Arc<dyn PongCallback<Output=()>>>,

    mempool_transaction_callback: Option<Arc<dyn MempoolTransactionCallback<Output=()>>>,
    mempool_request_time: u64,

    known_block_hashes: Vec<UInt256>,
    known_chain_lock_hashes: Vec<UInt256>,
    known_tx_hashes: Vec<UInt256>,
    known_is_lock_hashes: Vec<UInt256>,
    known_isd_lock_hashes: Vec<UInt256>,
    known_governance_object_hashes: Vec<UInt256>,
    known_governance_vote_hashes: Vec<UInt256>,

    governance_request_state: GovernanceRequestState,

    current_block: Option<MerkleBlock>,
    current_block_tx_hashes: Option<Vec<UInt256>>,


    // transaction_delegate: &'static TransactionManager,
    // governance_delegate: &'static GovernanceSyncManager,
    // spork_delegate: &'static spork::Manager,
    // masternode_delegate: &'static MasternodeManager,
    // delegate_context: &'static DispatchContext,
    pub chain: Shared<Chain>,
    pub chain_type: ChainType,

    // reachability: Option<ReachabilityManager>,
    // reachability_handler: Option<Arc<dyn ReachabilityStatusCallback>>,
    // handle: Option<thread::JoinHandle<()>>,

    // read_buffer: BufReader<TcpStream>,
    // write_buffer: BufWriter<TcpStream>,

    // poll: Poll,
    // events: Events,

}

impl Debug for Peer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {} {:?}", self.socket_addr, self.chain_type.unique_id(), self.status)?;
        Ok(())
    }
}

// two peer objects are equal if they share an ip address and port number
impl PartialEq<Self> for Peer {
    fn eq(&self, other: &Self) -> bool {
        self == other || self.socket_addr == other.socket_addr
    }
}

impl std::hash::Hash for Peer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // let mut hash = FNV32_OFFSET;
        // (0..std::mem::size_of::<UInt128>()).for_each(|i| {
        //     hash = hash ^ (self.address.0[i] as u32) * FNV32_PRIME;
        // });
        // hash = (hash ^ ((self.port >> 8) & 0xff) as u32) * FNV32_PRIME;
        // hash = (hash ^ (self.port & 0xff) as u32) * FNV32_PRIME;
        // state.write(&hash.to_le_bytes());
        state.write(self.socket_addr.ip().to_string().as_bytes());
        state.write_u16(self.socket_addr.port())
    }
}


// #define FNV32_PRIME 0x01000193u
// #define FNV32_OFFSET 0x811C9dc5u
//
// // FNV32-1a hash of the ip address and port number: http://www.isthe.com/chongo/tech/comp/fnv/index.html#FNV-1a
// - (NSUInteger)hash {
// uint32_t hash = FNV32_OFFSET;
//
// for (int i = 0; i < sizeof(_address); i++) {
// hash = (hash ^ _address.u8[i]) * FNV32_PRIME;
// }
//
// hash = (hash ^ ((_port >> 8) & 0xff)) * FNV32_PRIME;
// hash = (hash ^ (_port & 0xff)) * FNV32_PRIME;
// return hash;
// }

impl Peer {

    pub(crate) fn peer_with_host(host: &str, chain: Shared<Chain>) -> Option<Peer> {
        todo!()
    }

    pub fn new(socket_addr: SocketAddr, timestamp: u64, services: u64, chain_type: ChainType, chain: Shared<Chain>) -> Self {
        Self {
            socket_addr,
            version: Version {
                timestamp,
                services,
                last_block_height: 0,
                nonce: 0,
                addr_recv_services: 0,
                addr_recv_address: Default::default(),
                addr_recv_port: 0,
                addr_trans_services: 0,
                addr_trans_address: Default::default(),
                version: 0,
                useragent: "".to_string(),
                addr_trans_port: 0 },
            chain_type,
            chain,
            // poll: Poll::new().unwrap(),
            // events: Events::with_capacity(128),
            // read_buffer: BufReader::new(),
            // write_buffer: (),
            priority: 0,
            status: PeerStatus::Unknown,
            last_block_hash: Default::default(),
            misbehaving: 0,
            low_preference_till: 0,
            ping_time: 0,
            local_nonce: 0,
            ping_start_time: 0,
            relay_speed: 0,
            relay_start_time: 0,
            last_requested_masternode_list: None,
            last_requested_governance_sync: None,
            earliest_key_time: 0,
            needs_filter_update: false,
            current_block_height: 0,
            synced: false,
            fee_per_byte: 0,
            sent_getaddr: false,
            sent_getdatatxblocks: false,
            sent_getdatamasternode: false,
            sent_filter: false,
            sent_getblocks: false,
            sent_getheaders: false,
            sent_mempool: false,
            sent_verack: false,
            got_verack: false,
            received_orphan_count: 0,
            pong_handlers: vec![],
            mempool_transaction_callback: None,
            mempool_request_time: 0,
            known_block_hashes: vec![],
            known_chain_lock_hashes: vec![],
            known_tx_hashes: vec![],
            known_is_lock_hashes: vec![],
            known_isd_lock_hashes: vec![],
            known_governance_object_hashes: vec![],
            known_governance_vote_hashes: vec![],
            governance_request_state: GovernanceRequestState::None,
            current_block: None,
            current_block_tx_hashes: None,
            // reachability: None,
            // reachability_handler: None,
            // handle: None,
        }
    }

    pub fn init_with_masternode(masternode: &MasternodeEntry, chain_type: ChainType, chain: Shared<Chain>) -> Self {
        Self::new(
            SocketAddr::new(masternode.socket_address.ip_address.to_ip_addr(), if masternode.socket_address.port == 0 { chain_type.standard_port() } else { masternode.socket_address.port }),
            0,
            0,
            chain_type,
            chain)
    }
    pub fn init_with_addr_info(addr_info: AddrInfo, chain_type: ChainType, chain: Shared<Chain>) -> Self {
        Self::new(addr_info.socket_addr, addr_info.timestamp, addr_info.services, chain_type, chain)
    }

    pub fn init_with_address(address: IpAddr, port: u16, chain_type: ChainType, chain: Shared<Chain>, timestamp: u64, services: u64) -> Self {
        Self::new(
            SocketAddr::new(address, if port == 0 { chain_type.standard_port() } else { port }),
            timestamp,
            services,
            chain_type,
            chain)
    }

    pub fn init_with_socket_addr(socket_addr: SocketAddr, chain_type: ChainType, chain: Shared<Chain>, timestamp: u64, services: u64) -> Self {
        Self::new(
            socket_addr,
            timestamp,
            services,
            chain_type,
            chain)
    }

    fn reset(&mut self) {
        self.received_orphan_count = 0;

        // self.msgHeader = [NSMutableData data];
        // self.msgPayload = [NSMutableData data];
        // self.outputBuffer = [NSMutableData data];
        // self.read_buffer.res
        self.got_verack = false;
        self.sent_verack = false;
        self.sent_filter = false;
        self.sent_getaddr = false;
        self.sent_getdatatxblocks = false;
        self.sent_getdatamasternode = false;
        self.sent_mempool = false;
        self.sent_getblocks = false;
        self.needs_filter_update = false;
        self.known_tx_hashes.clear();
        self.known_is_lock_hashes.clear();
        self.known_isd_lock_hashes.clear();
        self.known_block_hashes.clear();
        self.known_chain_lock_hashes.clear();
        self.known_governance_object_hashes.clear();
        self.known_governance_object_hashes.clear();
        self.current_block = None;
        self.current_block_tx_hashes = None;
        // self.managedObjectContext = [NSManagedObjectContext peerContext];
    }

    pub fn location(&self) -> String {
        format!("{}", self.socket_addr)
    }

    pub fn host(&self) -> String {
        self.socket_addr.ip().to_string()
    }

    pub fn status(&self) -> PeerStatus {
        self.status
    }

    pub fn r#type(&self) -> PeerType {
        todo!()
    }

    pub fn last_block_height(&self) -> u32 {
        self.version.last_block_height
    }

    pub fn services(&self) -> u64 {
        self.version.services
    }

    pub fn timestamp(&self) -> u64 {
        self.version.timestamp
    }

    pub fn doesnt_support_spv_filtering(&self) -> bool {
        self.version.doesnt_support_spv_filtering()
    }

    pub fn doesnt_support_full_blocks(&self) -> bool {
        self.version.doesnt_support_full_blocks()
    }

    pub fn not_synced_yet(&self, height: u32) -> bool {
        self.version.not_synced_yet(height)
    }

    pub fn sanitize_timestamp(&mut self) {
        let now = SystemTime::seconds_since_1970();
        if self.timestamp() > now + 2 * 60 * 60 || self.timestamp() < now - 2 * 60 * 60 {
            // timestamp sanity check
            self.version.timestamp = now;
        }
    }

    // pub fn useragent(&self) -> String {
    //     self.chain.user_agent()
    // }

    pub fn send_request(&self, request: Request) {
        let r#type = request.r#type();
        let message = request.compile();
        //println!("{}:{} sendRequest: [{}]: {}}", self.host(), self.port, r#type, payload.to_hex());
        self.send_message(message, r#type);
    }

    fn send_message<T: AsRef<[u8]>>(&self, message: T, r#type: MessageType) {
        let bytes = message.as_ref();
        if bytes.len() > MAX_MSG_LENGTH {
            println!("{} failed to send {:?}, length {} is too long", self.socket_addr, r#type, bytes.len());
            return;
        }
        // TODO: implement p2p message sending
        /*
        if (!self.runLoop) return;
        CFRunLoopPerformBlock([self.runLoop getCFRunLoop], kCFRunLoopCommonModes, ^{
            LOCK(self.outputBufferSemaphore);
            // magic_number from Chain
            [self.outputBuffer appendMessage:message type:type forChain:self.chain];
            while (self.outputBuffer.length > 0 && self.outputStream.hasSpaceAvailable) {
                NSInteger l = [self.outputStream write:self.outputBuffer.bytes maxLength:self.outputBuffer.length];
                if (l > 0) [self.outputBuffer replaceBytesInRange:NSMakeRange(0, l) withBytes:NULL length:0];
                //if (self.outputBuffer.length == 0) DSLog(@"%@:%u output buffer cleared", self.host, self.port);
            }
            UNLOCK(self.outputBufferSemaphore);
        });
        CFRunLoopWakeUp([self.runLoop getCFRunLoop]);
        */
    }

    fn send_version_message(&mut self) {
        self.local_nonce = thread_rng().gen::<u64>() << 32 | thread_rng().gen::<u64>();
        self.ping_start_time = SystemTime::seconds_since_1970();
        self.send_request(Request::Version(self.socket_addr, self.services(), self.local_nonce, self.chain_type));
    }

    pub fn send_verack_message(&mut self) {
        self.send_request(Request::Default(MessageType::Verack));
        self.sent_verack = true;
        self.did_connect();
    }

    pub fn send_filterload_message(&mut self, filter: Vec<u8>) {
        self.sent_filter = true;
        println!("Sending filter with fingerprint {} to node {} {}", "<REDACTED>", self.host(), "");
        self.send_request(Request::FilterLoad(filter));
    }

    pub fn mempool_timeout(&mut self) {
        println!("[DSPeer] mempool time out {}", self.host());
        self.send_ping_message(Arc::new(|success| {
            // if let Some(completion) = &self.mempool_transaction_callback {
            //     completion(success, true, false);
            // }
        }));
        self.mempool_transaction_callback = None;
    }

    pub fn send_mempool_message(&mut self, published_tx_hashes: Vec<UInt256>, completion: Arc<dyn MempoolTransactionCallback>) {
        println!("{} send_mempool_message", self.socket_addr);
        self.known_tx_hashes.extend(published_tx_hashes);
        self.sent_mempool = true;
        // TODO: impl async callback
        if let Some(mempool_callback) = &self.mempool_transaction_callback {
            //dispatch_async(self.delegateQueue, ^{
            // if self.status() == PeerStatus::Connected {
            //     completion(false, false, false);
            // }
            //});
        } else {
            self.mempool_transaction_callback = Some(completion);
            //dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(MEMPOOL_TIMEOUT * NSEC_PER_SEC)), self.delegateQueue, ^{
            if SystemTime::seconds_since_1970() - self.mempool_request_time >= MEMPOOL_TIMEOUT {
                self.mempool_timeout();
            }
            //});
        }
        self.mempool_request_time = SystemTime::seconds_since_1970();
        self.send_request(Request::Default(MessageType::Mempool));
    }

    // the standard blockchain download protocol works as follows (for SPV mode):
    // - local peer sends getblocks
    // - remote peer reponds with inv containing up to 500 block hashes
    // - local peer sends getdata with the block hashes
    // - remote peer responds with multiple merkleblock and tx messages
    // - remote peer sends inv containg 1 hash, of the most recent block
    // - local peer sends getdata with the most recent block hash
    // - remote peer responds with merkleblock
    // - if local peer can't connect the most recent block to the chain (because it started more than 500 blocks behind), go
    //   back to first step and repeat until entire chain is downloaded
    //
    // we modify this sequence to improve sync performance and handle adding bip32 addresses to the bloom filter as needed:
    // - local peer sends getheaders
    // - remote peer responds with up to 2000 headers
    // - local peer immediately sends getheaders again and then processes the headers
    // - previous two steps repeat until a header within a week of earliestKeyTime is reached (further headers are ignored)
    // - local peer sends getblocks
    // - remote peer responds with inv containing up to 500 block hashes
    // - local peer sends getdata with the block hashes
    // - if there were 500 hashes, local peer sends getblocks again without waiting for remote peer
    // - remote peer responds with multiple merkleblock and tx messages, followed by inv containing up to 500 block hashes
    // - previous two steps repeat until an inv with fewer than 500 block hashes is received
    // - local peer sends just getdata for the final set of fewer than 500 block hashes
    // - remote peer responds with multiple merkleblock and tx messages
    // - if at any point tx messages consume enough wallet addresses to drop below the bip32 chain gap limit, more addresses
    //   are generated and local peer sends filterload with an updated bloom filter
    // - after filterload is sent, getdata is sent to re-request recent blocks that may contain new tx matching the filter
    pub fn send_getheaders_message_with_locators(&mut self, locators: Vec<UInt256>, hash_stop: UInt256) {
        let request = Request::GetHeaders(locators, hash_stop, self.chain_type.protocol_version());
        if self.relay_start_time == 0 {
            self.relay_start_time = SystemTime::seconds_since_1970();
        }
        self.send_request(request);
    }

    pub fn send_getblocks_message_with_locators(&mut self, locators: Vec<UInt256>, hash_stop:UInt256) {
        // DSGetBlocksRequest *request = [DSGetBlocksRequest requestWithLocators:locators andHashStop:hashStop protocolVersion:self.chain.protocolVersion];
        let request = Request::GetBlocks(locators, hash_stop, self.chain_type.protocol_version());
        self.sent_getblocks = true;
        self.send_request(request);
    }

    pub fn send_inv_message_for_hashes(&mut self, inv_hashes: Vec<UInt256>, inv_type: InvType) {
        println!("{} sending inv message of type {} hashes count {}", self.socket_addr, inv_type.name(), inv_hashes.len());
        let mut hashes = inv_hashes.clone();
        hashes.retain(|x| !self.known_tx_hashes.contains(x));
        if hashes.is_empty() {
            return;
        }
        let request = Request::Inv(inv_type, hashes.clone());
        self.send_request(request);
        // todo!();
        match inv_type {
            InvType::Tx => extend_unique(&mut self.known_tx_hashes, hashes),
            InvType::GovernanceObjectVote => extend_unique(&mut self.known_governance_vote_hashes, hashes),
            InvType::GovernanceObject => extend_unique(&mut self.known_governance_object_hashes, hashes),
            InvType::Block => extend_unique(&mut self.known_block_hashes, hashes),
            InvType::ChainLockSignature => extend_unique(&mut self.known_chain_lock_hashes, hashes),
            _ => {}
        }
    }

    pub fn send_transaction_inv_messages_for_transaction_hashes(&mut self, tx_inv_hashes: Option<Vec<UInt256>>, tx_lock_request_inv_hashes: Option<Vec<UInt256>>) {
        let mut new_tx_hashes = tx_inv_hashes.map(|hashes| hashes.into_iter().filter(|x| !self.known_tx_hashes.contains(x)).collect::<Vec<_>>()).unwrap_or(vec![]);
        let new_tx_lock_request_hashes = tx_lock_request_inv_hashes.map(|hashes| hashes.into_iter().filter(|x| !self.known_tx_hashes.contains(x)).collect::<Vec<_>>()).unwrap_or(vec![]);
        if new_tx_hashes.is_empty() && new_tx_lock_request_hashes.is_empty() {
            return;
        }
        let request = Request::TransactionInv(new_tx_hashes.clone(), new_tx_lock_request_hashes.clone());
        self.send_request(request);
        extend_unique(&mut new_tx_hashes, new_tx_lock_request_hashes);
        self.known_tx_hashes.extend(new_tx_hashes);
    }

    pub fn send_getdata_message_for_tx_hash(&mut self, tx_hash: UInt256) {
        if self.chain_type.sync_type().bits() & SyncType::GetsNewBlocks.bits() == 0 {
            return;
        }
        self.send_request(Request::GetDataForTransactionHash(tx_hash));
    }

    pub fn send_getdata_message_with_tx_hashes(&mut self, tx_hashes: Option<Vec<UInt256>>, is_lock_hashes: Option<Vec<UInt256>>, isd_lock_hashes: Option<Vec<UInt256>>, block_hashes: Option<Vec<UInt256>>, c_lock_hashes: Option<Vec<UInt256>>) {
        if self.chain_type.sync_type().bits() & SyncType::GetsNewBlocks.bits() == 0 {
            return;
        }
        let tx_hashes_len = tx_hashes.as_ref().map_or(0, |hashes| hashes.len());
        let is_lock_hashes_len = is_lock_hashes.as_ref().map_or(0, |hashes| hashes.len());
        let isd_lock_hashes_len = isd_lock_hashes.as_ref().map_or(0, |hashes| hashes.len());
        let block_hashes_len = block_hashes.as_ref().map_or(0, |hashes| hashes.len());
        let c_lock_hashes_len = c_lock_hashes.as_ref().map_or(0, |hashes| hashes.len());
        let total_hashes = tx_hashes_len + is_lock_hashes_len + isd_lock_hashes_len + block_hashes_len + c_lock_hashes_len;
        if total_hashes > MAX_GETDATA_HASHES {
            // limit total hash count to MAX_GETDATA_HASHES
            println!("{} couldn't send getdata, {} is too many items, max is {}", self.socket_addr, total_hashes, MAX_GETDATA_HASHES);
            return;
        } else if total_hashes == 0 {
            return;
        }
        self.sent_getdatatxblocks = true;
        self.send_request(Request::GetDataForTransactionHashes(
            tx_hashes,
            block_hashes,
            is_lock_hashes,
            isd_lock_hashes,
            c_lock_hashes
        ));
    }

    // pub fn send_governance_request(&mut self, request: Request, state: GovernanceRequestState) {
    //     // return;
    //
    //     if hashes.len() > MAX_GETDATA_HASHES { // limit total hash count to MAX_GETDATA_HASHES
    //         println!("{}:{} couldn't send governance votes getdata, {} is too many items, max is {}", self.host(), self.port, hashes.len(), MAX_GETDATA_HASHES);
    //         return;
    //     } else if hashes.len() == 0 {
    //         println!("{}:{} couldn't send governance getdata, there is no items", self.host(), self.port);
    //         return;
    //     }
    //     Request::GovernanceHashes(state)
    //     self.send_request(request.message_request());
    // }

    pub fn send_getaddr_message(&mut self) {
        self.sent_getaddr = true;
        self.send_request(Request::Default(MessageType::Getaddr));
    }

    pub fn send_ping_message(&mut self, callback: Arc<dyn PongCallback>) {
        // TODO: async?
        //dispatch_async(self.delegateQueue, ^{
        self.pong_handlers.push(callback);
        self.ping_start_time = SystemTime::seconds_since_1970();
        self.send_request(Request::Ping(self.local_nonce));
        //});
    }

    /// re-request blocks starting from blockHash, useful for getting any additional transactions after a bloom filter update
    pub fn rerequest_blocks_from(&mut self, block_hash: &UInt256 ) {
        if let Some(pos) = self.known_block_hashes.iter().position(|hash| hash == block_hash) {
            self.known_block_hashes.drain(0..pos);
            println!("{} re-requesting {} blocks", self.socket_addr, self.known_block_hashes.len());
            self.send_getdata_message_with_tx_hashes(None, None, None, Some(self.known_block_hashes.clone()), None);
        }
    }

    /// Sporks
    pub fn send_get_sporks(&self) {
        self.send_request(Request::Default(MessageType::GetSporks))
    }

    /// Governance
    /// Synchronization for Votes and Objects
    pub fn send_governance_sync_request(&mut self, request: Request, state: GovernanceRequestState) {
        // Make sure we aren't in a governance sync process
        println!("{} Requesting Governance Object Vote Hashes", self.socket_addr);
        if self.governance_request_state != GovernanceRequestState::None {
            println!("{} Requesting Governance Object Hashes out of resting state", self.socket_addr);
            return;
        }
        self.governance_request_state = state.clone();
        self.send_request(request);
        if GovernanceRequestState::GovernanceObjectHashes.eq(&state) {
            // we aren't afraid of coming back here within 5 seconds because a peer can only sendGovSync once every 3 hours
            //dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(10 * NSEC_PER_SEC)), dispatch_get_main_queue(), ^{
            if self.governance_request_state == GovernanceRequestState::GovernanceObjectHashes {
                println!("{} Peer ignored request for governance object hashes", self.socket_addr);
                //[self.governanceDelegate peer:self ignoredGovernanceSync:DSGovernanceRequestState_GovernanceObjectHashes];
            }
            //});
        }
    }

    pub fn send_gov_object_vote(&self, vote: governance::Vote) {
        self.send_message(vote.data_message(), MessageType::Govobjvote);
    }

    pub fn send_gov_object(&self, object: governance::Object) {
        self.send_message(object.data_message(), MessageType::Govobj);
    }

    /// Accept
    pub fn accept_message(&mut self, message: &[u8], r#type: MessageType) {
        if self.current_block.is_some() && (!(r#type == MessageType::Tx || r#type == MessageType::Ix || r#type == MessageType::Islock || r#type == MessageType::Isdlock))  {
            let hash = self.current_block.as_ref().unwrap().block_hash();
            self.current_block = None;
            self.current_block_tx_hashes = None;
            self.disconnect_with_error(Some(Error::Default(format!("incomplete merkleblock {}, got {:?}", hash, r#type))));
            return;
        }
        match r#type {
            MessageType::Version => self.accept_version_message(message),
            MessageType::Verack => self.accept_verack_message(message),
            MessageType::Addr => self.accept_addr_message(message),
            MessageType::AddrV2 => self.accept_addrv2_message(message),
            MessageType::Inv => self.accept_inv_message(message),
            MessageType::Getdata => self.accept_getdata_message(message),
            MessageType::NotFound => self.accept_notfound_message(message),
            MessageType::Tx | MessageType::Ix => self.accept_tx_message(message),
            MessageType::Islock => self.accept_islock_message(message, false),
            MessageType::Isdlock => self.accept_islock_message(message, true),
            MessageType::Chainlock => self.accept_chain_lock_message(message),
            MessageType::Headers => self.accept_headers_message(message),
            MessageType::Getaddr => self.accept_getaddr_message(message),
            MessageType::Ping => self.accept_ping_message(message),
            MessageType::Pong => self.accept_pong_message(message),
            MessageType::Merkleblock => self.accept_merkleblock_message(message),
            MessageType::Reject => self.accept_reject_message(message),
            MessageType::Feefilter => self.accept_fee_filter_message(message),
            MessageType::Spork => self.accept_spork_message(message),
            MessageType::Ssc => self.accept_ssc_message(message),
            MessageType::Mnlistdiff => self.accept_mnlistdiff_message(message),
            MessageType::Qrinfo => self.accept_qrinfo_message(message),
            MessageType::Govobj => self.accept_gov_object_message(message),
            MessageType::Govobjvote => self.accept_gov_object_vote_message(message),
            _ => {},
        }
    }

    pub fn accept_version_message(&mut self, message: &[u8]) {
        match message.read_with::<Version>(&mut 0, self.chain_type) {
            Ok(version) => {
                self.version = version;
                self.send_verack_message();
            },
            Err(err) => self.disconnect_with_error(Some(err.into()))
        }
    }

    pub fn accept_verack_message(&mut self, message: &[u8]) {
        if self.got_verack {
            println!("{} got unexpected verack", self.socket_addr);
        } else {
            // use verack time as initial ping time
            self.ping_time = SystemTime::seconds_since_1970() - self.ping_start_time;
            self.ping_start_time = 0;
            self.got_verack = true;
            self.did_connect();
        }
    }

    // pub fn decode<T, Ctx>(message: &[u8], ctx: Ctx) -> Result<T, byte::Error> {
    //     message.read_with::<T>(&mut 0, ctx)
    // }

    /// TODO: relay addresses
    pub fn accept_addr_message(&mut self, message: &[u8]) {
        match message.read_with::<Addr>(&mut 0, self.sent_getaddr) {
            Ok(addr) => {
                self.chain.peer_relayed_peers(self, addr);
                //dispatch_async(self.delegateQueue, ^{
                // if self.status() == PeerStatus::Connected {
                //     [self.peerDelegate peer:self relayedPeers:peers]
                // }
                //});
            },
            Err(err) => self.disconnect_with_error(Some(err.into()))
        }
    }

    pub fn accept_addrv2_message(&self, message: &[u8]) {
        println!("{} sendaddrv2, len:{}, (not implemented)", self.socket_addr, message.len());
    }

    pub fn accept_inv_message(&mut self, message: &[u8]) {
        let offset = &mut 0;
        let count = message.read_with::<VarInt>(offset, byte::LE).unwrap();
        if count.len() == 0 || message.len() < count.len() + count.0 as usize * 36 {
            self.disconnect_with_error(Some(Error::Default(format!("malformed inv message, length is {}, should be {} for {} items", message.len(), if count.len() == 0 { 1 } else { count.len() + count.0 as usize * 36}, count.0))));
            return;
        } else if count.0 > MAX_GETDATA_HASHES as u64 {
            println!("{} dropping inv message, {} is too many items, max is {}", self.socket_addr, count.0, MAX_GETDATA_HASHES);
            return;
        }
        let mut tx_hashes = Vec::<UInt256>::new();
        let mut is_lock_hashes = Vec::<UInt256>::new();
        let mut isd_lock_hashes = Vec::<UInt256>::new();
        let mut c_lock_hashes = Vec::<UInt256>::new();
        let mut block_hashes = Vec::<UInt256>::new();
        let mut spork_hashes = Vec::<UInt256>::new();
        let mut governance_object_hashes = Vec::<UInt256>::new();
        let governance_object_vote_hashes = Vec::<UInt256>::new();
        let mut only_private_send_transactions = false;
        (count.len()..count.len() + 36 * count.0 as usize)
            .step_by(36)
            .for_each(|mut off| {
                let inv_hash = message.read_with::<InvHash>(&mut off, byte::LE).unwrap();
                if !inv_hash.hash.is_zero() {
                    if inv_hash.r#type != InvType::Tx {
                        only_private_send_transactions = false;
                    } else if off == count.len() {
                        only_private_send_transactions = true;
                    }
                    match inv_hash.r#type {
                        InvType::Tx | InvType::TxLockRequest => tx_hashes.push(inv_hash.hash),
                        InvType::Block | InvType::Merkleblock => block_hashes.push(inv_hash.hash),
                        InvType::InstantSendLock => is_lock_hashes.push(inv_hash.hash),
                        InvType::InstantSendDeterministicLock => isd_lock_hashes.push(inv_hash.hash),
                        InvType::Spork => spork_hashes.push(inv_hash.hash),
                        InvType::GovernanceObject => governance_object_hashes.push(inv_hash.hash),
                        InvType::ChainLockSignature => c_lock_hashes.push(inv_hash.hash),
                        InvType::DSTx |
                        InvType::TxLockVote |
                        InvType::MasternodePing |
                        InvType::MasternodeVerify |
                        InvType::MasternodeBroadcast |
                        InvType::QuorumFinalCommitment |
                        InvType::DummyCommitment |
                        InvType::QuorumContribution |
                        InvType::CompactBlock |
                        InvType::QuorumPrematureCommitment |
                        InvType::GovernanceObjectVote |
                        InvType::MasternodePaymentVote => {},
                        _ => assert!(false, "inventory type not dealt with: {:?}", inv_hash.r#type)
                    }
                }
            });

        if self.chain_type.syncs_blockchain() && !self.sent_filter && !self.sent_mempool && !self.sent_getblocks && !tx_hashes.is_empty() && !only_private_send_transactions {
            self.disconnect_with_error(Some(Error::Default(format!("got tx inv message before loading a filter"))));
            return;
        } else if tx_hashes.len() + is_lock_hashes.len() + isd_lock_hashes.len() > 10000 {
            // this was happening on testnet, some sort of DOS/spam attack?
            println!("{} too many transactions, disconnecting", self.socket_addr);
            // disconnecting seems to be the easiest way to mitigate it
            self.disconnect();
            return;
        } else if self.current_block_height > 0 &&
            (3..500).contains(&block_hashes.len()) &&
            self.current_block_height + ((self.known_block_hashes.len() + block_hashes.len()) as u32) < self.last_block_height() {
            self.disconnect_with_error(Some(Error::Default(format!("non-standard inv, {} is fewer block hashes than expected", block_hashes.len()))));
            return;
        }

        if block_hashes.len() == 1 && self.last_block_hash.eq(&block_hashes[0]) {
            block_hashes.clear();
        }
        if block_hashes.len() == 1 {
            self.last_block_hash = block_hashes[0];
        }
        // todo

        /*if !block_hashes.is_empty() {
            // remember blockHashes in case we need to re-request them with an updated bloom filter
            self.delegate_context.queue(|| {
                let known = HashSet::from_iter(self.known_block_hashes.into_iter());
                let iter = HashSet::from_iter(block_hashes.into_iter());
                self.known_block_hashes = known.union(&iter).cloned().collect();
                while self.known_block_hashes.len() < MAX_GETDATA_HASHES {
                    self.known_block_hashes.drain(0..self.known_block_hashes.len() / 3);
                }
            });
        }*/
        let (new_hashes, unique_hashes) = extract_new_and_unique(self.known_tx_hashes.clone(), tx_hashes);
        tx_hashes = new_hashes;
        self.known_tx_hashes = unique_hashes;

        //dispatch_async(self.delegateQueue, ^{
        // if self.status() == PeerStatus::Connected {
        //     self.transaction_delegate.peer_has_transaction_with_hash(self, tx_hashes);
        // }
        //});

        if !is_lock_hashes.is_empty() {
            let (new_hashes, unique_hashes) = extract_new_and_unique(self.known_is_lock_hashes.clone(), is_lock_hashes);
            is_lock_hashes = new_hashes;
            self.known_is_lock_hashes = unique_hashes;
            // todo
            //dispatch_async(self.delegateQueue, ^{
            // if self.status() == PeerStatus::Connected {
            //     self.transaction_delegate.peer_has_instant_send_lock_hashes(self, is_lock_hashes);
            // }
            //});
        }

        if !isd_lock_hashes.is_empty() {
            let (new_hashes, unique_hashes) = extract_new_and_unique(self.known_isd_lock_hashes.clone(), isd_lock_hashes);
            isd_lock_hashes = new_hashes;
            self.known_isd_lock_hashes = unique_hashes;
            // todo
            //dispatch_async(self.delegateQueue, ^{
            // if self.status() == PeerStatus::Connected {
            //     self.transaction_delegate.peer_has_instant_send_deterministic_lock_hashes(self, isd_lock_hashes);
            // }
            //});
        }

        if !c_lock_hashes.is_empty() {
            let (new_hashes, unique_hashes) = extract_new_and_unique(self.known_chain_lock_hashes.clone(), c_lock_hashes);
            c_lock_hashes = new_hashes;
            // todo
            //dispatch_async(self.delegateQueue, ^{
            // if self.status() == PeerStatus::Connected {
            //     self.transaction_delegate.peer_has_chain_lock_hashes(self, c_lock_hashes);
            // }
            //});
            self.known_chain_lock_hashes = unique_hashes;
        }

        if tx_hashes.len() + is_lock_hashes.len() + isd_lock_hashes.len() > 0 ||
            (!self.needs_filter_update && (block_hashes.len() + c_lock_hashes.len() > 0)) {
            self.send_getdata_message_with_tx_hashes(
                Some(tx_hashes.clone()),
                Some(is_lock_hashes.clone()),
                Some(isd_lock_hashes.clone()),
                if self.needs_filter_update { None } else { Some(block_hashes.clone()) },
                Some(c_lock_hashes.clone()),
            );
        }

        // to improve chain download performance, if we received 500 block hashes, we request the next 500 block hashes
        /*if !self.needs_filter_update {
            self.chain.with(|mut chain| {
                if block_hashes.len() >= 500 {
                    let locators = vec![block_hashes[block_hashes.len() - 1], block_hashes[0]];
                    if chain.should_request_merkle_blocks_for_next_sync_block_height() {
                        self.send_getblocks_message_with_locators(locators, UInt256::MIN);
                    } else {
                        self.send_getheaders_message_with_locators(locators, UInt256::MIN);
                    }
                } else if block_hashes.len() >= 2 && chain.sync_phase == SyncPhase::ChainSync {
                    let last_terminal_block_hash = chain.last_terminal_block.as_ref().unwrap().upgrade().unwrap().block_hash();
                    if block_hashes.iter().find(|block_hash| last_terminal_block_hash.eq(block_hash)).is_none() {
                        // we did not find the last hash, lets ask the remote again for blocks as a race condition might have occured
                        self.send_getblocks_message_with_locators(vec![block_hashes[block_hashes.len() - 1], block_hashes[0]], UInt256::MIN);
                    }
                } else if block_hashes.len() == 1 && chain.sync_phase == SyncPhase::ChainSync {
                    // this could either be a terminal block, or very rarely (1 in 500) the race condition dealt with above but block hashes being 1
                    // First we ust find if the blockHash is a terminal block hash
                    let found_in_terminal_blocks = chain.terminal_blocks.get(block_hashes.first().unwrap()).is_some();
                    let is_last_terminal_block = chain.last_terminal_block.as_ref().unwrap().upgrade().unwrap().block_hash().eq(block_hashes.first().unwrap());
                    if found_in_terminal_blocks && !is_last_terminal_block {
                        self.send_getblocks_message_with_locators(vec![block_hashes[block_hashes.len() - 1], block_hashes[0]], UInt256::MIN);
                    }
                }
            })
        }*/
        if self.mempool_transaction_callback.is_some() && (tx_hashes.len() + governance_object_hashes.len() + spork_hashes.len() > 0) {
            // this will cancel the mempool timeout
            self.mempool_request_time = SystemTime::seconds_since_1970();
            println!("[DSPeer] got mempool tx inv messages {}", self.host());
            let completion = self.mempool_transaction_callback.as_ref().unwrap();
            // self.send_ping_message(Arc::new(|success| completion(success, true, false)));
            self.mempool_transaction_callback = None;
        }

        if !governance_object_hashes.is_empty() {
            self.chain.peer_has_governance_object_hashes(self, governance_object_hashes);
        }
        if !governance_object_vote_hashes.is_empty() {
            self.chain.peer_has_governance_vote_hashes(self, governance_object_vote_hashes);
        }
        if !spork_hashes.is_empty() {
            self.chain.peer_has_spork_hashes(self, spork_hashes);
        }
    }

    pub fn accept_tx_message(&mut self, message: &[u8]) {

        // let tx = tx::Factory::transaction_with_message(message, ReadContext(self.chain_type, self.chain.clone()));

        let tx = tx::Factory::transaction_with_message(message, ReadContext(self.chain_type, self.chain.clone()));
        if tx.is_none() && !tx::Factory::should_ignore_transaction_message(message) {
            self.disconnect_with_error(Some(Error::Default(format!("malformed tx message: {:?}", message))));
            return;
        } else if !self.sent_filter && !self.sent_getdatatxblocks {
            self.disconnect_with_error(Some(Error::Default(format!("got tx message before loading a filter"))));
            return;
        }

        if let Some(ref transaction) = tx {
            // let current_block = self.current_block;
            //dispatch_async(self.delegateQueue, ^{
            if let Some(c_block) = self.current_block.take() {
                self.chain.peer_relayed_transaction(self, transaction, &c_block);
                // self.chain.upgrade().unwrap().peer_relayed_transaction(self, transaction, &c_block);
            }
            //});
        }

        if let Some(c_block) = self.current_block.take() {
            // we're collecting tx messages for a merkleblock
            let tx_hash = tx.map_or(UInt256::sha256d(message), |tx| tx.tx_hash());
            if let Some(block_tx_hashes) = self.current_block_tx_hashes.as_mut() {
                block_tx_hashes.retain(|x| x != &tx_hash);
                if block_tx_hashes.is_empty() {
                    // we received the entire block including all matched tx
                    println!("{} clearing current block", self.socket_addr);
                    self.current_block = None;
                    self.current_block_tx_hashes = None;
                    //dispatch_sync(self.delegateQueue, ^{ // syncronous dispatch so we don't get too many queued up tx
                    self.chain.peer_relayed_block(self, &c_block);
                    //});
                }
            }
        } else {
            println!("{} no current block", self.socket_addr);
        }
    }

    fn accept_islock_message(&mut self, message: &[u8], deterministic: bool) {
        if !self.chain.read(|chain| chain.spork_manager.deterministic_masternode_list_enabled) {
            println!("instant send lock (deterministic: {}) message when DML not enabled: {:?}", deterministic, message);
            return;
        } else if !self.chain.read(|chain| chain.spork_manager.llmq_instant_send_enabled) {
            println!("instant send lock (deterministic: {}) message when llmq instant send is not enabled: {:?}", deterministic, message);
            return;
        }
        match message.read_with::<InstantSendLock>(&mut 0, instant_send_lock::ReadContext { chain_type: self.chain_type, chain: self.chain.clone(), deterministic } ) {
            Ok(transaction_lock) if self.sent_filter || self.sent_getdatatxblocks =>
                self.chain.peer_relayed_instant_send_transaction_lock(self, transaction_lock),
            _ => self.disconnect_with_error(Some(Error::Default(format!("got islock (deterministic: {}) message before loading a filter or it's malformed", deterministic))))
        }
    }

    // HEADER FORMAT:
    // 01 ................................. Header count: 1
    //
    // 02000000 ........................... Block version: 2
    // b6ff0b1b1680a2862a30ca44d346d9e8
    // 910d334beb48ca0c0000000000000000 ... Hash of previous block's header
    // 9d10aa52ee949386ca9385695f04ede2
    // 70dda20810decd12bc9b048aaab31471 ... Merkle root
    // 24d95a54 ........................... Unix time: 1415239972
    // 30c31b18 ........................... Target (bits)
    // fe9f0864 ........................... Nonce
    //
    // 00 ................................. Transaction count (0x00)
    pub fn accept_headers_message(&mut self, message: &[u8]) {
        let mut offset = 0usize;
        let count = message.read_with::<VarInt>(&mut offset, byte::LE).unwrap();
        let num_headers = count.0;
        let expected_size = count.len() + 81 * num_headers as usize;
        if message.len() < expected_size {
            self.disconnect_with_error(Some(Error::Default(format!("malformed headers message, length is {}, should be {} for {} items", message.len(), expected_size, num_headers))));
            return;
        }
        if self.relay_start_time != 0 {
            // keep track of relay performance
            let speed = num_headers / (SystemTime::seconds_since_1970() - self.relay_start_time);

            if self.relay_speed == 0 {
                self.relay_speed = speed;
            }
            self.relay_speed = (self.relay_speed as f64 * 0.9 + speed as f64 * 0.1) as u64;
            self.relay_start_time = 0;
        }

        // To improve chain download performance, if this message contains 2000 headers then request the next 2000 headers
        // immediately, and switch to requesting blocks when we receive a header newer than earliestKeyTime
        // Devnets can run slower than usual
        // TODO: check reading offset
        let last_timestamp = message.read_with::<u32>(&mut offset, byte::LE).unwrap() as u64;
        let first_timestamp = message.read_with::<u32>(&mut offset, byte::LE).unwrap() as u64;
        // NSTimeInterval lastTimestamp = [message UInt32AtOffset:l + 81 * (count - 1) + 68];
        // NSTimeInterval firstTimestamp = [message UInt32AtOffset:l + 81 + 68];
        if (first_timestamp + DAY_TIME_INTERVAL * 2 >= self.earliest_key_time) &&
            self.chain.with(|chain| chain.should_request_merkle_blocks_for_zone_after_last_sync_height()) {
            // this is a rare scenario where we called getheaders but the first header returned was actually past the cuttoff, but the previous header was before the cuttoff
            // println!("{}:{} calling getblocks with locators: {:?}", self.host(), self.port, self.chain.chain_sync_block_locator_array());
            self.send_getblocks_message_with_locators(self.chain.with(|chain| chain.chain_sync_block_locator_array()), UInt256::MIN);
            return;
        }
        if num_headers == 0 {
            return;
        }
        if num_headers >= self.chain_type.header_max_amount() ||
            (((last_timestamp + DAY_TIME_INTERVAL * 2) >= self.earliest_key_time) &&
                (!self.chain.with(|chain| chain.needs_initial_terminal_headers_sync()))) {
            let mut offset = offset.clone();
            let first_block_hash = UInt256::x11_hash(message.read_with(&mut offset, Bytes::Len(80)).unwrap());
            let last_offset = &mut (offset + 81 * (num_headers as usize - 1));
            let mut last_block_hash = UInt256::x11_hash(message.read_with(last_offset, Bytes::Len(80)).unwrap());
            if last_timestamp + DAY_TIME_INTERVAL * 2 >= self.earliest_key_time &&
                self.chain.with(|chain| chain.should_request_merkle_blocks_for_zone_after_last_sync_height()) {
                // request blocks for the remainder of the chain
                //NSTimeInterval timestamp = [message UInt32AtOffset:l + 81 + 68];
                // for (off = l; timestamp > 0 && ((timestamp + DAY_TIME_INTERVAL * 2) < self.earliestKeyTime);) {
                //     off += 81;
                //     timestamp = [message UInt32AtOffset:off + 81 + 68];
                // }
                let timestamp = loop {
                    let ts = message.read_with::<u32>(&mut offset, byte::LE).unwrap() as u64;
                    if ts == 0 || ts + DAY_TIME_INTERVAL * 2 >= self.earliest_key_time {
                        break ts;
                    }
                };

                last_block_hash = UInt256::x11_hash(message.read_with(&mut offset, Bytes::Len(80)).unwrap());
                println!("{} calling getblocks with locators: [{}, {}]", self.socket_addr, last_block_hash, first_block_hash);
                self.send_getblocks_message_with_locators(vec![last_block_hash, first_block_hash], UInt256::MIN);
            } else {
                println!("{} calling getheaders with locators: [{}, {}]", self.socket_addr, last_block_hash, first_block_hash);
                self.send_getheaders_message_with_locators(vec![last_block_hash, first_block_hash], UInt256::MIN);
            }
        }
        // let mut iter: Iter<&[u8], _> = message.read_iter(&mut offset, Bytes::Len(81));
        // while let Some(data) = iter.next() {
        //     let block = data.read_with::<MerkleBlock>(&mut offset, merkle_block::ReadContext(self.chain_type, self.chain.borrow())).unwrap();
        //     if !block.is_valid() {
        //         self.disconnect_with_error(Some(Error::Default(format!("invalid block header {}", block.block_hash()))));
        //         return;
        //     }
        //     //dispatch_async(self.delegateQueue, ^{
        //     self.chain.peer_relayed_header(self, &block);
        //     //});
        // }
    }

    pub fn accept_getaddr_message(&self, _message: &[u8]) {
        println!("{} got getaddr", self.socket_addr);
        self.send_request(Request::Addr);
    }

    pub fn accept_getdata_message(&mut self, message: &[u8]) {
        let mut offset = 0usize;
        let count = message.read_with::<VarInt>(&mut offset, byte::LE).unwrap();
        let l = count.len();
        let size = count.0;
        if l == 0 || message.len() < l + size as usize * 36 {
            self.disconnect_with_error(Some(Error::Default(format!("malformed getdata message, length is {}, should be {} for {} items", message.len(), if l == 0 { 1 } else { l } + size as usize * 36, size as usize))));
            return;
        } else if size > MAX_GETDATA_HASHES as u64 {
            println!("{} dropping getdata message, {} is too many items, max is {}", self.socket_addr, size, MAX_GETDATA_HASHES);
            return;
        }
        //dispatch_async(self.delegateQueue, ^{
        let mut writer = Vec::<u8>::new();
        let mut iter: Iter<InvHash, _> = message.read_iter(&mut offset, byte::LE);
        while let Some(inv_hash) = iter.next() {
            if inv_hash.hash.is_zero() {
                continue;
            }
            match inv_hash.r#type {
                InvType::Tx |
                InvType::TxLockRequest => {
                    if let Some(transaction) = self.chain.peer_requested_transaction(self, &inv_hash.hash) {
                        self.send_message(transaction.to_data(), MessageType::Tx);
                    } else {
                        inv_hash.enc(&mut writer);
                    }
                },
                InvType::GovernanceObjectVote => {
                    if let Some(vote) = self.chain.peer_requested_vote(self, &inv_hash.hash) {
                        self.send_message(vote.data_message(), MessageType::Govobjvote);
                    } else {
                        inv_hash.enc(&mut writer);
                    }
                },
                InvType::GovernanceObject => {
                    if let Some(object) = self.chain.peer_requested_object(self, &inv_hash.hash) {
                        self.send_message(object.data_message(), MessageType::Govobj);
                    } else {
                        inv_hash.enc(&mut writer);
                    }
                },
                _ => {
                    inv_hash.enc(&mut writer);
                }

            }
        }
        if !writer.is_empty() {
            self.send_request(Request::NotFound(writer));
        }
        //});
    }

    pub fn accept_notfound_message(&mut self, message: &[u8]) {
        match message.read_with::<NotFound>(&mut 0, self.socket_addr) {
            Ok(NotFound { tx_hashes, block_hashes, .. }) =>
                self.chain.peer_relayed_not_found_messages_with_transaction_hashes(self, tx_hashes, block_hashes),
            Err(err) =>
                self.disconnect_with_error(Some(Error::Default(format!("{:?}", err))))
        }
    }

    pub fn accept_ping_message(&mut self, message: &[u8]) {
        if message.len() < std::mem::size_of::<u64>() {
            self.disconnect_with_error(Some(Error::Default(format!("malformed ping message, length is {}, should be 4", message.len()))));
        } else {
            self.send_message(message.to_vec(), MessageType::Pong);
        }
    }

    pub fn accept_pong_message(&mut self, message: &[u8]) {
        if message.len() < std::mem::size_of::<u64>() {
            self.disconnect_with_error(Some(Error::Default(format!("malformed pong message, length is {}, should be 4", message.len()))));
            return;
        } else if message.read_with::<u64>(&mut 0, byte::LE).unwrap() != self.local_nonce {
            self.disconnect_with_error(Some(Error::Default(format!("pong message contained wrong nonce: {}, expected: {}", message.read_with::<u64>(&mut 0, byte::LE).unwrap(), self.local_nonce))));
            return;
        } else if self.pong_handlers.is_empty() {
            println!("{} got unexpected pong", self.socket_addr);
            return;
        }
        if self.ping_start_time > 1 {
            // 50% low pass filter on current ping time
            self.ping_time = (0.5 * (self.ping_time + SystemTime::seconds_since_1970() - self.ping_start_time) as f64) as u64;
            self.ping_start_time = 0;
        }
        //dispatch_async(self.delegateQueue, ^{
        // if self.status() == PeerStatus::Connected && !self.pong_handlers.is_empty() {
        //     if let Some(handler) = self.pong_handlers.first() {
        //         handler(true);
        //         self.pong_handlers.remove(0);
        //     }
        // }
        //});
    }

    // Dash nodes don't support querying arbitrary transactions, only transactions not yet accepted in a block.
    // After a merkleblock message, the remote node is expected to send tx messages for the tx referenced in the block.
    // When a non-tx message is received we should have all the tx in the merkleblock.
    pub fn accept_merkleblock_message(&mut self, message: &[u8]) {
        if let Ok(block) = message.read_with::<MerkleBlock>(&mut 0, merkle_block::ReadContext(self.chain_type, self.chain.clone())) {
            if !block.is_valid() {
                self.disconnect_with_error(Some(Error::Default(format!("invalid merkleblock: {}", block.block_hash()))));
                return;
            } else if !self.sent_filter && !self.sent_getdatatxblocks {
                self.disconnect_with_error(Some(Error::Default(format!("got merkleblock message before loading a filter"))));
                return;
            }
            let new_block_tx_hashes = block.transaction_hashes()
                .into_iter()
                .collect::<HashSet<_>>()
                .difference(&self.known_tx_hashes.iter()
                    .cloned()
                    .collect::<HashSet<_>>())
                .cloned()
                .collect::<Vec<_>>();
            if !new_block_tx_hashes.is_empty() {
                // wait til we get all the tx messages before processing the block
                self.current_block = Some(block);
                self.current_block_tx_hashes = Some(new_block_tx_hashes);
            } else {
                //dispatch_async(self.delegateQueue, ^{
                self.chain.peer_relayed_block(self, &block);
                //});
            }
        }
    }

    /// DIP08: https://github.com/dashpay/dips/blob/master/dip-0008.md
    pub fn accept_chain_lock_message(&mut self, message: &[u8]) {
        if !self.chain.read(|chain| chain.spork_manager.chain_locks_enabled) {
            return;
        }
        match message.read_with::<ChainLock>(&mut 0, chain_lock::ReadContext(self.chain_type, self.chain.clone())) {
            Ok(chain_lock) if self.sent_filter || self.sent_getdatatxblocks =>
                self.chain.peer_relayed_chain_lock(self, chain_lock),
            _ => self.disconnect_with_error(Some(Error::Default(format!("got chain lock message before loading a filter or malformed: {:?}", message))))
        }
    }

    pub fn accept_reject_message(&mut self, message: &[u8]) {
        if let Ok(Reject { hash: Some(tx_hash), code, .. }) = message.read_with::<Reject>(&mut 0, ()) {
            self.chain.peer_rejected_transaction(self, tx_hash, code);
        }
    }

    /// BIP133: https://github.com/bitcoin/bips/blob/master/bip-0133.mediawiki
    pub fn accept_fee_filter_message(&mut self, message: &[u8]) {
        match message.read_with::<u64>(&mut 0, byte::LE) {
            Ok(fee) => {
                self.fee_per_byte = fee / 1000;
                self.chain.peer_set_fee_per_byte(self, self.fee_per_byte);
            },
            Err(err) => self.disconnect_with_error(Some(Error::Default(format!("malformed freerate message, length is {}, should be 4", message.len()))))
        }
    }

    pub fn accept_spork_message(&mut self, message: &[u8]) {
        let updated_signatures = self.chain.with(|chain| chain.spork_manager.sporks_updated_signatures());
        let spork = message.read_with::<Spork>(&mut 0, spork::spork::ReadContext(self.chain_type, updated_signatures)).unwrap();
        println!("received spork {:?} with message {}", spork.identifier, hex_with_data(message));
        // self.chain.peer_relayed_spork(self, spork);
    }

    pub fn accept_ssc_message(&mut self, message: &[u8]) {
        let sync_count_info = message.read_with::<SyncCountInfo>(&mut 0, byte::LE).unwrap();
        let count = message.read_with::<u32>(&mut 4, byte::LE).unwrap();
        println!("received ssc message {:?} {}", sync_count_info, count);
        match (sync_count_info, self.governance_request_state) {
            (SyncCountInfo::GovernanceObject, GovernanceRequestState::GovernanceObjectHashes) => {
                self.governance_request_state = GovernanceRequestState::GovernanceObjectHashesCountReceived;
                self.chain.peer_relayed_sync_info(self, &sync_count_info, count);
            },
            (SyncCountInfo::GovernanceObject, GovernanceRequestState::GovernanceObjectHashesReceived) => {
                self.governance_request_state = GovernanceRequestState::GovernanceObjects;
                self.chain.peer_relayed_sync_info(self, &sync_count_info, count);
            },
            (SyncCountInfo::GovernanceObjectVote, GovernanceRequestState::GovernanceObjectVoteHashes) => {
                self.governance_request_state = GovernanceRequestState::GovernanceObjectVoteHashesCountReceived;
                self.chain.peer_relayed_sync_info(self, &sync_count_info, count);
            },
            (SyncCountInfo::GovernanceObjectVote, GovernanceRequestState::GovernanceObjectVoteHashesReceived) => {
                self.governance_request_state = GovernanceRequestState::GovernanceObjectVotes;
                self.chain.peer_relayed_sync_info(self, &sync_count_info, count);
            },
            (SyncCountInfo::MNW, _) |
            (SyncCountInfo::List, _) => {
                self.chain.peer_relayed_sync_info(self, &sync_count_info, count);
            },
            _ => {}
        }
        //ignore when count = 0; (for votes)
    }

    pub fn accept_mnlistdiff_message(&self, message: &[u8]) {
        self.chain.peer_relayed_masternode_diff_message(self, message);
    }

    pub fn accept_qrinfo_message(&self, message: &[u8]) {
        self.chain.peer_relayed_quorum_rotation_info_message(self, message);
    }

    // accept Governance
    // https://dash-docs.github.io/en/developer-reference#govobj

    pub fn accept_gov_object_message(&self, message: &[u8]) {
        // if let Some(object) = governance::Object::init_with_message(message, self.chain) {
        //     self.governance_delegate.peer_relayed_governance_object(self, object);
        // }
    }

    pub fn accept_gov_object_vote_message(&self, message: &[u8]) {
        // if let Some(vote) = governance::Vote::init_with_message(message, self.chain) {
        //     self.governance_delegate.peer_relayed_governance_vote(self, vote);
        // }
    }

    /// FNV32-1a hash of the ip address and port number: http://www.isthe.com/chongo/tech/comp/fnv/index.html#FNV-1a
    pub fn hash(&self) -> u32 {
        let mut hash = FNV32_OFFSET;
        let ip: UInt128 = self.socket_addr.ip().into();
        (0..std::mem::size_of::<UInt128>()).for_each(|i| {
            hash = hash ^ (ip.0[i] as u32) * FNV32_PRIME;
            // hash ^= self.address.0[i] * FNV32_PRIME;
        });
        hash = (hash ^ ((self.socket_addr.port() >> 8) & 0xff) as u32) * FNV32_PRIME;
        hash = (hash ^ (self.socket_addr.port() & 0xff) as u32) * FNV32_PRIME;
        hash
    }

    pub fn chain_tip(&self) -> String {
        if let Some(b) = &self.current_block {
            short_hex_string_from(b.block_hash().as_bytes())
        } else {
            "".to_string()
        }
    }

    pub fn save(&self) {
        todo!("PeerEntity save")
    }



    pub fn connect(&mut self) {
        if self.status() != PeerStatus::Disconnected { return; }
        self.status = PeerStatus::Connecting;
        self.ping_time = u64::MAX;

        // if !self.reachability.is_none() {
        //     self.reachability = Some(ReachabilityManager::new());
        // }
        // let mut reachability = self.reachability.unwrap();
        //
        // if reachability.last_status() == Status::NotReachable {
        //     println!("{} not reachable, waiting...", self.socket_addr);
        //
        //     reachability.add_handler(|status| {
        //         if status != Status::NotReachable {
        //             self.status = PeerStatus::Disconnected;
        //             self.connect();
        //         }
        //     });
        //     if !reachability.is_running {
        //         reachability.start_monitoring();
        //     }
        //     return;
        // }
        self.reset();

        thread::Builder::new()
            .name(format!("peer.{}", self.socket_addr))
            .spawn(move || {


            })
            .expect("Can't spawn peer thread");


        // thread::spawn(move || {
        //     println!("Connecting... {}", self.socket_addr);
            // match TcpStream::connect(self.socket_addr) {
            //     Ok(stream) => {
            //         // let mut stream_clone = stream.try_clone()?;
            //         let mut buffer = [0; 1024];
            //
            //     },
            //     Err(_) => {},
            // }
            // Setup the server socket.
            // let addr = "127.0.0.1:13265".parse()?;
            // self.connect_loop()
            //     .expect("Can't setup connection loop");
        // });
    }

/*    fn connect_loop(&mut self) -> io::Result<()> {
        let mut server = TcpListener::bind(self.socket_addr)?;
        // Start listening for incoming connections.
        self.poll.registry()
            .register(&mut server, SERVER, Interest::READABLE)?;

        // Setup the client socket.
        let mut client = TcpStream::connect(self.socket_addr)?;
        // Register the socket.
        self.poll.registry()
            .register(&mut client, CLIENT, Interest::READABLE | Interest::WRITABLE)?;

        loop {
            self.poll.poll(&mut self.events, None)?;
            for event in self.events.iter() {
                // We can use the token we previously provided to `register` to
                // determine for which socket the event is.
                match event.token() {
                    SERVER => {
                        // If this is an event for the server, it means a connection
                        // is ready to be accepted.
                        // Accept the connection and drop it immediately. This will
                        // close the socket and notify the client of the EOF.
                        let connection = server.accept();
                        drop(connection);
                    }
                    CLIENT => {
                        if event.is_writable() {
                            // We can (likely) write to the socket without blocking.
                        }

                        if event.is_readable() {
                            // We can (likely) read from the socket without blocking.
                        }

                        // Since the server just shuts down the connection, let's
                        // just exit from our event loop.
                        return Ok(());
                    }
                    // We don't expect any events with tokens other than those we provided.
                    _ => unreachable!(),
                }
            }
        }
    }
*/
    pub fn disconnect(&mut self) {
        self.disconnect_with_error(None);
    }

    pub fn disconnect_with_error(&mut self, error: Option<Error>) {
        if self.status() == PeerStatus::Disconnected {
            return;
        }
        if let Some(err) = error {
            println!("Disconnected from peer {} ({:?}) with error {}", self.host(), self.version, err);
        } else {
            println!("Disconnected from peer {} ({:?}) with no error", self.host(), self.version);
        }

        //[NSObject cancelPreviousPerformRequestsWithTarget:self]; // cancel connect timeout

        self.status = PeerStatus::Disconnected;
        todo!()
    }

    pub fn did_connect(&mut self) {
        if self.status() != PeerStatus::Connecting || !self.sent_verack || !self.got_verack {
            return;
        }

        // println!("{}:{} handshake completed {}", self.host(), self.port, (self.peerDelegate.downloadPeer == self) ? @"(download peer)" : @"");
        //[NSObject cancelPreviousPerformRequestsWithTarget:self]; // cancel pending handshake timeout
        self.status = PeerStatus::Connected;
        // self.delegate_context.queue(|| {
        //     if self.status() == PeerStatus::Connected {
        //         self.per
        //     }
        // });
        // dispatch_async(self.delegateQueue, ^{
        //     if (self->_status == DSPeerStatus_Connected) [self.peerDelegate peerConnected:self];
        // });
        todo!()
    }

    pub fn received_orphan_block(&mut self) {
        self.received_orphan_count += 1;
        if self.received_orphan_count > 9 {
            //after 10 orphans mark this peer as bad by saying we got a bad block
            self.chain.peer_relayed_too_many_orphan_blocks(self, self.received_orphan_count as usize);
        }

    }
}

// Conversion Sqlite
/*impl Peer {

    pub fn update_values(&self) -> Box<dyn EntityUpdates<bool, ResultType = (bool, )>> {
        Box::new((
            peers::timestamp.eq(NaiveDateTime::from_timestamp_opt(self.timestamp as i64, 0)),
            peers::services.eq(self.services as u64),
            peers::misbehaving.eq(self.misbehaving),
            peers::priority.eq(self.priority as i32),
            peers::low_preference_till.eq(self.low_preference_till as i64),
            peers::last_requested_masternode_list.eq(self.last_requested_masternode_list),
            peers::last_requested_governance_sync.eq(self.last_requested_governance_sync),
        ))
    }

    pub fn create_entity(&self, chain_id: i32) -> NewPeerEntity {
        //TODO: store IPv6 addresses
        // if (self.address.u64[0] != 0 || peer.address.u32[2] != CFSwapInt32HostToBig(0xffff)) return nil;
        NewPeerEntity {
            address: self.address.ip_address_to_i32(),
            port: self.port as i16,
            misbehaving: self.misbehaving,
            priority: self.priority as i32,
            services: self.services as i64,
            timestamp: NaiveDateTime::from_timestamp_opt(self.timestamp as i64, 0).unwrap(),
            last_requested_governance_sync:
            if let Some(timestamp) = self.last_requested_governance_sync {
                NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap()
            } else {
                None
            },
            last_requested_masternode_list:
            if let Some(timestamp) = self.last_requested_governance_sync {
                NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap()
            } else {
                None
            },
            low_preference_till: NaiveDateTime::from_timestamp_opt(self.low_preference_till as i64, 0).unwrap(),
            chain_id
        }
    }

}
*/
