use std::fmt::Debug;
use crate::chain::Chain;
use crate::chain::network::Peer;
use crate::chain::tx::ProviderRegistrationTransaction;
use crate::chain::wallet::ext::masternodes::Masternodes;
use crate::util::Shared;

pub trait PeerMasternodeDelegate: Sync + Debug {
    fn peer_relayed_masternode_diff_message(&self, peer: &Peer, message: &[u8]);
    fn peer_relayed_quorum_rotation_info_message(&self, peer: &Peer, message: &[u8]);
}

impl PeerMasternodeDelegate for Shared<Chain> {
    fn peer_relayed_masternode_diff_message(&self, peer: &Peer, message: &[u8]) {
        todo!()
    }

    fn peer_relayed_quorum_rotation_info_message(&self, peer: &Peer, message: &[u8]) {
        todo!()
    }
}

pub trait TriggerUpdates {
    fn create_local_masternode_if_need(&self, transaction: &ProviderRegistrationTransaction);
}

impl TriggerUpdates for Shared<Chain> {

    fn create_local_masternode_if_need(&self, transaction: &ProviderRegistrationTransaction) {
        self.with(|chain| {
            // wallet has any type of provider keys?
            // let has_wallet_that_can_authorize_this_tx =
            if chain.wallets.iter().find(|w_lock| w_lock.can_authorize_provider_transaction(transaction)).is_some() {
                chain.masternode_manager.local_masternode_from_provider_registration_transaction(transaction, true);
            }
            // if chain.wallets.iter().find(|wallet| {
            //     match wallet.read() {
            //         Ok(ref w) => w.can_authorize_provider_transaction(transaction),
            //         _ => false
            //     }
            // }) {
            //     chain.masternode_manager.local_masternode_from_provider_registration_transaction(transaction, true);
            // }
        })
    }
}
