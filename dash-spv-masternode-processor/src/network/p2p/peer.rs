use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use mio::{Interest, Poll, Token};
use mio::net::TcpStream;
use crate::chain::common::ChainType;
use crate::chain::network::message::message::Message;
use crate::manager::peer_manager;
use crate::network::p2p::buffer::Buffer;
use crate::network::p2p::layer::VersionCarrier;
use crate::network::p2p::state_flags::PeerStateFlags;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct PeerId {
    chain_type: ChainType,
    token: Token
}

impl PeerId {
    pub fn new(chain_type: ChainType, token: Token) -> Self {
        Self { chain_type, token }
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}-{}", self.chain_type, self.token.0)?;
        Ok(())
    }
}
pub type PeerMap = HashMap<PeerId, Mutex<Peer>>;

pub struct Peer {
    pub pid: PeerId,
    poll: Arc<Mutex<Poll>>,
    pub(crate) stream: TcpStream,
    pub(crate) read_buffer: Buffer,
    pub(crate) write_buffer: Buffer,
    pub flags: PeerStateFlags,
    pub version: Option<VersionCarrier>,
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    writeable: AtomicBool,
    pub(crate) connected: bool,
    pub(crate) ban: u32,
    pub(crate) outgoing: bool
}

impl Peer {
    pub fn new(pid: PeerId, stream: TcpStream, poll: Arc<Mutex<Poll>>, outgoing: bool) -> Result<Peer, peer_manager::Error> {
        let (sender, receiver) = mpsc::channel();
        let peer = Peer { pid, poll: poll.clone(), stream, read_buffer: Buffer::new(), write_buffer: Buffer::new(),
            flags: PeerStateFlags::EMPTY, version: None, sender, receiver, writeable: AtomicBool::new(false),
            connected: false, ban: 0, outgoing };
        Ok(peer)
    }

    pub(crate) fn reregister_read(&mut self) -> Result<(), peer_manager::Error> {
        if self.writeable.swap(false, Ordering::Acquire) {
            if let Ok(poll) = self.poll.lock() {
                poll.registry()
                    .reregister(&mut self.stream, self.pid.token, Interest::READABLE)
                    .expect("Can't reregister stream");
            }
        }
        Ok(())
    }

    pub(crate) fn register_read(&mut self) -> Result<(), peer_manager::Error> {
        if let Ok(poll) = self.poll.lock() {
            poll.registry()
                .register(&mut self.stream, self.pid.token, Interest::READABLE)
                .expect("Can't register stream");
        }
        self.writeable.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn send(&mut self, msg: Message) -> Result<(), peer_manager::Error> {
        self.sender.send(msg).map_err(|_| peer_manager::Error::Downstream("can not send to peer queue".to_owned()))?;
        self.reregister_write()?;
        Ok(())
    }

    fn reregister_write(&mut self) -> Result<(), peer_manager::Error> {
        if !self.writeable.swap(true, Ordering::Acquire) {
            if let Ok(poll) = self.poll.lock() {
                poll.registry().reregister(&mut self.stream, self.pid.token, Interest::WRITABLE)?;
            }
        }
        Ok(())
    }

    pub(crate) fn register_write(&mut self) -> Result<(), peer_manager::Error> {
        if let Ok(poll) = self.poll.lock() {
            poll.registry().register(&mut self.stream, self.pid.token, Interest::WRITABLE)?;
        }
        self.writeable.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub(crate) fn try_receive(&self) -> Option<Message> {
        self.receiver.try_recv().ok()
    }
}
