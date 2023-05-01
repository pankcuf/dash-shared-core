use crate::chain::Chain;
use crate::chain::network::Peer;
use crate::chain::spork::manager::PeerSporkDelegate;
use crate::chain::spork::Spork;
use crate::crypto::UInt256;
use crate::util::Shared;

impl PeerSporkDelegate for Shared<Chain> {
    fn peer_relayed_spork(&self, peer: &mut Peer, spork: Spork) {
        self.with(|chain| {
                if !spork.is_valid {
                    chain.peer_manager.peer_misbehaving(peer, format!("Spork is not valid"));
                    return;
                }
                if chain.spork_manager.update_with_spork(spork) {
                    // if current_spork.is_none() || updated {
                    //
                    /*chain.chain_context().perform_block_and_wait(|context| {
                        // todo: think maybe it's better to store spork hashes separately
                        SporkEntity::update_with_spork(&spork, spork.calculate_spork_hash(), context)
                            .expect("Can't update spork entity");
                    });
                    DispatchContext::main_context().queue(|| NotificationCenter::post(Notification::SporkListDidUpdate {
                        chain: self.chain,
                        old: updated_spork,
                        new: Some(&spork),
                    }));*/
                }
        })
    }

    fn peer_has_spork_hashes(&self, peer: &Peer, hashes: Vec<UInt256>) {
        self.with(|chain| chain.spork_manager.update_with_spork_hashes(peer, hashes))
    }
}
