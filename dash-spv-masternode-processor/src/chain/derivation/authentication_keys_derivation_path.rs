use std::collections::{HashMap, HashSet};
use std::sync::Weak;
use crate::chain::ScriptMap;
use crate::chain::common::ChainType;
use crate::chain::derivation::BIP32_HARD;
use crate::chain::derivation::derivation_path_feature_purpose::DerivationPathFeaturePurpose;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::derivation_path_type::DerivationPathType;
use crate::chain::derivation::index_path::{IIndexPath, IndexPath};
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::derivation::simple_indexed_derivation_path::{ISimpleIndexedDerivationPath, SimpleIndexedDerivationPath};
use crate::chain::wallet::seed::Seed;
use crate::crypto::{UInt160, UInt256};
use crate::keys::{IKey, Key, KeyKind};
use crate::storage::keychain::Keychain;
use crate::storage::manager::managed_context::ManagedContext;
use crate::util::address::address;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuthenticationKeysDerivationPath {
    pub base: SimpleIndexedDerivationPath,
    pub has_extended_private_key: bool,
    pub uses_hardened_keys: bool,

    should_store_extended_private_key: bool,
    addresses_by_identity: HashMap<u32, Vec<String>>,
}

impl IIndexPath for AuthenticationKeysDerivationPath {
    type Item = UInt256;
    fn new(indexes: Vec<Self::Item>) -> Self {
        Self { base: SimpleIndexedDerivationPath::new(indexes), ..Default::default() }
    }

    fn new_hardened(indexes: Vec<Self::Item>, hardened: Vec<bool>) -> Self {
        Self { base: SimpleIndexedDerivationPath::new_hardened(indexes, hardened), ..Default::default() }
    }

    fn indexes(&self) -> &Vec<Self::Item> {
        self.base.indexes()
    }
    fn hardened_indexes(&self) -> &Vec<bool> {
        self.base.hardened_indexes()
    }
}


impl IDerivationPath for AuthenticationKeysDerivationPath {
    fn chain_type(&self) -> ChainType {
        self.base.chain_type()
    }

    fn context(&self) -> Weak<ManagedContext> {
        self.base.context()
    }

    fn is_transient(&self) -> bool {
        self.base.is_transient()
    }

    fn set_is_transient(&mut self, is_transient: bool) {
        self.base.set_is_transient(is_transient);
    }

    fn wallet_unique_id(&self) -> Option<String> {
        self.base.wallet_unique_id()
    }

    fn set_wallet_unique_id(&mut self, unique_id: String) {
        self.base.set_wallet_unique_id(unique_id);
    }

    fn signing_algorithm(&self) -> KeyKind {
        self.base.signing_algorithm()
    }

    fn reference(&self) -> DerivationPathReference {
        self.base.reference()
    }

    fn extended_public_key(&self) -> Option<Key> {
        self.base.extended_public_key()
    }

    fn extended_public_key_mut(&mut self) -> Option<Key> {
        self.base.extended_public_key_mut()
    }

    fn has_extended_public_key(&self) -> bool {
        self.base.has_extended_public_key()
    }

    fn depth(&self) -> u8 {
        self.base.depth()
    }

    fn all_addresses(&self) -> HashSet<String> {
        self.base.all_addresses()
    }

    fn used_addresses(&self) -> HashSet<String> {
        self.base.used_addresses()
    }

    fn standalone_extended_public_key_unique_id(&mut self) -> Option<String> {
        self.base.standalone_extended_public_key_unique_id()
    }

    // fn kind(&self) -> DerivationPathKind {
    //     DerivationPathKind::AuthenticationKeys
    // }

    fn balance(&self) -> u64 {
        self.base.balance()
    }

    fn set_balance(&mut self, amount: u64) {
        self.base.set_balance(amount);
    }

    // fn private_key_at_index_path_from_seed<KEY: IKey>(&self, index_path: &IndexPath<u32>, seed: &Vec<u8>) -> Option<KEY> where Self: Sized {
    //     //if (!seed || !indexPath) return nil;
    //     if self.base.base.length() == 0 {
    //         // there needs to be at least 1 length
    //         return None;
    //     }
    //     if let Some(top_key) = self.signing_algorithm().key_with_seed_data(seed) {
    //         top_key.private_derive_to_256bit_derivation_path(self)?.private_derive_to_path(index_path)
    //     } else {
    //         assert!(false, "Top key should exist");
    //         None
    //     }
    // }

    fn public_key_data_at_index_path(&mut self, index_path: &IndexPath<u32>) -> Option<Vec<u8>> {
        let mut has_hardened_derivation = false;
        for i in 0..index_path.length() {
            let derivation = index_path.index_at_position(i);
            has_hardened_derivation |= derivation & BIP32_HARD > 0;
            if has_hardened_derivation {
                break;
            }
        }
        if has_hardened_derivation {
            if self.has_extended_private_key {
                self.private_key_at_index_path(index_path).map(|key| key.public_key_data())
            } else {
                None
            }
        } else {
            self.base.public_key_data_at_index_path(index_path)
        }
    }

