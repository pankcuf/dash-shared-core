use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use crate::chain::network::message::{Message, Response};
use crate::network::p2p::{P2PNotification, P2PNotificationSender, PeerId};

#[derive(Clone)]
pub struct P2PNotificationDispatcher {
    sender: Arc<Mutex<P2PNotificationSender>>
}

impl P2PNotificationDispatcher {
    pub fn new(sender: P2PNotificationSender) -> P2PNotificationDispatcher {
        Self { sender: Arc::new(Mutex::new(sender)) }
    }

    pub fn send(&self, msg: P2PNotification) {
        self.sender.lock()
            .unwrap()
            .send(msg)
            .expect("P2P message send failed");
    }

    pub(crate) fn send_connected(&self, pid: PeerId, address: Option<SocketAddr>) {
        self.send(P2PNotification::Connected(pid, address));
    }

    pub(crate) fn send_disconnected(&self, pid: PeerId, banned: bool) {
        self.send(P2PNotification::Disconnected(pid, banned));
    }

    pub(crate) fn send_incoming(&self, pid: PeerId, response: Response) {
        self.send(P2PNotification::Incoming(pid, response));
    }

    pub(crate) fn send_outgoing(&self, message: Message) {
        self.send(P2PNotification::Outgoing(message));
    }
}
