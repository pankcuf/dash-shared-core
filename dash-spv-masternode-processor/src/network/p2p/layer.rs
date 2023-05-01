//
// Copyright 2018-2019 Tamas Blummer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
//!
//! # P2P network communication
//!
//! This module establishes network connections and routes messages between the P2P network and this node
//!

use futures::{future, Future, FutureExt, TryFutureExt};
use futures::task::Poll as Async;
use futures::task::Waker;

use std::{
    cmp::min,
    collections::HashMap,
    io,
    io::{Read, Write},
    net::{Shutdown, SocketAddr},
    sync::{Arc, atomic::{AtomicUsize, Ordering}, mpsc, Mutex, RwLock},
    thread,
    time::{Duration, SystemTime}
};
use std::net::SocketAddrV4;
use futures::future::Either;
use futures_timer::Delay;
use mio::{Interest, Poll, Token};
use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use crate::chain::Chain;
use crate::chain::common::ChainType;
use crate::chain::network::message::message::{Message, Payload};
use crate::chain::network::message::response::Response;
use crate::chain::network::MessageType;
use crate::manager::peer_manager::Error;
use crate::network::p2p::peer::{Peer, PeerId, PeerMap};
use crate::network::p2p::state::PeerState;
use crate::network::p2p::state_flags::PeerStateFlags;
use crate::util::{Shared, TimeUtil};

pub const IO_BUFFER_SIZE:usize = 1024*1024;
const EVENT_BUFFER_SIZE:usize = 1024;
const CONNECT_TIMEOUT_SECONDS: u64 = 5;
const BAN :u32 = 100;


/// A message from network to downstream
#[derive(Clone)]
pub enum PeerNotification {
    Outgoing(Message),
    Incoming(PeerId, Response),
    Connected(PeerId, Option<SocketAddr>),
    Disconnected(PeerId, bool) // true if banned
}

/// a map of peer id to peers
pub type PeerNotificationReceiver = mpsc::Receiver<PeerNotification>;
pub type PeerNotificationSender = mpsc::SyncSender<PeerNotification>;

pub enum P2PControl {
    Send(PeerId, Message),
    Broadcast(Message),
    Ban(PeerId, u32),
    Disconnect(PeerId),
    Height(u32),
    Bind(SocketAddr)
}


type P2PControlReceiver = mpsc::Receiver<P2PControl>;

#[derive(Clone)]
pub struct P2PControlSender {
    sender: Arc<Mutex<mpsc::Sender<P2PControl>>>,
    peers: Arc<RwLock<PeerMap>>,
    pub back_pressure: usize
}

impl P2PControlSender {
    fn new(sender: mpsc::Sender<P2PControl>, peers: Arc<RwLock<PeerMap>>, back_pressure: usize) -> P2PControlSender {
        P2PControlSender { sender: Arc::new(Mutex::new(sender)), peers, back_pressure }
    }

    pub fn send(&self, control: P2PControl) {
        self.sender.lock()
            .unwrap()
            .send(control)
            .expect("P2P control send failed");
    }

    pub fn send_network(&self, peer: PeerId, msg: Message) {
        self.send(P2PControl::Send(peer, msg))
    }

    pub fn ban(&self, peer: PeerId, increment: u32) {
        println!("increase ban score with {} peer={}", increment, peer);
        self.send(P2PControl::Ban(peer, increment))
    }

    pub fn peer_version(&self, peer: PeerId) -> Option<VersionCarrier> {
        if let Some(peer) = self.peers.read().unwrap().get(&peer) {
            let locked_peer = peer.lock().unwrap();
            return locked_peer.version.clone();
        }
        None
    }

    pub fn peers(&self) -> Vec<PeerId> {
        self.peers.read().unwrap().keys().cloned().collect::<Vec<_>>()
    }
}

#[derive(Clone)]
pub enum PeerSource {
    Outgoing(SocketAddr),
    Incoming(Arc<TcpListener>)
}

#[derive(Clone)]
pub struct PeerNotificationDispatcher {
    sender: Arc<Mutex<PeerNotificationSender>>
}

impl PeerNotificationDispatcher {
    pub fn new(sender: PeerNotificationSender) -> PeerNotificationDispatcher {
        PeerNotificationDispatcher { sender: Arc::new(Mutex::new(sender)) }
    }

