use std::collections::HashSet;
use std::sync::Weak;
use crate::chain::ScriptMap;
use crate::chain::common::ChainType;
use crate::chain::derivation::derivation_path_feature_purpose::DerivationPathFeaturePurpose;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::derivation_path_type::DerivationPathType;
use crate::chain::derivation::index_path::IndexPath;
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::derivation::simple_indexed_derivation_path::{ISimpleIndexedDerivationPath, SimpleIndexedDerivationPath};
use crate::chain::wallet::seed::Seed;
use crate::crypto::{UInt160, UInt256};
use crate::keys::{Key, KeyKind};
use crate::storage::manager::managed_context::ManagedContext;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MasternodeHoldingsDerivationPath {
    pub base: SimpleIndexedDerivationPath,
}

impl IDerivationPath for MasternodeHoldingsDerivationPath {

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

    fn balance(&self) -> u64 {
        self.base.balance()
    }

    fn set_balance(&mut self, amount: u64) {
        self.base.set_balance(amount);
    }

    fn index_path_for_known_address(&self, address: &String) -> Option<IndexPath<u32>> {
        self.base.index_path_for_known_address(address)
    }

    fn generate_extended_public_key_from_seed(&mut self, seed: &Seed) -> Option<Key> {
        self.base.generate_extended_public_key_from_seed(seed)
    }

    fn register_transaction_address(&mut self, address: &String) -> bool {
        self.base.register_transaction_address(address)
    }
}

impl ISimpleIndexedDerivationPath for MasternodeHoldingsDerivationPath {
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

    fn default_gap_limit(&self) -> u32 {
        5
    }
}

impl MasternodeHoldingsDerivationPath {
    pub fn provider_funds_derivation_path_for_chain(chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self {
            base: SimpleIndexedDerivationPath::simple_indexed_derivation_path(
                vec![
                    UInt256::from(DerivationPathFeaturePurpose::Default),
                    UInt256::from(chain_type.coin_type()),
                    UInt256::from(3u64),
                    UInt256::from(0u64),
                ],
                vec![true, true, true, true],
                DerivationPathType::ProtectedFunds,
                KeyKind::ECDSA,
                DerivationPathReference::ProviderFunds,
                chain_type,
                context
            )
        }
    }

    pub fn provider_funds_derivation_path_for_wallet(chain_type: ChainType, is_transient: bool, wallet_unique_id: String, context: Weak<ManagedContext>, load: bool) -> Self {
        let mut path = Self::provider_funds_derivation_path_for_chain(chain_type, context);
        path.set_wallet_unique_id(wallet_unique_id);
        path.set_is_transient(is_transient);
        if load && path.has_extended_public_key() {
            path.load_addresses();
        }
        path
    }

    pub fn receive_address(&mut self) -> Option<String> {
        self.register_addresses_with_gap_limit(1)
            .map_or(self.base.ordered_addresses.last().cloned(), |a| a.last().cloned())
    }
}
