use std::collections::HashSet;
use crate::chain::masternode::local_masternode::LocalMasternode;
use crate::chain::tx::ProviderRegistrationTransaction;
use crate::chain::wallet::wallet::Wallet;
use crate::crypto::{UInt160, UInt384};
use crate::keys::{BLSKey, ECDSAKey};

/// Wallet operations with masternodes (providers)
pub trait Masternodes {
    fn provider_owner_addresses(&self) -> HashSet<String>;
    fn provider_voting_addresses(&self) -> HashSet<String>;
    fn provider_operator_addresses(&self) -> HashSet<String>;

    fn unused_provider_owner_index(&self) -> u32;
    fn unused_provider_voting_index(&self) -> u32;
    fn unused_provider_operator_index(&self) -> u32;

    fn register_masternode_operator(&mut self, masternode: &LocalMasternode);
    fn register_masternode_operator_with_public_key(&mut self, masternode: &LocalMasternode, operator_key: &mut BLSKey);

    fn register_masternode_owner(&mut self, masternode: &LocalMasternode);
    fn register_masternode_owner_with_owner_private_key(&mut self, masternode: &LocalMasternode, owner_key: &mut ECDSAKey);

    fn register_masternode_voter(&mut self, masternode: &LocalMasternode);
    fn register_masternode_voter_with_voter_key(&mut self, masternode: &LocalMasternode, voting_key: &mut ECDSAKey);

    fn contains_provider_voting_authentication_hash(&self, hash: &UInt160) -> bool;
    fn contains_provider_owning_authentication_hash(&self, hash: &UInt160) -> bool;
    fn contains_provider_operator_authentication_key(&self, key: &UInt384) -> bool;
    fn contains_identity_bls_authentication_hash(&self, hash: &UInt160) -> bool;
    fn contains_holding_address(&self, address: &String) -> bool;

    fn index_of_provider_voting_authentication_hash(&self, hash: &UInt160) -> Option<u32>;
    fn index_of_provider_owning_authentication_hash(&self, hash: &UInt160) -> Option<u32>;
    fn index_of_provider_operator_authentication_key(&self, key: &UInt384) -> Option<u32>;
    fn index_of_holding_address(&self, address: &String) -> Option<u32>;

    fn can_authorize_provider_transaction(&self, transaction: &ProviderRegistrationTransaction) -> bool {
        self.index_of_provider_owning_authentication_hash(&transaction.owner_key_hash).is_some() ||
            self.index_of_provider_voting_authentication_hash(&transaction.voting_key_hash).is_some() ||
            self.index_of_provider_operator_authentication_key(&transaction.operator_key).is_some()
    }

    fn index_of_identity_authentication_hash(&self, hash: &UInt160) -> Option<u32>;
    fn index_of_identity_credit_funding_registration_hash(&self, hash: &UInt160) -> Option<u32>;
    fn index_of_identity_credit_funding_topup_hash(&self, hash: &UInt160) -> Option<u32>;
    fn index_of_identity_credit_funding_invitation_hash(&self, hash: &UInt160) -> Option<u32>;


}

impl Masternodes for Wallet {
    fn provider_owner_addresses(&self) -> HashSet<String> {
        todo!()
        // let mut derivation_path = self.chain.factory().provider_owner_keys_derivation_path_for_wallet(self);
        // if derivation_path.has_extended_public_key() {
        //     derivation_path.addresses_to_index_using_cache(self.unused_provider_owner_index() + 10, true, true)
        // } else {
        //     HashSet::new()
        // }
    }

    fn provider_voting_addresses(&self) -> HashSet<String> {
        todo!()
        // let mut derivation_path = self.chain.factory().provider_voting_keys_derivation_path_for_wallet(self);
        // if derivation_path.has_extended_public_key() {
        //     derivation_path.addresses_to_index_using_cache(self.unused_provider_voting_index() + 10, true, true)
        // } else {
        //     HashSet::new()
        // }
    }

