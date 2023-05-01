use std::sync::{Arc, RwLock, Weak};
use crate::chain::{Chain, Wallet};
use crate::chain::common::chain_type::IHaveChainSettings;
use crate::chain::common::ChainType;
use crate::chain::derivation::authentication_keys_derivation_path::AuthenticationKeysDerivationPath;
use crate::chain::derivation::credit_funding_derivation_path::CreditFundingDerivationPath;
use crate::chain::derivation::masternode_holdings_derivation_path::MasternodeHoldingsDerivationPath;
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::wallet::seed::Seed;
use crate::storage::manager::managed_context::ManagedContext;

pub trait Derivation {
    fn register_derivation_paths_for_seed(&self, seed: &Seed, chain_type: ChainType, context: Weak<ManagedContext>);
    fn identity_registration_funding_derivation_path_for_wallet(&self, wallet: Weak<Wallet>) -> CreditFundingDerivationPath;
    fn identity_topup_funding_derivation_path_for_wallet(&self, wallet: Weak<Wallet>) -> CreditFundingDerivationPath;
    fn identity_invitation_funding_derivation_path_for_wallet(&self, wallet: Weak<Wallet>) -> CreditFundingDerivationPath;
    fn identity_bls_keys_derivation_path_for_wallet(&self, wallet: Weak<Wallet>) -> AuthenticationKeysDerivationPath;
    fn identity_ecdsa_keys_derivation_path_for_wallet(&self, wallet: Weak<Wallet>) -> AuthenticationKeysDerivationPath;

    // fn identity_funding_private_key_for_wallet(&self, wallet: Shared<Wallet>, coin_type: u32, is_for_invitation: bool, index: u32, seed: &Seed) -> Option<Key>;

    // fn identity_bls_keys_extended_public_key_for_wallet_from_seed(&self, coin_type: u32, wallet: &SharedWallet, seed: &Seed) -> Option<Key>;
    // fn identity_ecdsa_keys_extended_public_key_for_wallet_from_seed(&self, coin_type: u32, wallet: &SharedWallet, seed: &Seed) -> Option<Key>;
    // fn identity_registration_funding_extended_public_key_for_wallet_from_seed(&self, coin_type: u32, wallet: &SharedWallet, seed: &Seed) -> Option<Key>;
    // fn identity_topup_funding_extended_public_key_for_wallet_from_seed(&self, coin_type: u32, wallet: &SharedWallet, seed: &Seed) -> Option<Key>;
    // fn identity_invitation_funding_extended_public_key_for_wallet_from_seed(&self, coin_type: u32, wallet: &SharedWallet, seed: &Seed) -> Option<Key>;

    // fn standard_derivation_paths_for_account_number(&self, account_number: u32) -> Vec<DerivationPathKind>;
    // fn register_specialized_derivation_paths_for_seed(&self, seed: &Seed);

    // fn provider_operator_keys_derivation_path_for_wallet(&self, coin_type: u32, wallet: &SharedWallet) -> AuthenticationKeysDerivationPath;
    // fn provider_owner_keys_derivation_path_for_wallet(&self, coin_type: u32, wallet: &SharedWallet) -> AuthenticationKeysDerivationPath;
    // fn provider_voting_keys_derivation_path_for_wallet(&self, coin_type: u32, wallet: &SharedWallet) -> AuthenticationKeysDerivationPath;
    // fn provider_funds_derivation_path_for_wallet(&self, coin_type: u32, wallet: &SharedWallet) -> MasternodeHoldingsDerivationPath;
}

