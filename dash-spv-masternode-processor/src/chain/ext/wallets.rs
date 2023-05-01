use std::sync::{Arc, RwLock};
use bip0039::Language;
use crate::chain::{Chain, Wallet};
use crate::chain::common::ChainType;
use crate::chain::ext::{Derivation, Storage};
use crate::chain::tx::ProviderRegistrationTransaction;
use crate::chain::wallet::seed::Seed;
use crate::crypto::{UInt160, UInt384};
use crate::storage::keychain::Keychain;
use crate::util::Shared;

pub trait Wallets {
    /// Merging Wallets
    fn wallet_having_identity_credit_funding_registration_hash(&self, credit_funding_registration_hash: &UInt160) -> Option<(&Wallet, u32)>;
    fn wallet_having_identity_credit_funding_topup_hash(&self, credit_funding_topup_hash: &UInt160) -> Option<(&Wallet, u32)>;
    fn wallet_having_identity_credit_funding_invitation_hash(&self, credit_funding_invitation_hash: &UInt160) -> Option<(&Wallet, u32)>;
    fn wallet_having_provider_voting_authentication_hash(&self, voting_authentication_hash: &UInt160) -> Option<(&Wallet, u32)>;
    fn wallet_having_provider_owner_authentication_hash(&self, owner_authentication_hash: &UInt160) -> Option<(&Wallet, u32)>;
    fn wallet_having_provider_operator_authentication_key(&self, key: &UInt384) -> Option<(&Wallet, u32)>;
    fn wallet_containing_masternode_holding_address_for_provider_registration_transaction(&self, transaction: &ProviderRegistrationTransaction) -> Option<(&Wallet, u32)>;

    // fn has_wallet_with_unique_id(&self, unique_id: &String) -> bool;
}

pub trait WalletCreation {
    fn transient_wallet_with_seed(&self, seed: Seed, chain_type: ChainType);
    fn wallet_with_seed(&self, seed: Seed, is_transient: bool, chain_type: ChainType);
    fn wallet_with_seed_phrase<L: Language>(&self, seed_phrase: &str, is_transient: bool, created_at: u64, chain_type: ChainType);
    fn register_wallet(&mut self, wallet: Arc<RwLock<Wallet>>);
}

impl Wallets for Shared<Chain> {
    fn wallet_having_identity_credit_funding_registration_hash(&self, hash: &UInt160) -> Option<(&Wallet, u32)> {
        todo!()
        // self.with(|chain| chain.wallets.with(|wallets| wallets
        //     .iter()
        //     .find_map(move |wallet|
        //         wallet.index_of_identity_credit_funding_registration_hash(hash)
        //             .map(|index| (wallet, index)))))
    }

    fn wallet_having_identity_credit_funding_topup_hash(&self, hash: &UInt160) -> Option<(&Wallet, u32)> {
        todo!()
        // self.with(|chain| chain.wallets.with(|wallets| wallets
        //     .iter()
        //     .find_map(move |wallet|
        //         wallet.index_of_identity_credit_funding_topup_hash(hash)
        //             .map(|index| (wallet, index)))))
    }

    fn wallet_having_identity_credit_funding_invitation_hash(&self, hash: &UInt160) -> Option<(&Wallet, u32)> {
        todo!()
        // self.with(|chain| chain.wallets.with(|wallets| wallets
        //     .iter()
        //     .find_map(move |wallet|
        //         wallet.index_of_identity_credit_funding_invitation_hash(hash)
        //             .map(|index| (wallet, index)))))
    }

    fn wallet_having_provider_voting_authentication_hash(&self, hash: &UInt160) -> Option<(&Wallet, u32)> {
        todo!()
        // self.with(|chain| chain.wallets.with(|wallets| wallets
        //     .iter()
        //     .find_map(move |wallet|
        //         wallet.index_of_provider_voting_authentication_hash(hash)
        //             .map(|index| (wallet, index)))))
    }

    fn wallet_having_provider_owner_authentication_hash(&self, hash: &UInt160) -> Option<(&Wallet, u32)> {
        todo!()
        // self.with(|chain| chain.wallets.with(|wallets| wallets
        //     .iter()
        //     .find_map(move |wallet|
        //         wallet.index_of_provider_owning_authentication_hash(hash)
        //             .map(|index| (wallet, index)))))
    }

    fn wallet_having_provider_operator_authentication_key(&self, key: &UInt384) -> Option<(&Wallet, u32)> {
        todo!()
        // self.with(|chain| chain.wallets.with(|wallets| wallets
        //     .iter()
        //     .find_map(move |wallet|
        //         wallet.index_of_provider_operator_authentication_key(key)
        //             .map(|index| (wallet, index)))))
    }

    fn wallet_containing_masternode_holding_address_for_provider_registration_transaction(&self, transaction: &ProviderRegistrationTransaction) -> Option<(&Wallet, u32)> {
        todo!()
        // self.with(|chain| chain.wallets.with(|wallets| wallets
        //     .iter()
        //     .find_map(move |wallet|
        //         transaction.outputs()
        //             .iter()
        //             .find_map(|output|
        //                 output.address.and_then(|a| wallet.index_of_holding_address(&a))
        //                     .map(|index| (wallet, index))))))
    }

}


impl WalletCreation for Arc<RwLock<Chain>> {

    fn transient_wallet_with_seed(&self, seed: Seed, chain_type: ChainType) {
        self.wallet_with_seed(seed, true, chain_type)
    }

    fn wallet_with_seed(&self, seed: Seed, is_transient: bool, chain_type: ChainType) {
        let context = Arc::downgrade(self.try_read().unwrap().chain_context());
        self.register_derivation_paths_for_seed(&seed, chain_type, context.clone());
        Wallet::standard_wallet_with_seed(seed, 0, is_transient, chain_type, context, self)
    }

    fn wallet_with_seed_phrase<L: Language>(&self, seed_phrase: &str, is_transient: bool, created_at: u64, chain_type: ChainType) {
        match chain_type.seed_for_seed_phrase::<L>(seed_phrase) {
            Some(seed) => match Keychain::save_seed_phrase(seed_phrase, created_at, seed.unique_id_as_str()) {
                Ok(()) => self.wallet_with_seed(seed, is_transient, chain_type),
                Err(err) => panic!("Can't create wallet with seed")
            },
            None => panic!("Can't create seed with seed phrase")
        }
    }

    // TODO: register transient wallets?
    // registering wallet in chain allows to adjust bloom filter to
    // receiving wallet-related info like transactions etc
    fn register_wallet(&mut self, wallet: Arc<RwLock<Wallet>>) {
        println!("register_wallet.1: {:?}", wallet);
        // self.wallets.push(wallet);
        // self.with(|chain| {
        //     println!("register_wallet.2: {:?}", chain);
        //     // chain.wallets.with(|wallets| {
        //         wallet.with(|w| {
        //             println!("register_wallet.3: {:?}", w);
        //             chain.wallets.push(w.clone());
        //         })
        //     // })
        // });
    }
}
