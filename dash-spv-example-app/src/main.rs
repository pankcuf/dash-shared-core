use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::hash::Hash;
use std::net::SocketAddr;
use std::sync::{Arc, mpsc, Mutex};
use std::{cmp, thread};
use std::pin::Pin;
use std::task::Context;
// use std::io::{self, BufRead};
// use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use futures::{FutureExt, executor::{ThreadPool, ThreadPoolBuilder}};
// use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
// use tokio::net::TcpStream;
use futures::{future, Future, task::{Poll as Async, SpawnExt}};
use futures::task::noop_waker;
use futures_timer::Delay;

use dash_spv_masternode_processor::chain::{Chain, common::{ChainType, DevnetType}};
use dash_spv_masternode_processor::network::p2p::layer::{P2P, P2PControlDispatcher, P2PControlNotification, P2PNotificationDispatcher, P2PNotificationReceiver, P2PNotificationSender, PeerSource};
use dash_spv_masternode_processor::network::p2p::peer::PeerId;
use dash_spv_masternode_processor::network::p2p::state::DashP2PState;
use dash_spv_masternode_processor::secp256k1::rand::{RngCore, thread_rng};
use dash_spv_masternode_processor::util::Shared;

// fn main() -> Result<(), Box<dyn Error>> {
//     let chain_type = ChainType::TestNet;
//     let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
//     let shared_testnet = Chain::create_shared_testnet();
//     shared_testnet.wallet_with_seed_phrase::<bip0039::English>(seed_phrase, false, SystemTime::seconds_since_1970(), chain_type);
//     let mut testnet = shared_testnet.try_write().unwrap();
//     println!("Chain -> start sync");
//     testnet.start_sync();
//     println!("Chain -> start sync .... sleep 10 sec");
//     thread::sleep(Duration::from_secs(20));
//     println!("Chain -> stop sync");
//     testnet.stop_sync();
//     println!("Chain -> stop sync .... exit");
//     Ok(())
// }

// fn main() -> Result<(), Box<dyn Error>> {
//     use dash_spv_masternode_processor::manager::PeerManager;
//     use dash_spv_masternode_processor::chain::wallet::ext::constants::BIP39_CREATION_TIME;
//
//     let mut peer_manager = PeerManager::new(chain_type);
//     peer_manager.chain = chain.borrow();
//     peer_manager.connect(BIP39_CREATION_TIME);
//
//     Ok(())
// }