    fn index_path_for_known_address(&self, address: &String) -> Option<IndexPath<u32>> {
        self.base.index_path_for_known_address(address)
    }

    fn generate_extended_public_key_from_seed(&mut self, seed: &Seed) -> Option<Key> {
        self.base.base.generate_extended_public_key_from_seed_and_store_private_key(seed, self.should_store_extended_private_key)
    }

    fn register_transaction_address(&mut self, address: &String) -> bool {
        self.base.register_transaction_address(address)
    }
}

impl ISimpleIndexedDerivationPath for AuthenticationKeysDerivationPath {
    fn base(&self) -> &dyn IDerivationPath {
        &self.base
    }

    fn addresses_to_index_using_cache(&mut self, index: u32, use_cache: bool, add_to_cache: bool) -> HashSet<String> {
        self.base.addresses_to_index_using_cache(index, use_cache, add_to_cache)
    }

    fn index_of_known_address(&self, address: &String) -> Option<u32> {
        self.base.index_of_known_address(address)
    }
    fn index_of_known_address_hash_for_script(&self, hash: &UInt160, script: &ScriptMap) -> Option<u32> {
        self.base.index_of_known_address_hash_for_script(hash, script)
    }

    fn public_key_data_at_index(&mut self, index: u32) -> Option<Vec<u8>> {
        self.base.public_key_data_at_index(index)
    }
}

impl AuthenticationKeysDerivationPath {

    pub fn provider_voting_keys_derivation_path_for_chain(chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self::provider_keys_derivation_path_for_chain(
            DerivationPathReference::ProviderVotingKeys,
            KeyKind::ECDSA,
            1,
            chain_type,
            context)
    }

    pub fn provider_owner_keys_derivation_path_for_chain(chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self::provider_keys_derivation_path_for_chain(
            DerivationPathReference::ProviderOwnerKeys,
            KeyKind::ECDSA,
            2,
            chain_type,
            context)
    }

    pub fn provider_operator_keys_derivation_path_for_chain(chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self::provider_keys_derivation_path_for_chain(
            DerivationPathReference::ProviderOperatorKeys,
            KeyKind::BLS,
            3,
            chain_type,
            context)
    }

    pub fn platform_node_keys_derivation_path_for_chain(chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self::provider_keys_derivation_path_for_chain(
            DerivationPathReference::PlatformNodeKeys,
            KeyKind::ED25519,
            3,
            chain_type,
            context)
    }

    pub fn identity_ecdsa_keys_derivation_path_for_chain(chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self::identity_keys_derivation_path_for_chain(KeyKind::ECDSA, 0, chain_type, context)
    }

    pub fn identity_bls_keys_derivation_path_for_chain(chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self::identity_keys_derivation_path_for_chain(KeyKind::BLS, 1, chain_type, context)
    }

    pub fn provider_voting_keys_derivation_path_for_wallet(chain_type: ChainType, is_transient: bool, wallet_unique_id: String, context: Weak<ManagedContext>, load: bool) -> Self {
        let mut path = AuthenticationKeysDerivationPath::provider_voting_keys_derivation_path_for_chain(chain_type, context);
        path.set_wallet_unique_id(wallet_unique_id);
        path.set_is_transient(is_transient);
        if load && path.has_extended_public_key() {
            path.load_addresses();
        }
        path
    }

    pub fn provider_owner_keys_derivation_path_for_wallet(chain_type: ChainType, is_transient: bool, wallet_unique_id: String, context: Weak<ManagedContext>, load: bool) -> Self {
        let mut path = AuthenticationKeysDerivationPath::provider_owner_keys_derivation_path_for_chain(chain_type, context);
        path.set_wallet_unique_id(wallet_unique_id);
        path.set_is_transient(is_transient);
        if load && path.has_extended_public_key() {
            path.load_addresses();
        }
        path
    }

    pub fn provider_operator_keys_derivation_path_for_wallet(chain_type: ChainType, is_transient: bool, wallet_unique_id: String, context: Weak<ManagedContext>, load: bool) -> Self {
        let mut path = AuthenticationKeysDerivationPath::provider_operator_keys_derivation_path_for_chain(chain_type, context);
        path.set_wallet_unique_id(wallet_unique_id);
        path.set_is_transient(is_transient);
        if load && path.has_extended_public_key() {
            path.load_addresses();
        }
        path
    }

    pub fn platform_node_keys_derivation_path_for_wallet(chain_type: ChainType, is_transient: bool, wallet_unique_id: String, context: Weak<ManagedContext>, load: bool) -> Self {
        let mut path = AuthenticationKeysDerivationPath::platform_node_keys_derivation_path_for_chain(chain_type, context);
        path.set_wallet_unique_id(wallet_unique_id);
        path.set_is_transient(is_transient);
        if load && path.has_extended_public_key() {
            path.load_addresses();
        }
        path
    }

