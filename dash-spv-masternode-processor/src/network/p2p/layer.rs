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
use futures::task::{Poll as Async, Spawn, SpawnExt};
use futures::task::Waker;

use std::{cmp::min, collections::HashMap, io, io::{Read, Write}, net::{Shutdown, SocketAddr}, sync::{Arc, atomic::{AtomicUsize, Ordering}, mpsc, Mutex, RwLock}, thread, time::{Duration, SystemTime}};
use std::net::SocketAddrV4;
use futures::future::Either;
use futures_timer::Delay;
use mio::{Events, Interest, Poll, Token};
use mio::event::Event;
use mio::net::{TcpListener, TcpStream};
use crate::chain::Chain;
use crate::chain::common::{ChainType, IHaveChainSettings};
use crate::chain::network::message::message::Message;
use crate::chain::network::message::response::Response;
use crate::manager::peer_manager::Error;
use crate::network::p2p::peer::{Peer, PeerId, PeerMap};
use crate::network::p2p::state::PeerState;
use crate::network::p2p::state_flags::PeerStateFlags;
use crate::util::{Shared, TimeUtil};

pub const IO_BUFFER_SIZE:usize = 1024*1024;
pub const EVENT_BUFFER_SIZE:usize = 1024;
const CONNECT_TIMEOUT_SECONDS: u64 = 5;
const BAN :u32 = 100;

pub type ListenerMap = HashMap<Token, Arc<TcpListener>>;
pub type WakerMap = HashMap<PeerId, Waker>;

/// A message from network to downstream
#[derive(Clone)]
pub enum P2PNotification {
    Outgoing(Message),
    Incoming(PeerId, Response),
    Connected(PeerId, Option<SocketAddr>),
    Disconnected(PeerId, bool/*banned*/)
}

/// a map of peer id to peers
pub type P2PNotificationReceiver = mpsc::Receiver<P2PNotification>;
pub type P2PNotificationSender = mpsc::SyncSender<P2PNotification>;

pub enum P2PControlNotification {
    Send(PeerId, Message),
    Broadcast(Message),
    Ban(PeerId, u32),
    Disconnect(PeerId),
    Height(u32),
    Bind(SocketAddr)
}


type P2PControlReceiver = mpsc::Receiver<P2PControlNotification>;

#[derive(Clone)]
pub struct P2PControlDispatcher {
    sender: Arc<Mutex<mpsc::Sender<P2PControlNotification>>>,
    peers: Arc<RwLock<PeerMap>>,
    pub back_pressure: usize
}

impl P2PControlDispatcher {
    pub fn new(sender: mpsc::Sender<P2PControlNotification>, peers: Arc<RwLock<PeerMap>>, back_pressure: usize) -> P2PControlDispatcher {
        P2PControlDispatcher { sender: Arc::new(Mutex::new(sender)), peers, back_pressure }
    }

    pub fn send(&self, control: P2PControlNotification) {
        self.sender.lock()
            .unwrap()
            .send(control)
            .expect("P2P control send failed");
    }

    pub fn send_network(&self, peer: PeerId, msg: Message) {
        self.send(P2PControlNotification::Send(peer, msg))
    }

