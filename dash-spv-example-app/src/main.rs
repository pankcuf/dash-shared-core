use std::error::Error;
use std::io::Write;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use log::debug;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
// use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
// use tokio::net::TcpStream;
use dash_spv_masternode_processor::chain::{Chain, common::ChainType};
use dash_spv_masternode_processor::hashes::hex::FromHex;
use dash_spv_masternode_processor::network::pipeline::manager::PipelineManager;
use dash_spv_masternode_processor::util::logging::setup_logger;
use dash_spv_masternode_processor::util::Shared;

extern crate simplelog;

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
// cargo run --package dash-spv-example-app --bin dash-spv-example-app -- -Z unstable-options --format=json --show-output --nocapture
fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let mut manager = PipelineManager::new(
        ChainType::TestNet,
        Shared::RwLock(Chain::create_shared_testnet()));
    manager.start().expect("Can't start pipeline manager");
    Ok(())
}

// fn main() -> Result<(), Box<dyn Error>> {
//     setup_logger();
//
//     let running = AtomicBool::new(false);
//     running.store(true, Ordering::SeqCst);
//     let p2p =
//     executor.spawn(future::poll_fn(move |_| {
//         debug!("poll_events...");
//         if let Ok(p2p) = p2p.try_read() {
//             p2p.poll_events(0, &mut cex);
//         }
//         Poll::Ready(())
//     }))?;
//     while running.load(Ordering::SeqCst) {
//         thread::sleep(Duration::from_millis(30));
//     }
//
//     Ok(())
// }

fn main2() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let mut stream = TcpStream::connect("85.209.243.60:19999".parse().unwrap())?;
    let poll = Arc::new(RwLock::new(Poll::new()?));
    let mut events = Events::with_capacity(128);
    let token = Token(0);
    poll.read().unwrap().registry().register(&mut stream, token, Interest::WRITABLE.add(Interest::READABLE))?;
    loop {
        poll.write().unwrap().poll(&mut events, Some(Duration::from_millis(100)))?;
        debug!("poll: {:#?}", events);
        for event in events.iter() {
            if event.is_writable() {
                let msg = Vec::from_hex("5312010000000000000000004007606400000000050000000000000000000000000000000000ffff55d1f34b4e1f000000000000000000000000000000000000ffff7f0000014e1f4edca9469d06be4e192f6461736877616c6c65743a312e3028746573746e6574292f0000000000").unwrap();
                stream.write_all(&msg)?;
            }
        }

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