    pub fn send(&self, msg: PeerNotification) {
        self.sender.lock()
            .unwrap()
            .send(msg)
            .expect("P2P message send failed");
    }
}

#[derive(Clone)]
pub struct VersionCarrier {
    /// The P2P network protocol version
    pub version: u32,
    /// A bitmask describing the services supported by this node
    pub services: u64,
    /// The time at which the `version` message was sent
    pub timestamp: u64,
    /// The network address of the peer receiving the message
    pub receiver_address: SocketAddr,
    /// The network address of the peer sending the message
    pub sender_address: SocketAddr,
    /// A random nonce used to detect loops in the network
    pub nonce: u64,
    /// A string describing the peer's software
    pub user_agent: String,
    /// The height of the maximum-work blockchain that the peer is aware of
    pub start_height: u32,
    /// Whether the receiving peer should relay messages to the sender; used
    /// if the sender is bandwidth-limited and would like to support bloom
    /// filtering. Defaults to true.
    pub relay: bool
}


/// The P2P network layer
pub struct P2P<STATE: PeerState + Send + Sync + 'static> {
    // sender to the dispatcher of incoming messages
    dispatcher: PeerNotificationDispatcher,
    // network specific conf
    pub state: STATE,
    pub chain_type: ChainType,
    pub chain: Shared<Chain>,
    // The collection of connected peers
    peers: Arc<RwLock<PeerMap>>,
    // The poll object of the async IO layer (mio)
    // access to this is shared by P2P and Peer
    poll: Arc<Poll>,
    // next peer id
    // atomic only for interior mutability
    next_peer_id: AtomicUsize,
    // waker
    waker: Arc<Mutex<HashMap<PeerId, Waker>>>,
    // server
    listener: Arc<Mutex<HashMap<Token, Arc<TcpListener>>>>,
}

impl<STATE: PeerState + Send + Sync> P2P<STATE> {
    /// create a new P2P network controller
    pub fn new(config: STATE, dispatcher: PeerNotificationDispatcher, back_pressure: usize, chain_type: ChainType, chain: Shared<Chain>) -> (Arc<P2P<STATE>>, P2PControlSender) {
        let (control_sender, control_receiver) = mpsc::channel();
        let peers = Arc::new(RwLock::new(PeerMap::new()));
        let p2p = Arc::new(P2P {
            dispatcher,
            state: config,
            chain_type,
            chain,
            peers: peers.clone(),
            poll: Arc::new(Poll::new().unwrap()),
            next_peer_id: AtomicUsize::new(0),
            waker: Arc::new(Mutex::new(HashMap::new())),
            listener: Arc::new(Mutex::new(HashMap::new())),
        });
        let p2p2 = p2p.clone();
        thread::Builder::new()
            .name("p2pcntrl".to_string())
            .spawn(move || p2p2.control_loop(control_receiver))
            .unwrap();
        (p2p, P2PControlSender::new(control_sender, peers, back_pressure))
    }

    pub fn connected_peers(&self) -> Vec<SocketAddr> {
        self.peers.read().unwrap().values()
            .filter_map(|peer| peer.lock().unwrap().stream.peer_addr().ok())
            .collect()
    }

    pub fn n_connected_peers(&self) -> usize {
        self.peers.read().unwrap().len()
    }

    fn control_loop(&self, receiver: P2PControlReceiver) {
        while let Ok(control) = receiver.recv() {
            match control {
                P2PControl::Ban(peer_id, score) => {
                    self.ban(peer_id, score);
                },
                P2PControl::Disconnect(peer_id) => {
                    self.disconnect(peer_id, false);
                },
                P2PControl::Height(height) => {
                    self.state.set_height(height);
                }
                P2PControl::Bind(addr) => {
                    match self.add_listener(addr) {
                        Ok(()) => println!("listen to {}", addr),
                        Err(err) => println!("failed to listen to {} with {}", addr, err)
                    }
                },
                P2PControl::Broadcast(message) => {
                    for peer in self.peers.read().unwrap().values() {
                        peer.lock()
                            .unwrap()
                            .send(message.clone())
                            .expect("could not send to peer");
                    }
                }
                P2PControl::Send(peer_id, message) => {
                    if let Some(peer) = self.peers.read().unwrap().get(&peer_id) {
                        peer.lock()
                            .unwrap()
                            .send(message)
                            .expect("could not send to peer");
                    }
                }
            }
        }
        panic!("P2P Control loop failed");
    }

