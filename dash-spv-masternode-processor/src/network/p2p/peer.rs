use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, mpsc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TryRecvError;
use mio::{Interest, Poll, Token};
use mio::net::TcpStream;
use crate::chain::common::ChainType;
use crate::chain::network::message::message::Message;
use crate::manager::peer_manager;
use crate::network::p2p::{Buffer, PeerStateFlags, VersionCarrier};

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
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
    poll: Arc<RwLock<Poll>>,
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
    pub fn new(pid: PeerId, stream: TcpStream, poll: Arc<RwLock<Poll>>, outgoing: bool) -> Result<Peer, peer_manager::Error> {
        let (sender, receiver) = mpsc::channel();
        let peer = Self { pid, poll, stream, read_buffer: Buffer::new(), write_buffer: Buffer::new(),
            flags: PeerStateFlags::EMPTY, version: None, sender, receiver, writeable: AtomicBool::new(false),
            connected: false, ban: 0, outgoing };
        Ok(peer)
    }

    pub fn send(&mut self, msg: Message) -> Result<(), peer_manager::Error> {
        debug!("send: {:?}", msg);
        self.sender.send(msg)
            .map_err(|_| peer_manager::Error::Downstream("can not send to peer queue".to_owned()))?;
        self.reregister_write()?;
        Ok(())
    }

    pub(crate) fn reregister_read(&mut self) -> Result<(), peer_manager::Error> {
        debug!("re-register_read.1:");
        if self.writeable.swap(false, Ordering::Acquire) {
            debug!("re-register_read.2:");
            if let Ok(poll) = self.poll.read() {
                debug!("re-register_read.3:");
                poll.registry()
                    .reregister(&mut self.stream, self.pid.token, Interest::READABLE)?;
            }
        }
        debug!("re-register_read.4:");
        Ok(())
    }

    pub(crate) fn register_read(&mut self) -> Result<(), peer_manager::Error> {
        debug!("register_read.1:");
        if let Ok(poll) = self.poll.read() {
            debug!("register_read.2:");
            poll.registry()
                .register(&mut self.stream, self.pid.token, Interest::READABLE)?
        }
        self.writeable.store(false, Ordering::Relaxed);
        debug!("register_read.3:");
        Ok(())
    }

    fn reregister_write(&mut self) -> Result<(), peer_manager::Error> {
        debug!("re-register_write.1:");
        if !self.writeable.swap(true, Ordering::Acquire) {
            debug!("re-register_write.2:");
            if let Ok(poll) = self.poll.read() {
                debug!("re-register_write.3:");
                poll.registry()
                    .reregister(&mut self.stream, self.pid.token, Interest::WRITABLE)?;
                debug!("re-register_write.4:");
            }
        }
        debug!("re-register_write.5:");
        Ok(())
    }

    pub(crate) fn register_write(&mut self) -> Result<(), peer_manager::Error> {
        debug!("register_write.1:");
        if let Ok(poll) = self.poll.read() {
            debug!("register_write.2:");
            poll.registry()
                .register(&mut self.stream, self.pid.token, Interest::WRITABLE)?;
            debug!("register_write.3:");
        }
        debug!("register_write.4:");
        self.writeable.store(true, Ordering::Relaxed);
        debug!("register_write.5:");
        Ok(())
    }

    pub(crate) fn try_receive(&self) -> Result<Message, TryRecvError> {
        self.receiver.try_recv()
    }
}
