use std::collections::HashMap;
use std::time::Duration;
use secp256k1::rand::{RngCore, thread_rng};
use crate::chain::common::ChainType;
use crate::chain::network::message::message::{Message, MessagePayload};
use crate::chain::network::message::response::Response;
use crate::chain::network::Request;
use crate::network::p2p::{ExpectedReply, P2PControlDispatcher, P2PNotification, P2PNotificationDispatcher, P2PNotificationReceiver, PeerId, SharedTimeout};
use crate::network::pipeline::{Pipeline, ThreadName};

// ping peers every SECS seconds if not asked anything else in the meanwhile
const SECS: u64 = 60;

pub struct Ping {
    p2p: P2PControlDispatcher,
    timeout: SharedTimeout<ExpectedReply>,
    asked: HashMap<PeerId, u64>,
}

impl Pipeline for Ping {
    fn setup(self, back_pressure: usize, chain_type: ChainType) -> P2PNotificationDispatcher {
        self.setup_with(ThreadName::PipelinePing, back_pressure, chain_type)
    }

    fn run(&mut self, receiver: P2PNotificationReceiver) {
        loop {
            while let Ok(msg) = receiver.recv_timeout(Duration::from_millis(SECS*1000)) {
                match msg {
                    P2PNotification::Disconnected(pid,_) => {
                        self.timeout.lock().unwrap().forget(pid);
                        self.asked.remove(&pid);
                    },
                    P2PNotification::Incoming(pid, msg) => {
                        match msg {
                            Response::Pong(n) if self.asked.remove(&pid) == Some(n) => {
                                self.timeout.lock().unwrap().received(pid, 1, ExpectedReply::Pong);
                            },
                            Response::Ping(nonce) => {
                                let request = Request::Pong(nonce);
                                let message = Message::from(MessagePayload::Request(request));
                                self.p2p.send_network(pid, message);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            self.timeout.lock().unwrap().check(vec!(ExpectedReply::Pong));
            for peer in self.p2p.peers() {
                if !self.timeout.lock().unwrap().is_busy(peer) {
                    let ask = thread_rng().next_u64();
                    self.asked.insert(peer, ask);
                    self.timeout.lock().unwrap().expect(peer, 1, ExpectedReply::Pong);
                    let request = Request::Ping(ask);
                    let message = Message::from(MessagePayload::Request(request));
                    self.p2p.send_network(peer, message);
                }
            }
        }
    }
}

impl Ping {
    pub fn new(chain_type: ChainType, p2p: P2PControlDispatcher, timeout: SharedTimeout<ExpectedReply>) -> P2PNotificationDispatcher  {
        let back_pressure = p2p.back_pressure;
        let pipeline = Self { p2p, timeout, asked: HashMap::new() };
        pipeline.setup(back_pressure, chain_type)
    }
}
