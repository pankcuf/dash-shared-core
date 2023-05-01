use crate::chain::{Chain, SyncCountInfo};
use crate::chain::ext::Settings;
use crate::chain::network::Peer;
use crate::chain::network::message::addr::Addr;
use crate::util::Shared;

pub trait PeerChainDelegate {
    fn peer_relayed_sync_info(&self, peer: &Peer, sync_count_info: &SyncCountInfo, count: u32);
    fn peer_relayed_peers(&self, peer: &Peer, addr: Addr);
    fn chain_finished_syncing_transactions_and_blocks(&mut self, peer: Option<&Peer>, on_main_chain: bool);
}

impl PeerChainDelegate for Shared<Chain> {
    fn peer_relayed_sync_info(&self, peer: &Peer, sync_count_info: &SyncCountInfo, count: u32) {
        // self.with(|mut chain| chain.peer_manager)
    }

    fn peer_relayed_peers(&self, peer: &Peer, addr: Addr) {
        let chain = self.clone();
        self.with(|locked| {
            let chain_type = locked.r#type();
            let peers = addr.addresses.into_iter()
                .map(|addr_info| Peer::init_with_addr_info(addr_info, chain_type, self.clone()))
                .collect();
            locked.peer_manager.peers_relayed(peer, peers)
        });
    }

    fn chain_finished_syncing_transactions_and_blocks(&mut self, peer: Option<&Peer>, on_main_chain: bool) {
        todo!()
    }
}