    fn provider_operator_addresses(&self) -> HashSet<String> {
        todo!()
        // let mut derivation_path = self.chain.factory().provider_operator_keys_derivation_path_for_wallet(self);
        // if derivation_path.has_extended_public_key() {
        //     derivation_path.addresses_to_index_using_cache(self.unused_provider_operator_index() + 10, true, true)
        // } else {
        //     HashSet::new()
        // }
    }


    fn unused_provider_owner_index(&self) -> u32 {
        todo!()
        // self.masternode_owner_indexes.into_values().max().unwrap_or(0) + 1
    }

    fn unused_provider_voting_index(&self) -> u32 {
        todo!()
        // self.masternode_voter_indexes.into_values().max().unwrap_or(0) + 1
    }

    fn unused_provider_operator_index(&self) -> u32 {
        todo!()
        // self.masternode_operator_indexes.into_values().max().unwrap_or(0) + 1
    }


    fn register_masternode_operator(&mut self, masternode: &LocalMasternode) {
        todo!()
        // let registration_tx_hash = masternode.provider_registration_transaction.unwrap().tx_hash();
        // if !self.masternode_operator_indexes.contains_key(&registration_tx_hash) {
        //     self.masternode_operator_indexes.insert(registration_tx_hash, masternode.operator_wallet_index);
        //     let mut keychain_dictionary = Keychain::get_dict::<UInt256, u32>(self.wallet_masternode_operators_key()).unwrap_or(HashMap::new());
        //     keychain_dictionary.insert(registration_tx_hash.clone(), masternode.operator_wallet_index);
        //     Keychain::set_dict(keychain_dictionary, self.wallet_masternode_operators_key(), false)
        //         .expect("Can't save wallet masternode operators in keychain");
        // }
    }
    // TODO: check storage dictionary types as it can be u32 or UInt256 (migrate to struct)
    fn register_masternode_operator_with_public_key(&mut self, masternode: &LocalMasternode, operator_key: &mut BLSKey) {
        todo!()
        // let registration_tx_hash = masternode.provider_registration_transaction.unwrap().tx_hash();
        // if !self.masternode_operator_public_key_locations.contains_key(&registration_tx_hash) {
        //     let hashed_operator_key = UInt256::sha256(operator_key.public_key_data().as_slice());
        //     let operator_key_storage_location = format!("DS_OPERATOR_KEY_LOC_{}", hashed_operator_key.0.to_hex());
        //     self.masternode_operator_public_key_locations.insert(registration_tx_hash.clone(), operator_key_storage_location.clone());
        //     let mut keychain_dictionary = Keychain::get_dict::<UInt256, UInt256>(self.wallet_masternode_operators_key()).unwrap_or(HashMap::new());
        //     keychain_dictionary.insert(registration_tx_hash.clone(), hashed_operator_key);
        //     Keychain::set_dict(keychain_dictionary, self.wallet_masternode_operators_key(), false)
        //         .expect("Can't save hashed_operator_key in keychain");
        //     Keychain::set_data(operator_key_storage_location, Some(operator_key.public_key_data()), false)
        //         .expect("Can't save operator_key in keychain");
        // }
    }

    fn register_masternode_owner(&mut self, masternode: &LocalMasternode) {
        todo!()
        // let registration_tx_hash = masternode.provider_registration_transaction.unwrap().tx_hash();
        // if !self.masternode_owner_indexes.contains_key(&registration_tx_hash) &&
        //     masternode.owner_wallet_index != u32::MAX {
        //     self.masternode_owner_indexes.insert(registration_tx_hash.clone(), masternode.owner_wallet_index);
        //     let mut keychain_dictionary = Keychain::get_dict::<UInt256, u32>(self.wallet_masternode_owners_key()).unwrap_or(HashMap::new());
        //     keychain_dictionary.insert(registration_tx_hash.clone(), masternode.owner_wallet_index);
        //     Keychain::set_dict(keychain_dictionary, self.wallet_masternode_owners_key(), false)
        //         .expect("Can't save masternode owner_wallet_index in keychain");
        // }
    }

