use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;
use crate::chain::block::Block;
use crate::chain::Chain;
use crate::chain::common::ChainType;
use crate::chain::network::message::message::{Direction, Message, MessagePayload};
use crate::chain::network::message::response::Response;
use crate::chain::network::{InvType, MessageType, Request};
use crate::consensus::encode;
use crate::crypto::UInt256;
use crate::manager::peer_manager;
use crate::manager::peer_manager::{SERVICES_NODE_BLOOM, SERVICES_NODE_NETWORK};
use crate::network::p2p::buffer::Buffer;
use crate::network::p2p::state_flags::PeerStateFlags;
use crate::util::{Shared, TimeUtil};

pub trait PeerState {
    fn chain(&self) -> Shared<Chain>;
    fn chain_type(&self) -> ChainType;
    fn version(&self, remote: SocketAddr, max_protocol_version: u32) -> Message;
    fn nonce(&self) -> u64;
    fn magic(&self) -> u32;
    fn user_agent(&self) -> &str;
    fn get_height(&self) -> u32;
    fn set_height(&self, height: u32);
    fn get_local_height(&self) -> u32;
    fn get_local_hash(&self) -> UInt256;
    fn known_block_hashes(&self) -> Vec<UInt256>;

    fn flags(&self) -> PeerStateFlags;
    fn verack(&self) -> Message;

    fn pack(&self, message: Message) -> Vec<u8>;
    fn unpack(&self, message: Message) -> Result<Response, peer_manager::Error>;

    fn encode<E: AsRef<[u8]>>(&self, item: E, dst: &mut Buffer) -> Result<(), io::Error>;
    fn decode(&self, src: &mut Buffer) -> Result<Option<Message>, io::Error>;
}

pub struct DashP2PState {
    pub chain: Shared<Chain>,
    pub chain_type: ChainType,
    // This node's identifier on the network (random)
    pub nonce: u64,
    // height of the blockchain tree trunk
    pub height: AtomicUsize,
    // current height of the SPV
    pub local_height: AtomicUsize,
    pub local_hash: Arc<UInt256>,
    // This node's human readable type identification
    pub user_agent: String,
    // // this node's maximum protocol version
    // pub max_protocol_version: u32,
    // serving others
    pub server: bool,

    flags: PeerStateFlags,
    inventory: HashMap<InvType, Vec<UInt256>>,
    // sent_getaddr: bool,
    // sent_filter: bool,
    // sent_mempool: bool,
    // sent_getblocks: bool,
}

impl DashP2PState {
    pub fn new(last_block: Option<Block>, chain_type: ChainType, chain: Shared<Chain>) -> Self {
        Self {
            chain,
            chain_type,
            nonce: 0,
            height: AtomicUsize::new(0),
            local_height: AtomicUsize::new(last_block.as_ref().map_or(0, |b| b.height as usize)),
            local_hash: Arc::new(last_block.as_ref().map_or(UInt256::MIN, |b| b.block_hash)),
            user_agent: chain_type.user_agent(),
            server: false,
            flags: Default::default(),
            inventory: Default::default(),
        }
    }
}


impl PeerState for DashP2PState {
    fn chain(&self) -> Shared<Chain> {
        self.chain.clone()
    }
    fn chain_type(&self) -> ChainType {
        self.chain_type
    }

    // compile this node's version message for outgoing connections
    fn version(&self, remote: SocketAddr, max_protocol_version: u32) -> Message {
        let timestamp = SystemTime::seconds_since_1970() as i64;
        let services = if self.server { SERVICES_NODE_NETWORK | SERVICES_NODE_BLOOM } else { 0 };
        Message {
            r#type: MessageType::Version,
            direction: Direction::Outgoing,
            payload: MessagePayload::Request(Request::Version(remote, services, self.nonce, self.chain_type))
        }
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn magic(&self) -> u32 {
        self.chain_type.magic()
    }

    fn user_agent(&self) -> &str {
        self.user_agent.as_str()
    }

    fn get_height(&self) -> u32 {
        self.height.load(Ordering::Relaxed) as u32
    }

    fn set_height(&self, height: u32) {
        self.height.store(height as usize, Ordering::Relaxed)
    }

    fn get_local_height(&self) -> u32 {
        self.local_height.load(Ordering::Relaxed) as u32
    }

    fn get_local_hash(&self) -> UInt256 {
        *self.local_hash
    }

    fn known_block_hashes(&self) -> Vec<UInt256> {
        // todo: cache it
        let mut hashes = vec![];
        if let Some(block_hashes) = self.inventory.get(&InvType::Block) {
            hashes.extend(block_hashes)
        }
        if let Some(block_hashes) = self.inventory.get(&InvType::Merkleblock) {
            hashes.extend(block_hashes)
        }
        hashes
    }

    fn flags(&self) -> PeerStateFlags {
        self.flags
    }

    fn verack(&self) -> Message {
        Message {
            r#type: MessageType::Verack,
            direction: Direction::Outgoing,
            payload: MessagePayload::Request(Request::Default(MessageType::Verack))
        }
    }

    fn pack(&self, message: Message) -> Vec<u8> {
        message.compile(self.chain_type.magic())
    }

    fn unpack(&self, message: Message) -> Result<Response, peer_manager::Error> {
        message.decompile(self)
    }

    fn encode<E: AsRef<[u8]>>(&self, item: E, dst: &mut Buffer) -> Result<(), io::Error> {
        dst.write_all(item.as_ref())
    }

    // Decode header & check payload (without parsing)
    fn decode(&self, buffer: &mut Buffer) -> Result<Option<Message>, io::Error> /*where P: Decodable*/ {
        match Message::from_buffer(buffer) {
            Ok(m) => {
                // success: free the read data in buffer and return the message
                buffer.commit();
                Ok(Some(m))
            }
            Err(encode::Error::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // need more data, rollback and retry after additional read
                buffer.rollback();
                Ok(None)
            },
            Err(encode::Error::Io(e)) => {
                println!("{:?}", e);
                buffer.commit();
                Err(e)
            },
            Err(e) => {
                println!("{:?}", e);
                buffer.commit();
                Err(io::Error::new(io::ErrorKind::InvalidData, e))
            }
        }
    }

}