fn main() -> Result<(), Box<dyn Error>> {
    // loop {
    //     poll.poll(&mut events, None).expect("can not poll mio events");
    //     for event in events.iter() {
    //         if let Some(listener) = self.listeners.lock().unwrap().get(&event.token()) {
    //             spawn.spawn(self.add_peer(PeerSource::Incoming(listener.clone())).map(|_| ())).expect("can not add peer for incoming connection");
    //         } else {
    //             let pid = PeerId::new(self.chain_type, event.token());
    //             if let Err(error) = self.event_processor(event, pid, needed_services, iobuf.as_mut_slice()) {
    //                 self.ban(pid, 10);
    //             }
    //         }
    //     }
    //
    //
    // }
    let chain_type = ChainType::DevNet(DevnetType::Screwdriver);
    let chain = Chain::create_shared_devnet(DevnetType::Screwdriver);
    let shared_chain = Shared::RwLock(chain);
    const BACK_PRESSURE: usize = 10;
    let (to_dispatcher, from_p2p) = mpsc::sync_channel(BACK_PRESSURE);
    let (p2p, control_dispatcher) = P2P::new(DashP2PState::new(None, chain_type, shared_chain.clone()), P2PNotificationDispatcher::new(to_dispatcher),BACK_PRESSURE, chain_type, shared_chain.clone());
    // let timeout = Arc::new(Mutex::new(Timeout::new(control_dispatcher.clone())));

    let mut dispatcher = Dispatcher::new(from_p2p);

    // dispatcher.add_listener(HeaderDownload::new(chaindb.clone(), p2p_control.clone(), timeout.clone(), lightning.clone()));
    // dispatcher.add_listener(Ping::new(p2p_control.clone(), timeout.clone()));

    let mut executor = ThreadPoolBuilder::new()
        .name_prefix("dash-connect")
        .pool_size(2)
        .create()
        .expect("can not start futures thread pool");

    ["35.92.93.204:20001", "52.43.22.230:20001"].iter().for_each(|addr| {
        executor.spawn(p2p.lock().unwrap().add_peer(PeerSource::Outgoing(addr.parse().unwrap())).map(|_| ()))
            .expect("can not add peer for incoming connection");
    });

    // let mut keep_connected = KeepConnected {
    //     min_connections: 1,
    //     p2p: p2p.clone(),
    //     earlier: HashSet::new(),
    //     dns: chain_type.dns_seeds().iter().filter_map(|a| a.parse().ok()).collect(),
    //     cex: executor.clone()
    // };
    let dns = chain_type.dns_seeds().iter().filter_map(|a| a.parse().ok()).collect::<Vec<SocketAddr>>();
    let mut earlier = HashSet::new();
    let mut keep_p2p = p2p.clone();
    let ex = executor.clone();
    executor.spawn(async move {
        loop {
            // let _ = keep_connected.poll_unpin(&mut Context::from_waker(&noop_waker()));
            if let Ok(p2p) = keep_p2p.lock() {
                if p2p.connected_peers_count() < 1 {
                    let eligible = dns
                        .iter()
                        .cloned()
                        .filter(|a| !earlier.contains(a))
                        .collect::<Vec<_>>();
                    if !eligible.is_empty() {
                        let choice = eligible[(thread_rng().next_u32() as usize) % eligible.len()];
                        earlier.insert(choice.clone());
                        ex
                            .spawn(p2p.add_peer(PeerSource::Outgoing(choice)).map(|_| ()))
                            .expect("can not add peer for outgoing connection");
                    }
                }
            }

            thread::sleep(Duration::from_secs(10));
        }
    }).expect("can not keep connected");

    let mut p2p = p2p.clone();
    let mut cex = executor.clone();

    executor.spawn(future::poll_fn(move |_| {
        if let Ok(mut p2p) = p2p.lock() {
            p2p.poll_events(0, &mut cex);
        }
        Async::Ready(())
    })).expect("TODO: panic message");
    Ok(())

}

pub struct Dispatcher {
    listener: Arc<Mutex<Vec<P2PNotificationSender>>>
}

impl Dispatcher {
    pub fn new(incoming: P2PNotificationReceiver) -> Dispatcher {
        let listener = Arc::new(Mutex::new(Vec::new()));
        let l2 = listener.clone();
        thread::Builder::new().name("dispatcher".to_string()).spawn( move || { Self::incoming_messages_loop (incoming, l2) }).unwrap();
        Dispatcher { listener }
    }

    pub fn add_listener(&mut self, listener: P2PNotificationSender) {
        let mut list = self.listener.lock().unwrap();
        list.push(listener);
    }

    fn incoming_messages_loop(incoming: P2PNotificationReceiver, listener: Arc<Mutex<Vec<P2PNotificationSender>>>) {
        while let Ok(pm) = incoming.recv() {
            let list = listener.lock().unwrap();
            for listener in list.iter() {
                listener.send(pm.clone()).expect("Can't send notification");
            }
        }
        panic!("dispatcher failed");
    }
}

pub type SharedTimeout<Reply> = Arc<Mutex<Timeout<Reply>>>;

const TIMEOUT:u64 = 60;

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum ExpectedReply {
    Block,
    Headers,
    Pong,
    FilterHeader,
    FilterCheckpoints,
    Filter
}

pub struct Timeout<Reply: Eq + Hash + std::fmt::Debug> {
    timeouts: HashMap<PeerId, u64>,
    expected: HashMap<PeerId, HashMap<Reply, usize>>,
    p2p: P2PControlDispatcher
}

impl<Reply: Eq + Hash + std::fmt::Debug> Timeout<Reply> {
    pub fn new(p2p: P2PControlDispatcher) -> Timeout<Reply> {
        Timeout { p2p, timeouts: Default::default(), expected: Default::default() }
    }

    pub fn forget (&mut self, peer: PeerId) {
        self.timeouts.remove(&peer);
        self.expected.remove(&peer);
    }

    pub fn expect(&mut self, peer: PeerId, n: usize, what: Reply) {
        self.timeouts.insert(peer, Self::now() + TIMEOUT);
        *self.expected.entry(peer)
            .or_insert(HashMap::new())
            .entry(what)
            .or_insert(0) += n;
    }