    pub fn identity_bls_keys_derivation_path_for_wallet(chain_type: ChainType, is_transient: bool, wallet_unique_id: String, context: Weak<ManagedContext>, load: bool) -> Self {
        let mut path = AuthenticationKeysDerivationPath::identity_bls_keys_derivation_path_for_chain(chain_type, context);
        path.set_wallet_unique_id(wallet_unique_id);
        path.set_is_transient(is_transient);
        if load && (path.has_extended_public_key() || (path.has_extended_public_key() && !path.uses_hardened_keys)) {
            path.load_addresses();
        }
        path
    }

    pub fn identity_ecdsa_keys_derivation_path_for_wallet(chain_type: ChainType, is_transient: bool, wallet_unique_id: String, context: Weak<ManagedContext>, load: bool) -> Self {
        let mut path = AuthenticationKeysDerivationPath::identity_ecdsa_keys_derivation_path_for_chain(chain_type, context);
        path.set_wallet_unique_id(wallet_unique_id);
        path.set_is_transient(is_transient);
        if load && (path.has_extended_public_key() || (path.has_extended_public_key() && !path.uses_hardened_keys)) {
            path.load_addresses();
        }
        path
    }

    pub fn first_unused_public_key(&mut self) -> Option<Vec<u8>> {
        self.public_key_data_at_index(self.base.first_unused_index())
    }

    pub fn first_unused_private_key_from_seed(&mut self, seed: Seed) -> Option<Key> {
        self.private_key_at_index_path_from_seed(&IndexPath::index_path_with_index(self.base.first_unused_index()), &seed)
    }

    pub fn private_key_for_hash160(&self, hash: UInt160, seed: Seed) -> Option<Key> {
        self.index_of_known_address(&address::from_hash160_for_script_map(&hash, &self.chain_type().script_map()))
            .and_then(|pos| self.private_key_at_index_path_from_seed(&IndexPath::index_path_with_index(pos as u32), &seed))
    }

    pub fn public_key_data_for_hash160(&mut self, hash: UInt160) -> Option<Vec<u8>> {
        self.index_of_known_address(&address::from_hash160_for_script_map(&hash, &self.chain_type().script_map()))
            .and_then(|pos| self.public_key_data_at_index(pos as u32))
    }

    fn extended_private_key_data(&mut self) -> Option<Vec<u8>> {
        Keychain::get_data(self.base.base.wallet_based_extended_private_key_location_string()).ok()
    }

    pub fn private_key_at_index_path(&mut self, index_path: &IndexPath<u32>) -> Option<Key> {
        self.extended_private_key_data()
            .and_then(|data| self.signing_algorithm().private_key_from_extended_private_key_data(&data)
                .and_then(|key| key.private_derive_to_path(index_path)))
    }

    fn keys_derivation_path_for_chain(
        indexes: Vec<UInt256>,
        hardened: Vec<bool>,
        r#type: DerivationPathType,
        signing_algorithm: KeyKind,
        reference: DerivationPathReference,
        should_store_extended_private_key: bool,
        uses_hardened_keys: bool,
        chain_type: ChainType,
        context: Weak<ManagedContext>) -> Self {
        Self {
            base: SimpleIndexedDerivationPath::simple_indexed_derivation_path(indexes, hardened, r#type, signing_algorithm, reference, chain_type, context),
            should_store_extended_private_key,
            uses_hardened_keys,
            ..Default::default()
        }
    }

    fn provider_keys_derivation_path_for_chain(
        reference: DerivationPathReference,
        signing_algorithm: KeyKind,
        last_index: u32,
        chain_type: ChainType,
        context: Weak<ManagedContext>) -> Self {
        Self::keys_derivation_path_for_chain(
            vec![
                UInt256::from(DerivationPathFeaturePurpose::Default),
                UInt256::from(chain_type.coin_type()),
                UInt256::from(3u32),
                UInt256::from(last_index)
            ],
            vec![true, true, true, true],
            DerivationPathType::SingleUserAuthentication,
            signing_algorithm,
            reference,
            false,
            false,
            chain_type,
            context,
        )
    }

    fn identity_keys_derivation_path_for_chain(signing_algorithm: KeyKind, last_index: u32, chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self::keys_derivation_path_for_chain(
            vec![
                UInt256::from(DerivationPathFeaturePurpose::Default),
                UInt256::from(chain_type.coin_type()),
                UInt256::from(DerivationPathFeaturePurpose::Identities),
                UInt256::from(DerivationPathFeaturePurpose::IdentitiesSubfeatureAuthentication),
                UInt256::from(last_index)
            ],
            vec![true, true, true, true, true],
            DerivationPathType::MultipleUserAuthentication,
            signing_algorithm,
            DerivationPathReference::BlockchainIdentities,
            true,
            true,
            chain_type,
            context,
        )
    }

}
