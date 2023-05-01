use std::sync::Arc;
use crate::chain::Chain;
use crate::crypto::UInt256;
use crate::platform::identity::identity::Identity;

pub trait Identities {
    fn identity_for_unique_id_in_wallet_including_foreign_identites(&self, unique_id: UInt256, include_foreign_blockchain_identities: bool) -> Option<Identity>;
}

impl Identities for Arc<Chain> {
    fn identity_for_unique_id_in_wallet_including_foreign_identites(&self, unique_id: UInt256, include_foreign_blockchain_identities: bool) -> Option<Identity> {
        todo!("impl")
        // assert!(!unique_id.is_zero(), "unique_id must not be null");
        // self.wallets.iter().find_map(|&wallet| if let Some(identity) = wallet.identity_for_unique_id(unique_id) {
        //     Some(identity)
        // } else {
        //     None
        // }).or({
        //     if include_foreign_blockchain_identities {
        //         self.identities_manager().foreign_blockchain_identity_with_unique_id(unique_id)
        //     } else {
        //         None
        //     }
        // })
    }
}
