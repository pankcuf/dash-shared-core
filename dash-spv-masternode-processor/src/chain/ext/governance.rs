use std::fmt::Debug;
use crate::chain::{Chain, governance};
use crate::chain::governance::{Object, Vote};
use crate::chain::network::{GovernanceRequestState, Peer};
use crate::crypto::UInt256;
use crate::util::Shared;

pub trait PeerGovernanceDelegate: Send + Sync + Debug + Default where Self: Sized {
    fn peer_requested_object(&self, peer: &Peer, object_hash: &UInt256) -> Option<governance::Object>;
    fn peer_requested_vote(&self, peer: &Peer, vote_hash: &UInt256) -> Option<governance::Vote>;
    fn peer_has_governance_object_hashes(&self, peer: &Peer, hashes: Vec<UInt256>);
    fn peer_has_governance_vote_hashes(&self, peer: &Peer, hashes: Vec<UInt256>);
    fn peer_relayed_governance_object(&self, peer: &Peer, object: governance::Object);
    fn peer_relayed_governance_vote(&self, peer: &Peer, vote: governance::Vote);
    fn peer_ignored_governance_sync(&self, peer: &Peer, state: GovernanceRequestState);
}

impl PeerGovernanceDelegate for Shared<Chain> {
    fn peer_requested_object(&self, peer: &Peer, object_hash: &UInt256) -> Option<Object> {
        todo!()
    }

    fn peer_requested_vote(&self, peer: &Peer, vote_hash: &UInt256) -> Option<Vote> {
        todo!()
    }

    fn peer_has_governance_object_hashes(&self, peer: &Peer, hashes: Vec<UInt256>) {
        todo!()
    }

    fn peer_has_governance_vote_hashes(&self, peer: &Peer, hashes: Vec<UInt256>) {
        todo!()
    }

    fn peer_relayed_governance_object(&self, peer: &Peer, object: Object) {
        todo!()
    }

    fn peer_relayed_governance_vote(&self, peer: &Peer, vote: Vote) {
        todo!()
    }

    fn peer_ignored_governance_sync(&self, peer: &Peer, state: GovernanceRequestState) {
        todo!()
    }
}