    fn add_listener(&self, bind: SocketAddr) -> Result<(), io::Error> {
        let mut listener = TcpListener::bind(bind)?;
        let token = Token(self.next_peer_id.fetch_add(1, Ordering::Relaxed));
        self.poll.registry().register(&mut listener, token, Interest::READABLE | Interest::WRITABLE)?;
        self.listener.lock().unwrap().insert(token, Arc::new(listener));
        Ok(())
    }

    /// return a future that does not complete until the peer is connected
    pub fn add_peer(&self, source: PeerSource) -> impl Future<Output=Result<SocketAddr, Error>> + Send {
        // new token, never re-using previously connected peer's id
        // so log messages are easier to follow
        let token = Token(self.next_peer_id.fetch_add(1, Ordering::Relaxed));
        let pid = PeerId::new(self.chain_type, token);
        let peers = self.peers.clone();
        let peers2 = self.peers.clone();
        let waker = self.waker.clone();

        self.connecting(pid, source)
            .map_err(move |e| {
                let mut peers = peers2.write().unwrap();
                if let Some(peer) = peers.remove(&pid) {
                    peer.lock().unwrap().stream.shutdown(Shutdown::Both).unwrap_or(());
                }
                e
            })
            .and_then(move |addr| {
                future::poll_fn(move |ctx| {
                    if peers.read().unwrap().get(&pid).is_some() {
                        waker.lock().unwrap().insert(pid, ctx.waker().clone());
                        Async::Pending
                    } else {
                        println!("finished orderly peer={}", pid);
                        Async::Ready(Ok(addr))
                    }
                })
            })
    }

    fn connecting(&self, pid: PeerId, source: PeerSource) -> impl Future<Output=Result<SocketAddr, Error>> + Send {
        let version = self.state.version(self.state.chain_type().localhost(),self.state.chain_type().protocol_version());
        let peers = self.peers.clone();
        let peers2 = self.peers.clone();
        let poll = self.poll.clone();
        let waker = self.waker.clone();
        // todo: avoid cloning
        future::poll_fn(move |_| // {
            match Self::connect(version.clone(), peers.clone(), poll.clone(), pid, source.clone()) {
                Ok(addr) => Async::Ready(Ok(addr)),
                Err(e) => Async::Ready(Err(e))
            }
        ).and_then(move |addr| {
            let handshake_future = future::poll_fn(move |ctx|
                if let Some(peer) = peers2.read().unwrap().get(&pid) {
                    if peer.lock().unwrap().connected {
                        Async::Ready(Ok(addr))
                    } else {
                        waker.lock().unwrap().insert(pid, ctx.waker().clone());
                        Async::Pending
                    }
                } else {
                    Async::Ready(Err(Error::Handshake))
                }
            );
            let timeout_future = Delay::new(Duration::from_secs(CONNECT_TIMEOUT_SECONDS));
            future::select(handshake_future, timeout_future).map(|res| match res {
                Either::Left((status, timeout)) => status,
                Either::Right(..) => Err(Error::HandshakeTimeout)
            })
            // future::select_ok(vec![handshake_future, timeout_future]).map_err(|_| Error::HandshakeTimeout)
        })
    }

