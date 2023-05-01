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

/// A peer's Id
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct PeerId {
    // network: &'static str,
    chain_type: ChainType,
    // mio token used in networking
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

/// a peer
pub struct Peer {
    /// the peer's id for log messages
    pub pid: PeerId,
    // the event poller, shared with P2P, needed here to register for events
    poll: Arc<Poll>,
    // the connection to remote peer
    pub(crate) stream: TcpStream,
    // temporary buffer for not yet completely read incoming messages
    pub(crate) read_buffer: Buffer,
    // temporary buffer for not yet completely written outgoing messages
    pub(crate) write_buffer: Buffer,
    // did the remote peer already sent a verack?
    // pub(crate) got_verack: bool,
    // did the local peer already sent a getaddr?
    // sent_getaddr: bool,
    pub flags: PeerStateFlags,
    /// the version message the peer sent to us at connect
    pub version: Option<VersionCarrier>,
    // channel into the event processing loop for outgoing messages
    sender: mpsc::Sender<Message>,
    // channel into the event processing loop for outgoing messages
    receiver: mpsc::Receiver<Message>,
    // is registered for write?
    writeable: AtomicBool,
    // connected and handshake complete?
    pub(crate) connected: bool,
    // ban score
    pub(crate) ban: u32,
    // outgoing or incoming connection
    pub(crate) outgoing: bool
}

impl Peer {
    /// create a new peer
    pub fn new(pid: PeerId, stream: TcpStream, poll: Arc<Poll>, outgoing: bool) -> Result<Peer, peer_manager::Error> {
        let (sender, receiver) = mpsc::channel();
        let peer = Peer { pid, poll: poll.clone(), stream, read_buffer: Buffer::new(), write_buffer: Buffer::new(),
            flags: PeerStateFlags::EMPTY, version: None, sender, receiver, writeable: AtomicBool::new(false),
            connected: false, ban: 0, outgoing };
        Ok(peer)
    }

    // re-register for peer readable events
    pub(crate) fn reregister_read(&mut self) -> Result<(), peer_manager::Error> {
        if self.writeable.swap(false, Ordering::Acquire) {
            println!("re-register for read peer={}", self.pid);
            self.poll.registry()
                .reregister(&mut self.stream, self.pid.token, Interest::READABLE)
                .expect("Can't reregister stream");
        }
        Ok(())
    }

    // register for peer readable events
    pub(crate) fn register_read(&mut self) -> Result<(), peer_manager::Error> {
        println!("register for read peer={}", self.pid);
        self.poll.registry()
            .register(&mut self.stream, self.pid.token, Interest::READABLE)
            .expect("Can't register stream");
        self.writeable.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// send a message to P2P network
    pub fn send(&mut self, msg: Message) -> Result<(), peer_manager::Error> {
        // send to outgoing message channel
        self.sender.send(msg).map_err(| _ | peer_manager::Error::Downstream("can not send to peer queue".to_owned()))?;
        // register for writable peer events since we have outgoing message
        self.reregister_write()?;
        Ok(())
    }

    // register for peer writable events
    fn reregister_write(&mut self) -> Result<(), peer_manager::Error> {
        if !self.writeable.swap(true, Ordering::Acquire) {
            println!("re-register for write peer={}", self.pid);
            self.poll.registry()
                .reregister(&mut self.stream, self.pid.token, Interest::WRITABLE)?;
        }
        Ok(())
    }

    // register for peer writable events
    pub(crate) fn register_write(&mut self) -> Result<(), peer_manager::Error> {
        println!("register for write peer={}", self.pid);
        self.poll.registry()
            .register(&mut self.stream, self.pid.token, Interest::WRITABLE)?;
        self.writeable.store(true, Ordering::Relaxed);
        Ok(())
    }


    // try to receive a message from the outgoing message channel
    pub(crate) fn try_receive(&self) -> Option<Message> {
        self.receiver.try_recv().ok()
    }
}
