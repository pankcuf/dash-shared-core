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

use std::{cmp::min, collections::HashMap, io, io::{Read, Write}, net::{Shutdown, SocketAddr, SocketAddrV4}, sync::{Arc, atomic::{AtomicUsize, Ordering}, mpsc, Mutex, RwLock}, time::{Duration, SystemTime}};
use std::collections::HashSet;
use futures::{future, future::Either, Future, FutureExt, task::{Poll, Spawn, SpawnExt}, TryFutureExt};
use futures_timer::Delay;
use mio::{event::Event, Events, Interest, net::{TcpListener, TcpStream}, Token};
use crate::chain::{Chain, common::ChainType, network::message::{Message, Response}};
use crate::manager::peer_manager::Error;
use crate::network::p2p::{DashP2PState, PeerSelector};
use crate::network::pipeline::ThreadName;
use crate::util::{Shared, TimeUtil};
use super::{ListenerMap, P2PControlDispatcher, P2PControlNotification, P2PControlReceiver, P2PNotificationDispatcher, P2PNotificationSender, Peer, PeerId, PeerSource, PeerState, PeerStateFlags, VersionCarrier, WakerMap};

const EVENT_BUFFER_SIZE: usize = 1024;
pub(crate) const IO_BUFFER_SIZE: usize = EVENT_BUFFER_SIZE*EVENT_BUFFER_SIZE;
pub(crate) const CONNECT_TIMEOUT_SECONDS: u64 = 10;


pub struct P2P {
    notification_dispatcher: P2PNotificationDispatcher,
    pub state: DashP2PState,
    pub chain_type: ChainType,
    pub chain: Shared<Chain>,
    peer_selector: Arc<RwLock<PeerSelector>>,
    // peers: Arc<RwLock<PeerMap>>,
    poll: Arc<RwLock<mio::Poll>>,
    wakers: Arc<Mutex<WakerMap>>,
    listeners: Arc<Mutex<ListenerMap>>,
    next_peer_id: AtomicUsize,
}

impl P2P {
    pub fn new(notification_sender: P2PNotificationSender, back_pressure: usize, chain_type: ChainType, chain: Shared<Chain>) -> (Arc<P2P>, P2PControlDispatcher, HashSet<SocketAddr>) {
        let state = DashP2PState::new(None, chain_type, chain.clone());
        let (control_sender, control_receiver) = mpsc::channel();
        let selector = PeerSelector::new(chain_type);
        let addresses = selector.select_more_peers();

        let peer_selector = Arc::new(RwLock::new(selector));
        let notification_dispatcher = P2PNotificationDispatcher::new(notification_sender);
        let control_dispatcher = P2PControlDispatcher::new(control_sender, peer_selector.clone(), back_pressure);
        let poll = Arc::new(RwLock::new(mio::Poll::new().unwrap()));
        let p2p = P2P {
            notification_dispatcher,
            state,
            chain_type,
            chain,
            peer_selector,
            poll,
            next_peer_id: AtomicUsize::new(0),
            wakers: Arc::new(Mutex::new(HashMap::new())),
            listeners: Arc::new(Mutex::new(HashMap::new())),
        };
        // let p2parc = Arc::new(RwLock::new(p2p));
        let p2parc = Arc::new(p2p);
        let p2p2 = p2parc.clone();
        ThreadName::P2PManager
            .thread(chain_type)
            .spawn(move || p2p2.control_loop(control_receiver))
            .unwrap();
        (p2parc, control_dispatcher, addresses)
    }

    pub fn connected_peers(&self) -> Vec<SocketAddr> {
        self.peer_selector.read().unwrap().connected_peers()
    }

    fn control_loop(&self, receiver: P2PControlReceiver) {
        while let Ok(notification) = receiver.recv() {
            debug!("control_loop: {:?}", notification);
            match notification {
                P2PControlNotification::Ban(peer_id, score) => self.ban(peer_id, score),
                P2PControlNotification::Disconnect(pid) => self.disconnect(pid, false),
                P2PControlNotification::Height(height) => self.state.set_height(height),
                P2PControlNotification::Broadcast(message) => self.broadcast(message),
                P2PControlNotification::Send(pid, message) => self.send(pid, message),
                P2PControlNotification::Bind(addr) => match self.add_listener(addr) {
                    Ok(()) => debug!("listen to {}", addr),
                    Err(err) => debug!("failed to listen to {} with {}", addr, err)
                },
            }
        }
        panic!("P2P Control loop failed");
    }

    fn add_listener(&self, bind: SocketAddr) -> Result<(), io::Error> {
        let mut listener = TcpListener::bind(bind)?;
        let token = Token(self.next_peer_id.fetch_add(1, Ordering::Relaxed));
        if let Ok(poll) = self.poll.read() {
            poll.registry().register(&mut listener, token, Interest::READABLE | Interest::WRITABLE)?;
        }
        self.listeners.lock().unwrap().insert(token, Arc::new(listener));
        Ok(())
    }