    // initiate connection to peer
    fn connect(version: Message, peers: Arc<RwLock<PeerMap>>, poll: Arc<Poll>, pid: PeerId, source: PeerSource) -> Result<SocketAddr, Error> {
        let outgoing;
        let addr;
        let stream;
        match source {
            PeerSource::Outgoing(a) => {
                if peers.read().unwrap().values().any(|peer| peer.lock().unwrap().stream.peer_addr().map_or(false, |addr| a.ip() == addr.ip())) {
                    println!("rejecting outgoing connect for a peer already connected");
                    return Err(Error::Handshake);
                }

                addr = a;
                outgoing = true;
                println!("trying outgoing connect to {} peer={}", addr, pid);
                stream = TcpStream::connect(addr)?;
            },
            PeerSource::Incoming(listener) => {
                let (s, a) = listener.accept()?;
                if peers.read().unwrap().values().any(|peer| peer.lock().unwrap().stream.peer_addr().map_or(false, |addr| a.ip() == addr.ip())) {
                    println!("rejecting incoming connect from a peer already connected");
                    s.shutdown(Shutdown::Both).unwrap_or(());
                    return Err(Error::Handshake);
                }
                addr = a;
                stream = s;
                println!("trying incoming connect to {} peer={}", addr, pid);
                outgoing = false;
            }
        };
        // create lock protected peer object
        let peer = Mutex::new(Peer::new(pid, stream, poll.clone(), outgoing)?);
        let mut peers = peers.write().unwrap();
        // add to peer map
        peers.insert(pid, peer);
        let stored_peer = peers.get(&pid).unwrap();
        if outgoing {
            stored_peer.lock().unwrap().register_write()?;
        } else {
            stored_peer.lock().unwrap().register_read()?;
        }
        if outgoing {
            // send this node's version message to peer
            peers.get(&pid).unwrap().lock().unwrap().send(version)?;
        }

        Ok(addr)
    }

    fn disconnect(&self, pid: PeerId, banned: bool) {
        self.dispatcher.send(PeerNotification::Disconnected(pid, banned));
        {
            // remove from peers before waking up, so disconnect is recognized
            let mut peers = self.peers.write().unwrap();
            if let Some(peer) = peers.remove(&pid) {
                peer.lock().unwrap().stream.shutdown(Shutdown::Both).unwrap_or(());
            }
        }
        {
            let mut wakers = self.waker.lock().unwrap();
            if let Some(waker) = wakers.remove(&pid) {
                println!("waking for disconnect peer={}", pid);
                waker.wake();
            }
        }
    }

    fn connected(&self, pid: PeerId, address: Option<SocketAddr>) {
        self.dispatcher.send(PeerNotification::Connected(pid, address));
    }

    fn ban(&self, pid: PeerId, increment: u32) {
        let mut disconnect = false;
        if let Some(peer) = self.peers.read().unwrap().get(&pid) {
            let mut locked_peer = peer.lock().unwrap();
            locked_peer.ban += increment;
            println!("ban score {} for peer={}", locked_peer.ban, pid);
            if locked_peer.ban >= BAN {
                disconnect = true;
            }
        }
        if disconnect {
            println!("ban peer={}", pid);
            self.disconnect(pid, true);
        }
    }

