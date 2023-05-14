use std::sync::{Arc, Mutex, RwLock};
use crate::chain::network::message::Message;
use crate::network::p2p::{P2PControlNotification, P2PControlSender, PeerId, VersionCarrier};
use crate::network::p2p::peer_selector::PeerSelector;

#[derive(Clone)]
pub struct P2PControlDispatcher {
    sender: Arc<Mutex<P2PControlSender>>,
    peer_selector: Arc<RwLock<PeerSelector>>,
    pub back_pressure: usize
}

impl P2PControlDispatcher {
    pub fn new(sender: P2PControlSender, peer_selector: Arc<RwLock<PeerSelector>>, back_pressure: usize) -> P2PControlDispatcher {
        P2PControlDispatcher { sender: Arc::new(Mutex::new(sender)), peer_selector, back_pressure }
    }

    pub fn send(&self, notification: P2PControlNotification) {
        self.sender.lock()
            .unwrap()
            .send(notification)
            .expect("P2PControlDispatcher send failed");
    }

    pub fn send_network(&self, peer: PeerId, message: Message) {
        self.send(P2PControlNotification::Send(peer, message))
    }

    pub fn ban(&self, peer: PeerId, increment: u32) {
        self.send(P2PControlNotification::Ban(peer, increment))
    }

    pub fn peer_version(&self, pid: PeerId) -> Option<VersionCarrier> {
        self.peer_selector.read().unwrap().peer_version(pid)
    }

    pub fn peers(&self) -> Vec<PeerId> {
        self.peer_selector.read().unwrap().peer_ids()
    }
}