    pub fn received(&mut self, peer: PeerId, n: usize, what: Reply) {
        if let Some(expected) = self.expected.get(&peer) {
            if let Some(m) = expected.get(&what) {
                if *m > 0 {
                    self.timeouts.insert(peer, Self::now() + TIMEOUT);
                }
            }
        }
        {
            let expected = self.expected.entry(peer).or_insert(HashMap::new()).entry(what).or_insert(n);
            *expected -= cmp::min(n, *expected);
        }
        if let Some(expected) = self.expected.get(&peer) {
            if expected.values().all(|v| *v == 0) {
                self.timeouts.remove(&peer);
            }
        }
    }

    pub fn is_busy(&self, peer: PeerId) -> bool {
        self.timeouts.contains_key(&peer)
    }

    pub fn is_busy_with(&self, peer: PeerId, what: Reply) -> bool {
        if self.timeouts.contains_key(&peer) {
            if let Some(expected) = self.expected.get(&peer) {
                if let Some(n) = expected.get(&what) {
                    return *n > 0;
                }
            }
        }
        false
        //
        // self.timeouts
        //     .get(&peer)
        //     .and_then(|expected| expected.get(&what))
        //     .map_or(false, |&n| n > 0)
    }


    pub fn check(&mut self, expected: Vec<Reply>) {
        // let banned: Vec<_> = self.timeouts.iter()
        //     .filter(|(peer, &timeout)| timeout < Self::now() &&
        //         expected.iter().any(|expected| self.expected.get(peer)
        //             .map_or(false, |e| e.get(expected)
        //                 .map_or(false, |n| *n > 0))))
        //     .map(|(peer, _)| {
        //         self.p2p.send(P2PControlNotification::Disconnect(*peer));
        //         *peer
        //     })
        //     .collect();
        //
        // for peer in &banned {
        //     self.timeouts.remove(peer);
        //     self.expected.remove(peer);
        // }

        self.timeouts.retain(|peer, timeout| {
            if *timeout < Self::now() &&
                expected.iter().any(|expected| self.expected.get(peer)
                    .map_or(false, |e| e.get(expected)
                        .map_or(false, |n| *n > 0))) {
                self.p2p.send(P2PControlNotification::Disconnect(*peer));
                self.expected.remove(peer);
                false
            } else {
                true
            }
        });
    }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}
//
// #[derive(Clone)]
// struct KeepConnected {
//     cex: ThreadPool,
//     dns: Vec<SocketAddr>,
//     earlier: HashSet<SocketAddr>,
//     p2p: Arc<Mutex<P2P<DashP2PState>>>,
//     min_connections: usize
// }
//
// impl Future for KeepConnected {
//     type Output = ();
//
//     fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Async<Self::Output> {
//         // if let Ok(p2p) = self.p2p.lock() {
//         //     if p2p.connected_peers_count() < self.min_connections {
//         //         let eligible = self.dns.iter().cloned().filter(|a| !self.earlier.contains(a)).collect::<Vec<_>>();
//         //         if eligible.len() > 0 {
//         //             let choice = eligible[(thread_rng().next_u32() as usize) % eligible.len()];
//         //             self.earlier.insert(choice.clone());
//         //             let add = p2p.add_peer(PeerSource::Outgoing(choice)).map(|_| ());
//         //             self.cex.spawn(add).expect("can not add peer for outgoing connection");
//         //         }
//         //     }
//         // }
//         if let Ok(p2p) = self.p2p.lock() {
//             if p2p.connected_peers_count() < self.min_connections {
//                 let eligible = self
//                     .dns
//                     .iter()
//                     .cloned()
//                     .filter(|a| !self.earlier.contains(a))
//                     .collect::<Vec<_>>();
//                 if !eligible.is_empty() {
//                     let choice = eligible[(thread_rng().next_u32() as usize) % eligible.len()];
//                     self.earlier.insert(choice.clone());
//                     let add = p2p.add_peer(PeerSource::Outgoing(choice)).map(|_| ());
//                     self.cex
//                         .spawn(add)
//                         .expect("can not add peer for outgoing connection");
//                 }
//             }
//         }
//
//         Async::Ready(())
//     }
// }
