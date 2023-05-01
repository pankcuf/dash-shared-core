use std::collections::HashSet;
use std::sync::Weak;
use crate::chain::ScriptMap;
use crate::chain::common::ChainType;
use crate::chain::derivation::derivation_path_feature_purpose::DerivationPathFeaturePurpose;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::derivation_path_type::DerivationPathType;
use crate::chain::derivation::index_path::{IIndexPath, IndexPath};
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::derivation::simple_indexed_derivation_path::{ISimpleIndexedDerivationPath, SimpleIndexedDerivationPath};
use crate::chain::derivation::uint256_index_path::UInt256IndexPath;
use crate::chain::wallet::seed::Seed;
use crate::crypto::{UInt160, UInt256};
use crate::keys::{Key, KeyKind};
use crate::storage::manager::managed_context::ManagedContext;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreditFundingDerivationPath {
    pub base: SimpleIndexedDerivationPath,
}

impl IIndexPath for CreditFundingDerivationPath {
    type Item = UInt256;
    fn new(indexes: Vec<Self::Item>) -> Self {
        Self { base: SimpleIndexedDerivationPath::new(indexes) }
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



impl IDerivationPath for CreditFundingDerivationPath {

    // fn chain(&self) -> Weak<Chain> {
    //     self.base.chain()
    // }

    fn chain_type(&self) -> ChainType {
        self.base.chain_type()
    }

    // fn wallet(&self) -> &Option<Weak<Wallet>> {
    //     self.base.wallet()
    // }
    //
    // fn set_wallet(&mut self, wallet: Weak<Wallet>) {
    //     self.base.set_wallet(wallet);
    // }

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
    // fn params(&self) -> &Params {
    //     self.base.params()
    // }
    //
    // fn wallet(&self) -> Weak<Wallet> {
    //     self.base.wallet()
    // }
    //
    // fn context(&self) -> Weak<ManagedContext> {
    //     self.base.context()
    // }

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
    //     DerivationPathKind::CreditFunding
    // }

    fn balance(&self) -> u64 {
        self.base.balance()
    }

    fn set_balance(&mut self, amount: u64) {
        self.base.set_balance(amount);
    }

    fn private_key_at_index_path_from_seed(&self, index_path: &IndexPath<u32>, seed: &Seed) -> Option<Key>
        where Self: Sized + IIndexPath + IDerivationPath<UInt256IndexPath> {
        self.base.private_key_at_index_path_from_seed(index_path, seed)
    }

    fn index_path_for_known_address(&self, address: &String) -> Option<IndexPath<u32>> {
        self.base.index_path_for_known_address(address)
    }

    fn generate_extended_public_key_from_seed(&mut self, seed: &Seed) -> Option<Key> {
        self.base.base.generate_extended_public_key_from_seed_and_store_private_key(seed, false)
    }

    fn register_transaction_address(&mut self, address: &String) -> bool {
        self.base.register_transaction_address(address)
    }
}

impl ISimpleIndexedDerivationPath for CreditFundingDerivationPath {

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

impl CreditFundingDerivationPath {
    fn identity_funding_derivation_path_for_chain(reference: DerivationPathReference, last_index: u32, chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self {
            base: SimpleIndexedDerivationPath::simple_indexed_derivation_path(
                    vec![
                        UInt256::from(DerivationPathFeaturePurpose::Default),
                        UInt256::from(chain_type.coin_type()),
                        UInt256::from(DerivationPathFeaturePurpose::Identities),
                        UInt256::from(last_index),
                    ],
                    vec![true, true, true, true],
                    DerivationPathType::CreditFunding,
                    KeyKind::ECDSA,
                    reference,
                    chain_type,
                    context
                ),
            ..Default::default()
        }
    }

    pub fn identity_registration_funding_derivation_path_for_chain(chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self::identity_funding_derivation_path_for_chain(
            DerivationPathReference::BlockchainIdentityCreditRegistrationFunding,
            u32::from(DerivationPathFeaturePurpose::IdentitiesSubfeatureRegistration),
            chain_type,
            context
        )
    }

    pub fn identity_topup_funding_derivation_path_for_chain(chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self::identity_funding_derivation_path_for_chain(
            DerivationPathReference::BlockchainIdentityCreditTopupFunding,
            u32::from(DerivationPathFeaturePurpose::IdentitiesSubfeatureTopup),
            chain_type,
            context
        )
    }

    pub fn identity_invitation_funding_derivation_path_for_chain(chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self::identity_funding_derivation_path_for_chain(
            DerivationPathReference::BlockchainIdentityCreditInvitationFunding,
            u32::from(DerivationPathFeaturePurpose::IdentitiesSubfeatureInvitations),
            chain_type,
            context
        )
    }

    pub fn identity_registration_funding_derivation_path_for_wallet(chain_type: ChainType, is_transient: bool, wallet_unique_id: String, context: Weak<ManagedContext>, load: bool) -> Self {
        let mut path = Self::identity_registration_funding_derivation_path_for_chain(chain_type, context);
        path.set_wallet_unique_id(wallet_unique_id);
        path.set_is_transient(is_transient);
        if load && path.has_extended_public_key() {
            path.load_addresses();
        }
        path
    }

    pub fn identity_topup_funding_derivation_path_for_wallet(chain_type: ChainType, is_transient: bool, wallet_unique_id: String, context: Weak<ManagedContext>, load: bool) -> Self {
        let mut path = Self::identity_topup_funding_derivation_path_for_chain(chain_type, context);
        path.set_wallet_unique_id(wallet_unique_id);
        path.set_is_transient(is_transient);
        if load && path.has_extended_public_key() {
            path.load_addresses();
        }
        path
    }

    pub fn identity_invitation_funding_derivation_path_for_wallet(chain_type: ChainType, is_transient: bool, wallet_unique_id: String, context: Weak<ManagedContext>, load: bool) -> Self {
        let mut path = Self::identity_invitation_funding_derivation_path_for_chain(chain_type, context);
        path.set_wallet_unique_id(wallet_unique_id);
        path.set_is_transient(is_transient);
        if load && path.has_extended_public_key() {
            path.load_addresses();
        }
        path
    }

}