impl Derivation for Arc<RwLock<Chain>> {
    fn register_derivation_paths_for_seed(&self, seed: &Seed, chain_type: ChainType, context: Weak<ManagedContext>) {
        AuthenticationKeysDerivationPath::provider_owner_keys_derivation_path_for_chain(chain_type, context.clone())
            .generate_extended_public_key_from_seed(seed);
        AuthenticationKeysDerivationPath::provider_operator_keys_derivation_path_for_chain(chain_type, context.clone())
            .generate_extended_public_key_from_seed(seed);
        AuthenticationKeysDerivationPath::provider_voting_keys_derivation_path_for_chain(chain_type, context.clone())
            .generate_extended_public_key_from_seed(seed);
        MasternodeHoldingsDerivationPath::provider_funds_derivation_path_for_chain(chain_type, context.clone())
            .generate_extended_public_key_from_seed(seed);
        if chain_type.is_evolution_enabled() {
            AuthenticationKeysDerivationPath::identity_bls_keys_derivation_path_for_chain(chain_type, context.clone())
                .generate_extended_public_key_from_seed(seed);
            AuthenticationKeysDerivationPath::identity_ecdsa_keys_derivation_path_for_chain(chain_type, context.clone())
                .generate_extended_public_key_from_seed(seed);
            CreditFundingDerivationPath::identity_registration_funding_derivation_path_for_chain(chain_type, context.clone())
                .generate_extended_public_key_from_seed(seed);
            CreditFundingDerivationPath::identity_topup_funding_derivation_path_for_chain(chain_type, context.clone())
                .generate_extended_public_key_from_seed(seed);
            CreditFundingDerivationPath::identity_invitation_funding_derivation_path_for_chain(chain_type, context.clone())
                .generate_extended_public_key_from_seed(seed);
        }
    }

    fn identity_registration_funding_derivation_path_for_wallet(&self, wallet: Weak<Wallet>) -> CreditFundingDerivationPath {
        todo!()
        // self.with(|chain|
        //     chain.derivation_factory.identity_registration_funding_derivation_path_for_wallet(
        //         chain.r#type(), ))
    }

    fn identity_topup_funding_derivation_path_for_wallet(&self, wallet: Weak<Wallet>) -> CreditFundingDerivationPath {
        todo!()
    }

    fn identity_invitation_funding_derivation_path_for_wallet(&self, wallet: Weak<Wallet>) -> CreditFundingDerivationPath {
        todo!()
    }

    fn identity_bls_keys_derivation_path_for_wallet(&self, wallet: Weak<Wallet>) -> AuthenticationKeysDerivationPath {
        todo!()
    }

    fn identity_ecdsa_keys_derivation_path_for_wallet(&self, wallet: Weak<Wallet>) -> AuthenticationKeysDerivationPath {
        todo!()
    }

    // fn identity_funding_private_key_for_wallet(&self, wallet: &SharedWallet, coin_type: u32, is_for_invitation: bool, index: u32, seed: &Seed) -> Option<Key> {
    //     if is_for_invitation {
    //         self.identity_invitation_funding_derivation_path_for_wallet(coin_type, wallet)
    //     } else {
    //         self.identity_registration_funding_derivation_path_for_wallet(coin_type, wallet)
    //     }.private_key_at_index_path_from_seed(&IndexPath::index_path_with_index(index), seed)
    // }

    // fn identity_bls_keys_extended_public_key_for_wallet_from_seed(&self, coin_type: u32, wallet: &SharedWallet, seed: &Seed) -> Option<Key> {
    //     self.identity_bls_keys_derivation_path_for_wallet(coin_type, wallet)
    //         .generate_extended_public_key_from_seed(seed)
    // }
    //
    // fn identity_ecdsa_keys_extended_public_key_for_wallet_from_seed(&self, coin_type: u32, wallet: &SharedWallet, seed: &Seed) -> Option<Key> {
    //     self.identity_ecdsa_keys_derivation_path_for_wallet(coin_type, wallet)
    //         .generate_extended_public_key_from_seed(seed)
    // }
    // fn identity_registration_funding_extended_public_key_for_wallet_from_seed(&self, coin_type: u32, wallet: &SharedWallet, seed: &Seed) -> Option<Key> {
    //     self.identity_registration_funding_derivation_path_for_wallet(coin_type, wallet)
    //         .generate_extended_public_key_from_seed(seed)
    // }
    //
    // fn identity_topup_funding_extended_public_key_for_wallet_from_seed(&self, coin_type: u32, wallet: &SharedWallet, seed: &Seed) -> Option<Key> {
    //     self.identity_topup_funding_derivation_path_for_wallet(coin_type, wallet)
    //         .generate_extended_public_key_from_seed(seed)
    // }
    //
    // fn identity_invitation_funding_extended_public_key_for_wallet_from_seed(&self, coin_type: u32, wallet: &SharedWallet, seed: &Seed) -> Option<Key> {
    //     self.identity_invitation_funding_derivation_path_for_wallet(coin_type, wallet)
    //         .generate_extended_public_key_from_seed(seed)
    // }

}
