use std::collections::HashSet;
use std::sync::Weak;
use crate::chain::common::ChainType;
use crate::chain::derivation::authentication_keys_derivation_path::AuthenticationKeysDerivationPath;
use crate::chain::derivation::credit_funding_derivation_path::CreditFundingDerivationPath;
use crate::chain::derivation::derivation_path::DerivationPath;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::funds_derivation_path::FundsDerivationPath;
use crate::chain::derivation::incoming_funds_derivation_path::IncomingFundsDerivationPath;
use crate::chain::derivation::index_path::IndexPath;
use crate::chain::derivation::masternode_holdings_derivation_path::MasternodeHoldingsDerivationPath;
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::derivation::simple_indexed_derivation_path::SimpleIndexedDerivationPath;
use crate::chain::wallet::seed::Seed;
use crate::keys::{Key, KeyKind};
use crate::storage::manager::managed_context::ManagedContext;

#[derive(Clone, Debug, PartialEq)]
pub enum DerivationPathKind {
    Default(DerivationPath),
    SimpleIndexed(SimpleIndexedDerivationPath),
    AuthenticationKeys(AuthenticationKeysDerivationPath),
    Funds(FundsDerivationPath),
    IncomingFunds(IncomingFundsDerivationPath),
    CreditFunding(CreditFundingDerivationPath),
    MasternodeHoldings(MasternodeHoldingsDerivationPath),
}

impl From<DerivationPath> for DerivationPathKind {
    fn from(value: DerivationPath) -> Self {
        DerivationPathKind::Default(value)
    }
}
impl From<SimpleIndexedDerivationPath> for DerivationPathKind {
    fn from(value: SimpleIndexedDerivationPath) -> Self {
        DerivationPathKind::SimpleIndexed(value)
    }
}
impl From<AuthenticationKeysDerivationPath> for DerivationPathKind {
    fn from(value: AuthenticationKeysDerivationPath) -> Self {
        DerivationPathKind::AuthenticationKeys(value)
    }
}
impl From<FundsDerivationPath> for DerivationPathKind {
    fn from(value: FundsDerivationPath) -> Self {
        DerivationPathKind::Funds(value)
    }
}
impl From<IncomingFundsDerivationPath> for DerivationPathKind {
    fn from(value: IncomingFundsDerivationPath) -> Self {
        DerivationPathKind::IncomingFunds(value)
    }
}
impl From<CreditFundingDerivationPath> for DerivationPathKind {
    fn from(value: CreditFundingDerivationPath) -> Self {
        DerivationPathKind::CreditFunding(value)
    }
}
impl From<MasternodeHoldingsDerivationPath> for DerivationPathKind {
    fn from(value: MasternodeHoldingsDerivationPath) -> Self {
        DerivationPathKind::MasternodeHoldings(value)
    }
}

impl DerivationPathKind {
    pub fn path_mut(&mut self) -> &mut dyn IDerivationPath {
        match self {
            DerivationPathKind::Default(path) => path,
            DerivationPathKind::SimpleIndexed(path) => path,
            DerivationPathKind::AuthenticationKeys(path) => path,
            DerivationPathKind::Funds(path) => path,
            DerivationPathKind::IncomingFunds(path) => path,
            DerivationPathKind::CreditFunding(path) => path,
            DerivationPathKind::MasternodeHoldings(path) => path
        }
    }
    pub fn path(&self) -> &dyn IDerivationPath {
        match self {
            DerivationPathKind::Default(path) => path,
            DerivationPathKind::SimpleIndexed(path) => path,
            DerivationPathKind::AuthenticationKeys(path) => path,
            DerivationPathKind::Funds(path) => path,
            DerivationPathKind::IncomingFunds(path) => path,
            DerivationPathKind::CreditFunding(path) => path,
            DerivationPathKind::MasternodeHoldings(path) => path
        }
    }
}

impl IDerivationPath for DerivationPathKind {

    fn chain_type(&self) -> ChainType {
        self.path().chain_type()
    }

    fn context(&self) -> Weak<ManagedContext> {
        self.path().context()
    }

    fn is_transient(&self) -> bool {
        self.path().is_transient()
    }

    fn set_is_transient(&mut self, is_transient: bool) {
        self.path_mut().set_is_transient(is_transient);
    }

    fn wallet_unique_id(&self) -> Option<String> {
        self.path().wallet_unique_id()
    }

    fn set_wallet_unique_id(&mut self, unique_id: String) {
        self.path_mut().set_wallet_unique_id(unique_id);
    }

    fn signing_algorithm(&self) -> KeyKind {
        self.path().signing_algorithm()
    }

    fn reference(&self) -> DerivationPathReference {
        self.path().reference()
    }

    fn extended_public_key(&self) -> Option<Key> {
        self.path().extended_public_key()
    }

    fn extended_public_key_mut(&mut self) -> Option<Key> {
        self.path_mut().extended_public_key_mut()
    }

    fn has_extended_public_key(&self) -> bool {
        self.path().has_extended_public_key()
    }

    fn depth(&self) -> u8 {
        self.path().depth()
    }

    fn all_addresses(&self) -> HashSet<String> {
        self.path().all_addresses()
    }

    fn used_addresses(&self) -> HashSet<String> {
        self.path().used_addresses()
    }

    fn standalone_extended_public_key_unique_id(&mut self) -> Option<String> {
        self.path_mut().standalone_extended_public_key_unique_id()
    }

    fn balance(&self) -> u64 {
        self.path().balance()
    }

    fn set_balance(&mut self, amount: u64) {
        self.path_mut().set_balance(amount);
    }

    fn index_path_for_known_address(&self, address: &String) -> Option<IndexPath<u32>> {
        self.path().index_path_for_known_address(address)
    }

    fn generate_extended_public_key_from_seed(&mut self, seed: &Seed) -> Option<Key> {
        self.path_mut().generate_extended_public_key_from_seed(seed)
    }

    fn register_transaction_address(&mut self, address: &String) -> bool {
        self.path_mut().register_transaction_address(address)
    }
}