    fn register_masternode_owner_with_owner_private_key(&mut self, masternode: &LocalMasternode, owner_key: &mut ECDSAKey) {
        todo!()
        // let registration_tx_hash = masternode.provider_registration_transaction.unwrap().tx_hash();
        // if !self.masternode_owner_private_key_locations.contains_key(&registration_tx_hash) {
        //     let hashed_owner_key = UInt256::sha256(owner_key.public_key_data().as_slice());
        //     let owner_key_storage_location = format!("DS_OWNER_KEY_LOC_{}", hashed_owner_key.0.to_hex());
        //     self.masternode_owner_private_key_locations.insert(registration_tx_hash.clone(), owner_key_storage_location.clone());
        //     let mut keychain_dictionary = Keychain::get_dict::<UInt256, UInt256>(self.wallet_masternode_owners_key()).unwrap_or(HashMap::new());
        //     keychain_dictionary.insert(registration_tx_hash.clone(), hashed_owner_key);
        //     Keychain::set_dict(keychain_dictionary, self.wallet_masternode_owners_key(), false)
        //         .expect("Can't save hashed_owner_key in keychain");
        //     Keychain::set_data(owner_key_storage_location, owner_key.private_key_data(), false)
        //         .expect("Can't save owner_key in keychain");
        // }
    }

    fn register_masternode_voter(&mut self, masternode: &LocalMasternode) {
        todo!()
        // let registration_tx_hash = masternode.provider_registration_transaction.unwrap().tx_hash();
        // if !self.masternode_voter_indexes.contains_key(&registration_tx_hash) {
        //     self.masternode_voter_indexes.insert(registration_tx_hash.clone(), masternode.voting_wallet_index);
        //     let mut keychain_dictionary = Keychain::get_dict::<UInt256, u32>(self.wallet_masternode_voters_key()).unwrap_or(HashMap::new());
        //     keychain_dictionary.insert(registration_tx_hash.clone(), masternode.voting_wallet_index);
        //     Keychain::set_dict(keychain_dictionary, self.wallet_masternode_voters_key(), false)
        //         .expect("Can't save masternode voting_wallet_index in keychain");
        // }
    }

    fn register_masternode_voter_with_voter_key(&mut self, masternode: &LocalMasternode, voting_key: &mut ECDSAKey) {
        todo!()
        // let registration_tx_hash = masternode.provider_registration_transaction.unwrap().tx_hash();
        // if !self.masternode_voter_key_locations.contains_key(&registration_tx_hash) {
        //     let hashed_voter_key = UInt256::sha256(voting_key.public_key_data().as_slice());
        //     let voter_key_storage_location = format!("DS_VOTING_KEY_LOC_{}", hashed_voter_key.0.to_hex());
        //     self.masternode_voter_key_locations.insert(registration_tx_hash.clone(), voter_key_storage_location.clone());
        //     let mut keychain_dictionary = Keychain::get_dict::<UInt256, UInt256>(self.wallet_masternode_voters_key()).unwrap_or(HashMap::new());
        //     keychain_dictionary.insert(registration_tx_hash.clone(), hashed_voter_key);
        //     Keychain::set_dict(keychain_dictionary, self.wallet_masternode_voters_key(), false)
        //         .expect("Can't save hashed_owner_key in keychain");
        //     if voting_key.has_private_key() {
        //         Keychain::set_data(voter_key_storage_location, voting_key.private_key_data(), false)
        //             .expect("Can't save voting_key (private) in keychain");
        //     } else {
        //         Keychain::set_data(voter_key_storage_location, Some(voting_key.public_key_data().clone()), false)
        //             .expect("Can't save voting_key (public) in keychain");
        //     }
        // }
    }