    fn event_processor(&self, event: &Event, pid: PeerId, needed_services: u64, iobuf: &mut [u8]) -> Result<(), Error> {
        if event.is_read_closed() || event.is_error() {
            println!("left us peer={}", pid);
            self.disconnect(pid, false);
        } else {
            // check for ability to write before read, to get rid of data before buffering more read
            // token should only be registered for write if there is a need to write
            // to avoid superfluous wakeups from poll
            if event.is_writable() {
                println!("writeable peer={}", pid);
                // figure peer's entry in the peer map, provided it is still connected, ignore event if not
                if let Some(peer) = self.peers.read().unwrap().get(&pid) {
                    // get and lock the peer from the peer map entry
                    let mut locked_peer = peer.lock().unwrap();
                    loop {
                        let mut get_next = true;
                        // if there is previously unfinished write
                        if let Ok(len) = locked_peer.write_buffer.read_ahead(iobuf) {
                            if len > 0 {
                                println!("try write {} bytes to peer={}", len, pid);
                                // try writing it out now
                                let mut wrote = 0;
                                while let Ok(wlen) = locked_peer.stream.write(&iobuf[wrote..len]) {
                                    if wlen == 0 {
                                        println!("would block on peer={}", pid);
                                        // do not fetch next message until there is an unfinished write
                                        get_next = false;
                                        break;
                                    }
                                    println!("wrote {} bytes to peer={}", wlen, pid);
                                    // advance buffer and drop used store
                                    locked_peer.write_buffer.advance(wlen);
                                    locked_peer.write_buffer.commit();
                                    wrote += wlen;
                                    if wrote == len {
                                        break;
                                    }
                                }
                            }
                        }
                        if get_next {
                            // get an outgoing message from the channel (if any)
                            if let Some(msg) = locked_peer.try_receive() {
                                // serialize the message
                                let raw = self.state.pack(msg);
                                println!("next message {:?} to peer={}", raw, pid);
                                // refill write buffer
                                self.state.encode(raw, &mut locked_peer.write_buffer)?;
                            } else {
                                // no unfinished write and no outgoing message
                                // keep registered only for read events
                                println!("done writing to peer={}", pid);
                                locked_peer.reregister_read()?;
                                break;
                            }
                        }
                    }
                }
            }
            // is peer readable ?
            if event.is_readable() {
                println!("readable peer={}", pid);
                // collect incoming messages here
                // incoming messages are collected here for processing after release
                // of the lock on the peer map.
                let mut incoming = Vec::new();
                // disconnect if set
                let mut disconnect = false;
                // how to disconnect
                let mut ban = false;
                // new handshake if set
                let mut handshake = false;
                // peer address
                let mut address = None;
                // read lock peer map and retrieve peer
                if let Some(peer) = self.peers.read().unwrap().get(&pid) {
                    // lock the peer from the peer
                    let mut locked_peer = peer.lock().unwrap();
                    // read the peer's socket
                    if let Ok(len) = locked_peer.stream.read(iobuf) {
                        println!("received {} bytes from peer={}", len, pid);
                        if len == 0 {
                            println!("read zero length message, disconnecting peer={}", pid);
                            disconnect = true;
                        }
                        // accumulate in a buffer
                        locked_peer.read_buffer.write_all(&iobuf[0..len])?;
                        // extract messages from the buffer
                        while let Some(msg) = self.state.decode(&mut locked_peer.read_buffer)? {
                            println!("received {:?} peer={}", msg.r#type, pid);
                            if locked_peer.connected {
                                // regular processing after handshake
                                incoming.push(msg);
                            } else {
                                // have to get both version and verack to complete handhsake
                                if !(locked_peer.version.is_some() && locked_peer.flags.contains(PeerStateFlags::GOT_VERACK)) {
                                    // before handshake complete
                                    if let Ok(response) = self.state.unpack(msg) {
                                        if let Response::Version(version) = response {
                                            if locked_peer.version.is_some() {
                                                // repeated version
                                                disconnect = true;
                                                ban = true;
                                                println!("misbehaving peer, repeated version peer={}", pid);
                                                break;
                                            }
                                            if version.nonce == self.state.nonce() {
                                                // connect to myself
                                                disconnect = true;
                                                ban = true;
                                                println!("rejecting to connect to myself peer={}", pid);
                                                break;
                                            } else {
                                                if version.version < self.state.chain_type().min_protocol_version() || (needed_services & version.services) != needed_services {
                                                    println!("rejecting peer of version {} and services {:b} peer={}", version.version, version.services, pid);
                                                    disconnect = true;
                                                    break;
                                                } else {
                                                    if !locked_peer.outgoing {
                                                        // send own version message to incoming peer
                                                        let addr = locked_peer.stream.peer_addr()?;
                                                        println!("send version to incoming connection {}", addr);
                                                        // do not show higher version than the peer speaks
                                                        let version = self.state.version(addr, version.version);
                                                        locked_peer.send(version)?;
                                                    } else {
                                                        // outgoing connects should not be behind this
                                                        if version.last_block_height < self.state.get_height() {
                                                            println!("rejecting to connect with height {} peer={}", version.last_block_height, pid);
                                                            disconnect = true;
                                                            break;
                                                        }
                                                    }
                                                    println!("accepting peer of version {} and services {:b} peer={}", version.version, version.services, pid);
                                                    // acknowledge version message received
                                                    locked_peer.send(self.state.verack())?;
                                                    // all right, remember this peer
                                                    println!("client {} height: {} peer={}", version.useragent, version.last_block_height, pid);
                                                    // reduce protocol version to our capabilities
                                                    let vm = VersionCarrier {
                                                        version: min(version.version, self.state.chain_type().protocol_version()),
                                                        services: version.services,
                                                        timestamp: SystemTime::seconds_since_1970(),
                                                        receiver_address: SocketAddr::V4(SocketAddrV4::new(version.addr_recv_address.to_ipv4_addr(), version.addr_recv_port)),
                                                        sender_address: SocketAddr::V4(SocketAddrV4::new(version.addr_trans_address.to_ipv4_addr(), version.addr_trans_port)),
                                                        nonce: version.nonce,
                                                        user_agent: self.chain_type.user_agent(),
                                                        start_height: 0 /*v.start_height as u32*/,
                                                        relay: false /*v.relay*/
                                                    };

                                                    locked_peer.version = Some(vm);
                                                }
                                            }
                                        } else if response.r#type() == MessageType::Verack {
                                            if locked_peer.flags.contains(PeerStateFlags::GOT_VERACK) {
                                                // repeated verack
                                                disconnect = true;
                                                ban = true;
                                                println!("misbehaving peer, repeated version peer={}", pid);
                                                break;
                                            }
                                            println!("got verack peer={}", pid);
                                            locked_peer.flags |= PeerStateFlags::GOT_VERACK;
                                        } else {
                                            println!("misbehaving peer unexpected message before handshake peer={}", pid);
                                            // some other message before handshake
                                            disconnect = true;
                                            ban = true;
                                            break;
                                        }
                                        if locked_peer.version.is_some() && locked_peer.flags.contains(PeerStateFlags::GOT_VERACK) {
                                            locked_peer.connected = true;
                                            handshake = true;
                                            address = locked_peer.stream.peer_addr().ok()
                                        }
                                    } else {
                                        println!("Ban for malformed message peer={}", pid);
                                        disconnect = true;
                                        ban = true;
                                        break;
                                    }
                                }
                            }
                        }
                    } else {
                        println!("IO error reading peer={}", pid);
                        disconnect = true;
                    }
                }
                if disconnect {
                    println!("disconnecting peer={}", pid);
                    self.disconnect(pid, ban);
                } else {
                    if handshake {
                        println!("handshake peer={}", pid);
                        self.connected(pid, address);
                        if let Some(w) = self.waker.lock().unwrap().remove(&pid) {
                            println!("waking for handshake");
                            w.wake();
                        }
                    }
                    // process queued incoming messages outside lock
                    // as process could call back to P2P
                    for msg in incoming {
                        println!("processing {:?} for peer={}", msg.r#type, pid);
                        if let Ok(m) = self.state.unpack(msg) {
                            self.dispatcher.send(PeerNotification::Incoming(pid, m));
                        } else {
                            println!("Ban for malformed message peer={}", pid);
                            self.disconnect(pid, true);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // /// run the message dispatcher loop
    // /// this method does not return unless there is an error obtaining network events
    // /// run in its own thread, which will process all network events
    // pub fn poll_events(&mut self, needed_services: u64, spawn: &mut dyn Spawn) {
    //     // events buffer
    //     let mut events = Events::with_capacity(EVENT_BUFFER_SIZE);
    //     // IO buffer
    //     let mut iobuf = vec!(0u8; IO_BUFFER_SIZE);
    //
    //     loop {
    //         // get the next batch of events
    //         self.poll.poll(&mut events, None)
    //             .expect("can not poll mio events");
    //
    //         // iterate over events
    //         for event in events.iter() {
    //             // check for listener
    //             if let Some(server) = self.is_listener(event.token()) {
    //                 println!("incoming connection request");
    //                 spawn.spawn(self.add_peer(PeerSource::Incoming(server)).map(|_| ())).expect("can not add peer for incoming connection");
    //             } else {
    //                 // construct the id of the peer the event concerns
    //                 let pid = PeerId::new(network, event.token());
    //                 if let Err(error) = self.event_processor(event, pid, needed_services, iobuf.as_mut_slice()) {
    //                     use std::error::Error;
    //
    //                     println!("error {:?} peer={}", error.source(), pid);
    //                     self.ban(pid, 10);
    //                 }
    //             }
    //         }
    //     }
    // }

    fn is_listener(&self, token: Token) -> Option<Arc<TcpListener>> {
        if let Some(server) = self.listener.lock().unwrap().get(&token) {
            return Some(server.clone())
        }
        None
    }
}