    /// return a future that does not complete until the peer is connected
    pub fn add_peer(&self, source: PeerSource) -> impl Future<Output=Result<SocketAddr, Error>> + Send {
        let token = Token(self.next_peer_id.fetch_add(1, Ordering::Relaxed));
        let pid = PeerId::new(self.chain_type, token);
        let polling_selector = self.peer_selector.clone();
        let connecting_selector = self.peer_selector.clone();
        let waker = self.wakers.clone();
        debug!("add_peer: {:?}", source);
        self.connecting(pid, source)
            .map_err(move |e| {
                debug!("add_peer.connecting.error: {:?}", e);
                connecting_selector.write().unwrap().shutdown(pid);
                e
            })
            .and_then(move |addr| {
                debug!("add_peer.connecting.success: {:?}", addr);
                future::poll_fn(move |ctx| {
                    if polling_selector.read().unwrap().peer_by_id(pid).is_some() {
                        waker.lock().unwrap().insert(pid, ctx.waker().clone());
                        Poll::Pending
                    } else {
                        Poll::Ready(Ok(addr))
                    }
                })
            })
    }

    fn connecting(&self, pid: PeerId, source: PeerSource) -> impl Future<Output=Result<SocketAddr, Error>> + Send {
        let version = self.state.version(self.state.chain_type().localhost(),self.state.chain_type().protocol_version());
        let connecting_selector = self.peer_selector.clone();
        let poll_selector = self.peer_selector.clone();
        let poll = self.poll.clone();
        let waker = self.wakers.clone();
        // todo: avoid cloning
        debug!("connecting: {}", pid);
        future::poll_fn(move |_| {
            debug!("connecting.poll_fn: {}", pid);
            match Self::connect(version.clone(), connecting_selector.clone(), poll.clone(), pid, source.clone()) {
                Ok(addr) => Poll::Ready(Ok(addr)),
                Err(e) => Poll::Ready(Err(e))
            }
        }).and_then(move |addr| {
            let handshake_future = future::poll_fn(move |ctx|
                if let Some(peer) = poll_selector.read().unwrap().peer_by_id(pid) {
                    if peer.lock().unwrap().connected {
                        debug!("connecting.handshake.ready: {} {:?}", pid, addr);
                        Poll::Ready(Ok(addr))
                    } else {
                        waker.lock().unwrap().insert(pid, ctx.waker().clone());
                        debug!("connecting.handshake.pending: {} {:?}", pid, addr);
                        Poll::Pending
                    }
                } else {
                    debug!("connecting.handshake.no_peer: {} {:?}", pid, addr);
                    Poll::Ready(Err(Error::Handshake))
                }
            );
            debug!("connecting.and_then: {}", pid);
            future::select(handshake_future, Delay::new(Duration::from_secs(CONNECT_TIMEOUT_SECONDS)))
                .map(|res| match res {
                    Either::Left((status, timeout)) => status,
                    Either::Right(..) => Err(Error::HandshakeTimeout)
                })
        })
    }

    fn connect(version: Message, selector: Arc<RwLock<PeerSelector>>, poll: Arc<RwLock<mio::Poll>>, pid: PeerId, source: PeerSource) -> Result<SocketAddr, Error> {
        debug!("connect: {:?}", source);
        let (is_outgoing, addr, stream) = match source {
            PeerSource::Outgoing(a) => {
                if selector.read().unwrap().has_peer_with_address(a) {
                    return Err(Error::Handshake);
                }
                (true, a, TcpStream::connect(a)?)
            },
            PeerSource::Incoming(listener) => {
                let (s, a) = listener.accept()?;
                if selector.read().unwrap().has_peer_with_address(a) {
                    s.shutdown(Shutdown::Both).unwrap_or(());
                    return Err(Error::Handshake);
                }
                (false, a, s)
            }
        };

        let mut selector = selector.write().unwrap();
        selector.add(Peer::new(pid, stream, poll, is_outgoing)?)?;
        if is_outgoing {
            selector.peer_by_id(pid).unwrap().lock().unwrap().send(version)?;
        }
        Ok(addr)
    }

    fn disconnect(&self, pid: PeerId, banned: bool) {
        debug!("disconnect: {:?} {}", pid, banned);
        self.notification_dispatcher.send_disconnected(pid, banned);
        {
            // remove from peers before waking up, so disconnect is recognized
            let mut selector = self.peer_selector.write().unwrap();
            selector.shutdown(pid);
        }
        {
            let mut wakers = self.wakers.lock().unwrap();
            if let Some(waker) = wakers.remove(&pid) {
                waker.wake();
            }
        }
    }

    fn connected(&self, pid: PeerId, address: Option<SocketAddr>) {
        debug!("connected: {:?} {:?}", pid, address);
        self.notification_dispatcher.send_connected(pid, address);
    }

    fn ban(&self, pid: PeerId, increment: u32) {
        debug!("ban: {:?} {:?}", pid, increment);
        if self.peer_selector.read().unwrap().ban(pid, increment) {
            self.disconnect(pid, true);
        }
    }