    fn contains_provider_voting_authentication_hash(&self, hash: &UInt160) -> bool {
        todo!()
        // AuthenticationKeysDerivationPath::provider_voting_keys_derivation_path_for_wallet(self)
        //     .contains_address(&Address::from_hash160_for_script_map(hash, self.chain.script()))
    }

    fn contains_provider_owning_authentication_hash(&self, hash: &UInt160) -> bool {
        todo!()
        // AuthenticationKeysDerivationPath::provider_owner_keys_derivation_path_for_wallet(self)
        //     .contains_address(&Address::from_hash160_for_script_map(hash, self.chain.script()))
    }

    fn contains_provider_operator_authentication_key(&self, key: &UInt384) -> bool {
        todo!()
        // AuthenticationKeysDerivationPath::provider_operator_keys_derivation_path_for_wallet(self)
        //     .contains_address(&Address::from_hash160_for_script_map(&UInt160::hash160(&key.0), self.chain.script()))
    }

    fn contains_identity_bls_authentication_hash(&self, hash: &UInt160) -> bool {
        todo!()
        // AuthenticationKeysDerivationPath::identity_bls_keys_derivation_path_for_wallet(self)
        //     .contains_address(&Address::from_hash160_for_script_map(hash, self.chain.script()))
    }

    fn contains_holding_address(&self, address: &String) -> bool {
        todo!()
        // MasternodeHoldingsDerivationPath::provider_funds_derivation_path_for_wallet(self)
        //     .contains_address(address)
    }

    fn index_of_provider_voting_authentication_hash(&self, hash: &UInt160) -> Option<u32> {
        todo!()
        // AuthenticationKeysDerivationPath::provider_voting_keys_derivation_path_for_wallet(self)
        //     .index_of_known_address(&Address::from_hash160_for_script_map(hash, self.chain.script()))
    }

    fn index_of_provider_owning_authentication_hash(&self, hash: &UInt160) -> Option<u32> {
        todo!()
        // AuthenticationKeysDerivationPath::provider_owner_keys_derivation_path_for_wallet(self)
        //     .index_of_known_address_hash_for_chain(&hash, self.chain)
    }

    fn index_of_provider_operator_authentication_key(&self, key: &UInt384) -> Option<u32> {
        todo!()
        // AuthenticationKeysDerivationPath::provider_operator_keys_derivation_path_for_wallet(self)
        //     .index_of_known_address_hash_for_chain(&UInt160::hash160(&key.0), self.chain)
    }

    fn index_of_holding_address(&self, address: &String) -> Option<u32> {
        todo!()
        // self.chain.factory()
        //     .provider_funds_derivation_path_for_wallet(self)
        //     .index_of_known_address(address)
    }

    fn index_of_identity_authentication_hash(&self, hash: &UInt160) -> Option<u32> {
        todo!()
        // self.chain.factory()
        //     .identity_bls_keys_derivation_path_for_wallet(self)
        //     .index_of_known_address_hash_for_chain(hash, self.chain)
    }

    fn index_of_identity_credit_funding_registration_hash(&self, hash: &UInt160) -> Option<u32> {
        todo!()
        // self.chain.factory()
        //     .identity_registration_funding_derivation_path_for_wallet(self)
        //     .index_of_known_address_hash_for_chain(hash, self.chain)
    }

    fn index_of_identity_credit_funding_topup_hash(&self, hash: &UInt160) -> Option<u32> {
        todo!()
        // self.chain.factory()
        //     .identity_topup_funding_derivation_path_for_wallet(self)
        //     .index_of_known_address_hash_for_chain(hash, self.chain)
    }

    fn index_of_identity_credit_funding_invitation_hash(&self, hash: &UInt160) -> Option<u32> {
        todo!()
        // self.chain.factory()
        //     .identity_invitation_funding_derivation_path_for_wallet(self)
        //     .index_of_known_address_hash_for_chain(hash, self.chain)
    }
}
