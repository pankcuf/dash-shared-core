use std::sync::{Arc, mpsc, Mutex};
use std::{future, thread};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;
use futures::{FutureExt, task::{Poll, SpawnExt, SpawnError}};
use crate::chain::Chain;
use crate::chain::common::ChainType;
use crate::network::p2p::{P2P, PeerSource, Timeout};
use crate::network::pipeline::{Headers, Ping, PipelineDispatcher, ThreadName};
use crate::util::Shared;

const BACK_PRESSURE: usize = 10;

pub struct PipelineManager {
    running: Arc<AtomicBool>,
    chain_type: ChainType,
    chain: Shared<Chain>,
    handle: Option<JoinHandle<()>>,
}

impl PipelineManager {
    pub fn new(chain_type: ChainType, chain: Shared<Chain>) -> Self {
        Self { chain_type, chain, running: Arc::new(AtomicBool::new(false)), handle: None }
    }

    pub fn start(&mut self) -> Result<(), SpawnError> {
        let (notification_sender, notification_receiver) = mpsc::sync_channel(BACK_PRESSURE);
        let (p2p, control_dispatcher, addresses) = P2P::new(notification_sender, BACK_PRESSURE, self.chain_type, self.chain.clone());
        let timeout = Arc::new(Mutex::new(Timeout::new(control_dispatcher.clone())));
        let pipeline_dispatcher = PipelineDispatcher::new(self.chain_type, notification_receiver, vec![
            Headers::new(self.chain.clone(), self.chain_type, control_dispatcher.clone(), timeout.clone()),
            Ping::new(self.chain_type, control_dispatcher.clone(), timeout.clone())
        ]);
        let executor = ThreadName::P2PConnect.pool(self.chain_type, 3)
            .expect("can not start futures thread pool");

        debug!("start...");
        // executor.spawn(p2p.add_peer(PeerSource::Outgoing(choice)).map(|_| ())).expect("can not add peer for incoming connection");

        // executor.spawn(future::poll_fn(move |_| {
        //     debug!("poll_events...");
        //     if let Ok(p2p) = p2p.try_read() {
        //         p2p.
        //     }
        //     p2p.try_read().
        //         p2p.poll_events(0, &mut cex);
        //     }
        //     Poll::Ready(())
        // }))?;
        // addresses.iter().for_each(|addr| {
        //     executor.spawn(p2p.try_read().unwrap().add_peer(PeerSource::Outgoing(addr.parse().unwrap())).map(|_| ()))
        //         .expect("can not add peer for incoming connection");
        // });
        // ["35.92.93.204:20001", "52.43.22.230:20001"].iter().for_each(|addr| {
        //     executor.spawn(p2p.try_read().unwrap().add_peer(PeerSource::Outgoing(addr.parse().unwrap())).map(|_| ()))
        //         .expect("can not add peer for incoming connection");
        // });
        // debug!("peers added...");

        // let p2p_sources = p2p.clone();
        addresses.iter().for_each(|addr| {
            executor.spawn(p2p.add_peer(PeerSource::Outgoing(*addr)).map(|_| ()))
                .expect("can not add peer for outgoing connection");
        });

        // let add_peer_futures: Vec<_> = addresses.iter().map(|addr| {
        //     executor.spawn_with_handle(p2p.add_peer(PeerSource::Outgoing(*addr)).map(|_| ()))
        // }).collect();



        // let dns = self.chain_type.dns_seeds().iter().filter_map(|a| a.parse().ok()).collect::<Vec<SocketAddr>>();
        // let mut earlier = HashSet::new();
        // let p2p_keep_alive = p2p.clone();
        // let mut exec = executor.clone();
        // executor.spawn(Interval::new(Duration::new(10, 0)).for_each(move |_| keep_connected.clone())).expect("can not keep connected");
        // executor.spawn(async move {
        //     loop {
        //         debug!("keep connected...");
        //         // let _ = keep_connected.poll_unpin(&mut Context::from_waker(&noop_waker()));
        //         if let Ok(p2p) = p2p_keep_alive.try_read() {
        //             debug!("connected peers: {}", p2p.connected_peers_count());
        //             p2p.select_more_peers(&mut exec);
        //         }
        //         thread::sleep(Duration::from_secs(30));
        //     }
        // })?;
        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);

        // let add_peer_future_iterator = addresses.iter().map(|addr| executor.spawn_with_handle(p2p.add_peer(PeerSource::Outgoing(*addr)).map(|_| ())).unwrap()).collect::<Vec<_>>();
        // let add_peers_future = futures::future::join_all(add_peer_future_iterator);
        debug!("add_peers_future...");
        let mut cex = executor.clone();
        let poll_future = future::poll_fn(move |_| {
            debug!("poll_events ->");
            p2p.poll_events(0, &mut cex);
            Poll::Ready(())
        });
        // let poller = add_peers_future.then(|_| poll_future);
        debug!("spawn poller ->");
        executor.spawn(poll_future)?;
        debug!("poller spawned");
        // executor.spawn(future::poll_fn(move |_| {
        //     debug!("poll_events...");
        //     // if let Ok(p2p) = p2p.try_read() {
        //         p2p.poll_events(0, &mut cex);
        //     // }
        //     Poll::Ready(())
        // }))?;
        while running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(30));
        }
        debug!("exit pipeline manager start...");
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.running.store(false, Ordering::SeqCst);
            handle.join().unwrap();
        }
    }
}