    fn broadcast(&self, message: Message) {
        for peer in self.peer_selector.read().unwrap().peers.values() {
            peer.lock()
                .unwrap()
                .send(message.clone())
                .expect("could not send to peer");
        }
    }

    fn send(&self, pid: PeerId, message: Message) {
        debug!("send: {:?}", pid);
        if let Some(peer) = self.peer_selector.read().unwrap().peer_by_id(pid) {
            peer.lock()
                .unwrap()
                .send(message)
                .expect("could not send to peer");
        }
    }

    fn event_processor(&self, event: &Event, pid: PeerId, needed_services: u64, iobuf: &mut [u8]) -> Result<(), Error> {
        if event.is_read_closed() || event.is_error() {
        // if event.is_error() {
            debug!("event_processor -> disconnect");
            if let Some(peer) = self.peer_selector.read().unwrap().peer_by_id(pid) {
                debug!("event_processor -> peer lock");
                let mut locked_peer = peer.lock().unwrap();
                debug!("event_processor -> peer unlocked");
                match locked_peer.stream.read(iobuf) {
                    Ok(len) => {
                        debug!("closed with message: {} {:?}", len, &iobuf[..len]);
                    },
                    Err(err) => {
                        debug!("closed with error: {}", err);
                    }
                }
            }
            self.disconnect(pid, false);
        } else {
            // check for ability to write before read, to get rid of data before buffering more read
            // token should only be registered for write if there is a need to write
            // to avoid superfluous wakeups from poll
            if event.is_writable() {
                debug!("event_processor -> is_writable");
                if let Some(peer) = self.peer_selector.read().unwrap().peer_by_id(pid) {
                    let mut locked_peer = peer.lock().unwrap();
                    debug!("event_processor -> is_writable. locked_peer");
                    loop {
                        let mut get_next = true;
                        if let Ok(len @ 1..) = locked_peer.write_buffer.read_ahead(iobuf) {
                            debug!("event_processor -> is_writable. loop: {}", len);
                            let mut wrote = 0;
                            while let Ok(wlen) = locked_peer.stream.write(&iobuf[wrote..len]) {
                                debug!("event_processor -> is_writable. written: {}", wlen);
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
                            if let Ok(message) = locked_peer.try_receive() {
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
                debug!("event_processor -> is_readable");
                let mut incoming = Vec::new();
                let mut disconnect = false;
                let mut ban = false;
                let mut handshake = false;
                let mut address = None;
                if let Some(peer) = self.peer_selector.read().unwrap().peer_by_id(pid) {
                    debug!("event_processor -> peer lock");
                    let mut locked_peer = peer.lock().unwrap();
                    debug!("event_processor -> peer unlocked");
                    match locked_peer.stream.read(iobuf) {
                        Ok(len) => {
                            debug!("event_processor -> stream.read: {:?}", len);
                            if len == 0 {
                                disconnect = true;
                            }
                            locked_peer.read_buffer.write_all(&iobuf[0..len])?;
                            while let Some(msg) = self.state.decode(&mut locked_peer.read_buffer)? {
                                debug!("event_processor -> decoded msg {:?}", msg);
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
                            self.notification_dispatcher.send_incoming(pid, m);
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
        self.peer_selector.read().unwrap().len()
    }

    pub fn select_peers(&self) -> Vec<SocketAddr> {
        self.peer_selector.read().unwrap().select_peers()
    }

    /// run the message dispatcher loop
    /// this method does not return unless there is an error obtaining network events
    /// run in its own thread, which will process all network events
    pub fn poll_events(&self, needed_services: u64, executor: &mut dyn Spawn) {
        let mut events = Events::with_capacity(EVENT_BUFFER_SIZE);
        let mut iobuf = vec![0u8; IO_BUFFER_SIZE];
        loop {
            let mut poll = self.poll.write().unwrap();
            poll.poll(&mut events, Some(Duration::from_millis(100))).expect("can not poll mio events");
            drop(poll);
            if !events.is_empty() {
                debug!("poll_events: {:?}", events);
            }
            for event in events.iter() {
                if let Some(listener) = self.listeners.lock().unwrap().get(&event.token()) {
                    executor.spawn(self.add_peer(PeerSource::Incoming(listener.clone())).map(|_| ()))
                        .expect("can not add peer for incoming connection");
                } else {
                    let pid = PeerId::new(self.chain_type, event.token());
                    if let Err(error) = self.event_processor(event, pid, needed_services, iobuf.as_mut_slice()) {
                        self.ban(pid, 10);
                    }
                }
            }
        }
    }

    pub fn select_more_peers(&self, executor: &mut dyn Spawn) {
        if let Ok(selector) = self.peer_selector.try_read() {
            selector
                .select_more_peers()
                .iter()
                .for_each(|choice| {
                    executor.spawn(self.add_peer(PeerSource::Outgoing(*choice)).map(|_| ()))
                        .expect("can not add peer for outgoing connection");

                })
        }
    }

}

