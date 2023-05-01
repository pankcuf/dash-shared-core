use std::fmt::Debug;
use crate::chain::network::Peer;

pub trait PeerChainLocksDelegate: Sync + Debug {
    fn peer_relayed_masternode_diff_message(&self, peer: &Peer, message: &[u8]);
    fn peer_relayed_quorum_rotation_info_message(&self, peer: &Peer, message: &[u8]);
}
