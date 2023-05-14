use std::cmp;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::network::p2p::{P2PControlDispatcher, P2PControlNotification, PeerId};

pub type SharedTimeout<Reply> = Arc<Mutex<Timeout<Reply>>>;

const TIMEOUT:u64 = 60;

pub struct Timeout<Reply: Eq + std::hash::Hash + std::fmt::Debug> {
    timeouts: HashMap<PeerId, u64>,
    expected: HashMap<PeerId, HashMap<Reply, usize>>,
    p2p: P2PControlDispatcher
}

impl<Reply: Eq + std::hash::Hash + std::fmt::Debug> Timeout<Reply> {
    pub fn new(p2p: P2PControlDispatcher) -> Timeout<Reply> {
        Timeout { p2p, timeouts: Default::default(), expected: Default::default() }
    }

    pub fn forget(&mut self, peer: PeerId) {
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
        self.timeouts.get(&peer)
            .map_or(false, |_| self.is_expected(&peer, &what))

    }

    fn is_expected(&self, peer: &PeerId, what: &Reply) -> bool {
        self.expected.get(peer)
            .and_then(|exp| exp.get(what))
            .map(|n| *n > 0)
            .unwrap_or(false)
    }

    pub fn check(&mut self, expected: Vec<Reply>) {
        // let p2p = &mut self.p2p;
        // let exp = &mut self.expected;
        // self.timeouts.retain(|peer, timeout| {
        //     if *timeout < Self::now() && expected.iter().any(|what| self.is_expected(peer, what)) {
        //         p2p.send(P2PControlNotification::Disconnect(*peer));
        //         exp.remove(peer);
        //         false
        //     } else {
        //         true
        //     }
        // });
        let mut banned = Vec::new();
        for (peer, timeout) in &self.timeouts {
            if *timeout < Self::now () && expected.iter().any(|what| self.is_expected(peer, what)) {
                self.p2p.send(P2PControlNotification::Disconnect(*peer));
                banned.push(*peer);
            }
        }
        for peer in &banned {
            self.timeouts.remove(peer);
            self.expected.remove(peer);
        }

    }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}