    pub fn ban(&self, peer: PeerId, increment: u32) {
        self.send(P2PControlNotification::Ban(peer, increment))
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
pub struct P2PNotificationDispatcher {
    sender: Arc<Mutex<P2PNotificationSender>>
}

impl P2PNotificationDispatcher {
    pub fn new(sender: P2PNotificationSender) -> P2PNotificationDispatcher {
        P2PNotificationDispatcher { sender: Arc::new(Mutex::new(sender)) }
    }

    pub fn send(&self, msg: P2PNotification) {
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

pub struct P2P<STATE: PeerState + Send + Sync + 'static> {
    dispatcher: P2PNotificationDispatcher,
    pub state: STATE,
    pub chain_type: ChainType,
    pub chain: Shared<Chain>,
    peers: Arc<RwLock<PeerMap>>,
    poll: Arc<Mutex<Poll>>,
    wakers: Arc<Mutex<WakerMap>>,
    listeners: Arc<Mutex<ListenerMap>>,
    next_peer_id: AtomicUsize,
}

impl<STATE: PeerState + Send + Sync> P2P<STATE> {
    pub fn new(state: STATE, dispatcher: P2PNotificationDispatcher, back_pressure: usize, chain_type: ChainType, chain: Shared<Chain>) -> (Arc<Mutex<P2P<STATE>>>, P2PControlDispatcher) {
        let (control_sender, control_receiver) = mpsc::channel();
        let peers = Arc::new(RwLock::new(PeerMap::new()));
        let p2p = Arc::new(Mutex::new(P2P {
            dispatcher,
            state,
            chain_type,
            chain,
            peers: peers.clone(),
            poll: Arc::new(Mutex::new(Poll::new().unwrap())),
            next_peer_id: AtomicUsize::new(0),
            wakers: Arc::new(Mutex::new(HashMap::new())),
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }));
        let p2p2 = p2p.clone();
        thread::Builder::new()
            .name(format!("peer_manager_{}", chain_type.name()))
            .spawn(move || p2p2.lock().unwrap().control_loop(control_receiver))
            .unwrap();
        (p2p, P2PControlDispatcher::new(control_sender, peers, back_pressure))
    }

    pub fn connected_peers(&self) -> Vec<SocketAddr> {
        self.peers.read().unwrap().values()
            .filter_map(|peer| peer.lock().unwrap().stream.peer_addr().ok())
            .collect()
    }

    fn control_loop(&self, receiver: P2PControlReceiver) {
        while let Ok(control) = receiver.recv() {
            match control {
                P2PControlNotification::Ban(peer_id, score) => {
                    self.ban(peer_id, score);
                },
                P2PControlNotification::Disconnect(peer_id) => {
                    self.disconnect(peer_id, false);
                },
                P2PControlNotification::Height(height) => {
                    self.state.set_height(height);
                }
                P2PControlNotification::Bind(addr) => {
                    match self.add_listener(addr) {
                        Ok(()) => println!("listen to {}", addr),
                        Err(err) => println!("failed to listen to {} with {}", addr, err)
                    }
                },
                P2PControlNotification::Broadcast(message) => {
                    for peer in self.peers.read().unwrap().values() {
                        peer.lock()
                            .unwrap()
                            .send(message.clone())
                            .expect("could not send to peer");
                    }
                }
                P2PControlNotification::Send(peer_id, message) => {
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
        if let Ok(poll) = self.poll.lock() {
            poll.registry().register(&mut listener, token, Interest::READABLE | Interest::WRITABLE)?;
        }
        self.listeners.lock().unwrap().insert(token, Arc::new(listener));
        Ok(())
    }

    /// return a future that does not complete until the peer is connected
    pub fn add_peer(&self, source: PeerSource) -> impl Future<Output=Result<SocketAddr, Error>> + Send {
        let token = Token(self.next_peer_id.fetch_add(1, Ordering::Relaxed));
        let pid = PeerId::new(self.chain_type, token);
        let peers = self.peers.clone();
        let peers2 = self.peers.clone();
        let waker = self.wakers.clone();

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
        let waker = self.wakers.clone();
        // todo: avoid cloning
        future::poll_fn(move |_|
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
            future::select(handshake_future, timeout_future)
                .map(|res| match res {
                    Either::Left((status, timeout)) => status,
                    Either::Right(..) => Err(Error::HandshakeTimeout)
                })
            // future::select_ok(vec![handshake_future, timeout_future]).map_err(|_| Error::HandshakeTimeout)
        })
    }

    fn connect(version: Message, peers: Arc<RwLock<PeerMap>>, poll: Arc<Mutex<Poll>>, pid: PeerId, source: PeerSource) -> Result<SocketAddr, Error> {
        let outgoing;
        let addr;
        let stream;
        match source {
            PeerSource::Outgoing(a) => {
                if peers.read().unwrap().values().any(|peer| peer.lock().unwrap().stream.peer_addr().map_or(false, |addr| a.ip() == addr.ip())) {
                    return Err(Error::Handshake);
                }
                addr = a;
                outgoing = true;
                stream = TcpStream::connect(addr)?;
            },
            PeerSource::Incoming(listener) => {
                let (s, a) = listener.accept()?;
                if peers.read().unwrap().values().any(|peer| peer.lock().unwrap().stream.peer_addr().map_or(false, |addr| a.ip() == addr.ip())) {
                    s.shutdown(Shutdown::Both).unwrap_or(());
                    return Err(Error::Handshake);
                }
                addr = a;
                stream = s;
                outgoing = false;
            }
        };
        let peer = Mutex::new(Peer::new(pid, stream, poll.clone(), outgoing)?);
        let mut peers = peers.write().unwrap();
        peers.insert(pid, peer);
        let stored_peer = peers.get(&pid).unwrap();
        if outgoing {
            stored_peer.lock().unwrap().register_write()?;
        } else {
            stored_peer.lock().unwrap().register_read()?;
        }
        if outgoing {
            peers.get(&pid).unwrap().lock().unwrap().send(version)?;
        }
        Ok(addr)
    }

    fn disconnect(&self, pid: PeerId, banned: bool) {
        self.dispatcher.send(P2PNotification::Disconnected(pid, banned));
        {
            // remove from peers before waking up, so disconnect is recognized
            let mut peers = self.peers.write().unwrap();
            if let Some(peer) = peers.remove(&pid) {
                peer.lock().unwrap().stream.shutdown(Shutdown::Both).unwrap_or(());
            }
        }
        {
            let mut wakers = self.wakers.lock().unwrap();
            if let Some(waker) = wakers.remove(&pid) {
                waker.wake();
            }
        }
    }

    fn connected(&self, pid: PeerId, address: Option<SocketAddr>) {
        self.dispatcher.send(P2PNotification::Connected(pid, address));
    }

    fn ban(&self, pid: PeerId, increment: u32) {
        let mut disconnect = false;
        if let Some(peer) = self.peers.read().unwrap().get(&pid) {
            let mut locked_peer = peer.lock().unwrap();
            locked_peer.ban += increment;
            if locked_peer.ban >= BAN {
                disconnect = true;
            }
        }
        if disconnect {
            self.disconnect(pid, true);
        }
    }

    fn event_processor(&self, event: &Event, pid: PeerId, needed_services: u64, iobuf: &mut [u8]) -> Result<(), Error> {
        if event.is_read_closed() || event.is_error() {
            self.disconnect(pid, false);
        } else {
            // check for ability to write before read, to get rid of data before buffering more read
            // token should only be registered for write if there is a need to write
            // to avoid superfluous wakeups from poll
            if event.is_writable() {
                if let Some(peer) = self.peers.read().unwrap().get(&pid) {
                    let mut locked_peer = peer.lock().unwrap();
                    loop {
                        let mut get_next = true;
                        if let Ok(len @ 1..) = locked_peer.write_buffer.read_ahead(iobuf) {
                            let mut wrote = 0;
                            while let Ok(wlen) = locked_peer.stream.write(&iobuf[wrote..len]) {
                                if wlen == 0 {
                                    get_next = false;
                                    break;
                                }
                                locked_peer.write_buffer.advance(wlen);
                                locked_peer.write_buffer.commit();
                                wrote += wlen;
                                if wrote == len {
                                    break;
                                }
                            }
                        }
                        if get_next {
                            if let Some(message) = locked_peer.try_receive() {
                                self.state.encode(self.state.pack(message), &mut locked_peer.write_buffer)?;
                            } else {
                                locked_peer.reregister_read()?;
                                break;
                            }
                        }
                    }
                }
            }
            if event.is_readable() {
                let mut incoming = Vec::new();
                let mut disconnect = false;
                let mut ban = false;
                let mut handshake = false;
                let mut address = None;
                if let Some(peer) = self.peers.read().unwrap().get(&pid) {
                    let mut locked_peer = peer.lock().unwrap();
                    match locked_peer.stream.read(iobuf) {
                        Ok(len) => {
                            if len == 0 {
                                disconnect = true;
                            }
                            locked_peer.read_buffer.write_all(&iobuf[0..len])?;
                            while let Some(msg) = self.state.decode(&mut locked_peer.read_buffer)? {
                                let has_no_version = locked_peer.version.is_none();
                                let has_no_verack = !locked_peer.flags.contains(PeerStateFlags::GOT_VERACK);
                                if locked_peer.connected {
                                    incoming.push(msg);
                                } else if has_no_version || has_no_verack {
                                    match self.state.unpack(msg) {
                                        Ok(response) => {
                                            match response {
                                                Response::Version(version) => if has_no_version && version.nonce != self.state.nonce() {
                                                    if version.version < self.state.chain_type().min_protocol_version() ||
                                                        (needed_services & version.services) != needed_services ||
                                                        locked_peer.outgoing && version.last_block_height < self.state.get_height() {
                                                        disconnect = true;
                                                        break;
                                                    } else {
                                                        if !locked_peer.outgoing {
                                                            let remote = locked_peer.stream.peer_addr()?;
                                                            let max_protocol_version = version.version;
                                                            let msg = self.state.version(remote, max_protocol_version);
                                                            locked_peer.send(msg)?;
                                                        }
                                                        locked_peer.send(self.state.verack())?;
                                                        locked_peer.version = Some(VersionCarrier {
                                                            version: min(version.version, self.state.chain_type().protocol_version()),
                                                            services: version.services,
                                                            timestamp: SystemTime::seconds_since_1970(),
                                                            receiver_address: SocketAddr::V4(SocketAddrV4::new(version.addr_recv_address.to_ipv4_addr(), version.addr_recv_port)),
                                                            sender_address: SocketAddr::V4(SocketAddrV4::new(version.addr_trans_address.to_ipv4_addr(), version.addr_trans_port)),
                                                            nonce: version.nonce,
                                                            user_agent: self.chain_type.user_agent(),
                                                            start_height: 0 /*v.start_height as u32*/,
                                                            relay: false /*v.relay*/
                                                        });
                                                    }
                                                },
                                                Response::Verack => if has_no_verack {
                                                    locked_peer.flags |= PeerStateFlags::GOT_VERACK;
                                                },
                                                _ => {
                                                    disconnect = true;
                                                    ban = true;
                                                    break;
                                                }
                                            }
                                            if locked_peer.version.is_some() && locked_peer.flags.contains(PeerStateFlags::GOT_VERACK) {
                                                locked_peer.connected = true;
                                                handshake = true;
                                                address = locked_peer.stream.peer_addr().ok()
                                            }
                                        },
                                        Err(_) => {
                                            disconnect = true;
                                            ban = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        },
                        Err(_) => {
                            disconnect = true;
                        }
                    }
                }
                if disconnect {
                    self.disconnect(pid, ban);
                } else {
                    if handshake {
                        self.connected(pid, address);
                        if let Some(w) = self.wakers.lock().unwrap().remove(&pid) {
                            w.wake();
                        }
                    }
                    for msg in incoming {
                        if let Ok(m) = self.state.unpack(msg) {
                            self.dispatcher.send(P2PNotification::Incoming(pid, m));
                        } else {
                            self.disconnect(pid, true);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn connected_peers_count(&self) -> usize {
        self.peers.read().unwrap().len()
    }

    /// run the message dispatcher loop
    /// this method does not return unless there is an error obtaining network events
    /// run in its own thread, which will process all network events
    pub fn poll_events(&mut self, needed_services: u64, spawn: &mut dyn Spawn) {
        let mut events = Events::with_capacity(EVENT_BUFFER_SIZE);
        let mut iobuf = vec![0u8; IO_BUFFER_SIZE];
        if let Ok(mut poll) = self.poll.lock() {
            loop {
                poll.poll(&mut events, None).expect("can not poll mio events");
                for event in events.iter() {
                    if let Some(listener) = self.listeners.lock().unwrap().get(&event.token()) {
                        spawn.spawn(self.add_peer(PeerSource::Incoming(listener.clone())).map(|_| ())).expect("can not add peer for incoming connection");
                    } else {
                        let pid = PeerId::new(self.chain_type, event.token());
                        if let Err(error) = self.event_processor(event, pid, needed_services, iobuf.as_mut_slice()) {
                            self.ban(pid, 10);
                        }
                    }
                }
            }
        }
    }

}

