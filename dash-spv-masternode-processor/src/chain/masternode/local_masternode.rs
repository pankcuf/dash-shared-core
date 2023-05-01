use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use crate::chain::chain::Chain;
use crate::chain::masternode::LocalMasternodeStatus;
use crate::chain::tx::{ITransaction, ProviderRegistrationTransaction, ProviderUpdateRegistrarTransaction, ProviderUpdateRevocationTransaction, ProviderUpdateServiceTransaction};
use crate::chain::wallet::account::Account;
use crate::chain::wallet::wallet::Wallet;
use crate::crypto::{UInt128, UInt160, UInt384};
use crate::crypto::UTXO;
use crate::keys::bls_key::BLSKey;
use crate::keys::ecdsa_key::ECDSAKey;
use crate::storage::Keychain;
use crate::util::Shared;
// use crate::storage::manager::managed_context::ManagedContext;
// use crate::storage::models::masternode::LocalMasternodeEntity;

type RegistrationCallback = fn(ProviderRegistrationTransaction);
type UpdateServiceCallback = fn(ProviderUpdateServiceTransaction);
type UpdateRegistrarCallback = fn(ProviderUpdateRegistrarTransaction);
type ReclaimCallback = fn(dyn ITransaction);

pub const MASTERNODE_NAME_KEY: &str = "MASTERNODE_NAME_KEY";

#[derive(Clone, Debug, Default)]
pub struct LocalMasternode {
    pub name: String,
    ip_address: UInt128,
    port: u16,
    // only if this is contained in the wallet.
    pub operator_keys_wallet: Option<Wallet>,
    // the derivation path index of keys
    pub operator_wallet_index: u32,

    // only if this is contained in the wallet.
    pub owner_keys_wallet: Option<Wallet>,
    // the derivation path index of keys
    pub owner_wallet_index: u32,

    // only if this is contained in the wallet.
    pub voting_keys_wallet: Option<Wallet>,
    // the derivation path index of keys
    pub voting_wallet_index: u32,

    // only if this is contained in the wallet.
    pub holding_keys_wallet: Option<Wallet>,
    // the derivation path index of keys
    pub holding_wallet_index: u32,

    // previously used operator indexes
    pub previous_operator_wallet_indexes: HashSet<u32>,
    // previously used voting indexes
    pub previous_voting_wallet_indexes: HashSet<u32>,

    pub status: LocalMasternodeStatus,
    pub provider_registration_transaction: Option<Shared<ProviderRegistrationTransaction>>,
    pub provider_update_registrar_transactions: Vec<Shared<ProviderUpdateRegistrarTransaction>>,
    pub provider_update_service_transactions: Vec<Shared<ProviderUpdateServiceTransaction>>,
    pub provider_update_revocation_transactions: Vec<Shared<ProviderUpdateRevocationTransaction>>,
}

impl LocalMasternode {

    pub fn init_with_provider_registration_transaction(transaction: &ProviderRegistrationTransaction) -> Self {
        todo!()
        // let (owner_keys_wallet, owner_wallet_index) = if let Some((&wallet, index)) = transaction.chain().wallet_having_provider_owner_authentication_hash(&transaction.owner_key_hash) {
        //     (Some(wallet), index)
        // } else {
        //     (None, 0)
        // };
        // let (voting_keys_wallet, voting_wallet_index) = if let Some((&wallet, index)) = transaction.chain().wallet_having_provider_voting_authentication_hash(&transaction.voting_key_hash) {
        //     (Some(wallet), index)
        // } else {
        //     (None, 0)
        // };
        // let (operator_keys_wallet, operator_wallet_index) = if let Some((&wallet, index)) = transaction.chain().wallet_having_provider_operator_authentication_key(&transaction.operator_key) {
        //     (Some(wallet), index)
        // } else {
        //     (None, 0)
        // };
        // let (holding_keys_wallet, holding_wallet_index) = if let Some((&wallet, index)) = transaction.chain().wallet_containing_masternode_holding_address_for_provider_registration_transaction(&transaction) {
        //     (Some(wallet), index)
        // } else {
        //     (None, 0)
        // };
        // let key = format!("{}{}", MASTERNODE_NAME_KEY, transaction.tx_hash());
        // Self {
        //     name: Keychain::get_string(key).unwrap_or("".to_string()),
        //     ip_address: transaction.ip_address,
        //     port: transaction.port,
        //     operator_keys_wallet,
        //     operator_wallet_index,
        //     owner_keys_wallet,
        //     owner_wallet_index,
        //     voting_keys_wallet,
        //     voting_wallet_index,
        //     holding_keys_wallet,
        //     holding_wallet_index,
        //     previous_operator_wallet_indexes: HashSet::new(),
        //     previous_voting_wallet_indexes: HashSet::new(),
        //     status: LocalMasternodeStatus::Registered, // because it comes from a transaction already
        //     provider_registration_transaction: Some(transaction),
        //     provider_update_registrar_transactions: vec![],
        //     provider_update_service_transactions: vec![],
        //     provider_update_revocation_transactions: vec![]
        // }
    }


    pub fn register_in_associated_wallets(&self) {
        todo!()
        // self.operator_keys_wallet.as_mut().map(|mut wallet| wallet.register_masternode_operator(self));
        // if let Some(mut wallet) = &self.operator_keys_wallet {
        //     wallet.register_masternode_operator(self);
        // }
        // if let Some(mut wallet) = &self.owner_keys_wallet {
        //     wallet.register_masternode_owner(self);
        // }
        // if let Some(mut wallet) = &self.voting_keys_wallet {
        //     wallet.register_masternode_voter(self);
        // }
    }

    pub fn force_operator_public_key(&self, operator_public_key: &mut BLSKey) -> bool {
        todo!()
        // if let Some(mut wallet) = &self.owner_keys_wallet {
        //     if self.operator_wallet_index == u32::MAX {
        //         wallet.register_masternode_operator_with_public_key(self, operator_public_key);
        //         return true
        //     }
        // }
        // false
    }

    pub fn force_owner_private_key(&self, owner_private_key: &mut ECDSAKey) -> bool {
        todo!()
        // if let Some(mut wallet) = &self.owner_keys_wallet {
        //     if self.owner_wallet_index == u32::MAX && owner_private_key.has_private_key() {
        //         wallet.register_masternode_owner_with_owner_private_key(self, owner_private_key)
        //     }
        // }
        // false
    }

    // the voting key can either be private or public key
    pub fn force_voting_key(&self, voting_key: &mut ECDSAKey) -> bool {
        todo!()
        // if let Some(mut wallet) = &self.owner_keys_wallet {
        //     if self.voting_wallet_index == u32::MAX {
        //         wallet.register_masternode_voter_with_voter_key(self, voting_key);
        //     }
        // }
        // false
    }

    pub fn no_local_wallet(&self) -> bool {
        !(self.operator_keys_wallet.is_some() || self.holding_keys_wallet.is_some() || self.owner_keys_wallet.is_some() || self.voting_keys_wallet.is_some())
    }

    pub fn ip_address(&self) -> &UInt128 {
        todo!()
        // if !self.provider_update_service_transactions.is_empty() {
        //     &self.provider_update_service_transactions.last().unwrap().ip_address
        // } else if let Some(tx) = &self.provider_registration_transaction {
        //     &tx.ip_address
        // } else {
        //     &self.ip_address
        // }
    }

    pub fn chain(&self) -> Option<Arc<Chain>> {
        todo!()
        // if let Some(tx) = &self.provider_registration_transaction {
        //     Some(tx.chain().borrow())
        // } else if let Some(wallet) = &self.operator_keys_wallet {
        //     Some(wallet.chain.borrow())
        // } else if let Some(wallet) = &self.owner_keys_wallet {
        //     Some(wallet.chain.borrow())
        // } else if let Some(wallet) = &self.voting_keys_wallet {
        //     Some(wallet.chain.borrow())
        // } else if let Some(wallet) = &self.holding_keys_wallet {
        //     Some(wallet.chain.borrow())
        // } else {
        //     assert!(false, "A chain should have been found at this point");
        //     None
        // }
    }

    pub fn ip_address_string(&self) -> String {
        IpAddr::from(self.ip_address.0).to_string()
    }

    pub fn ip_address_and_port_string(&self) -> String {
        format!("{}:{}", self.ip_address_string(), self.port)
    }

    pub fn ip_address_and_if_non_standard_port_string(&self) -> String {
        todo!()
        // if self.chain().is_some() && self.chain().unwrap().is_mainnet() && self.port() == self.provider_registration_transaction.unwrap().chain().params.standard_port {
        //     self.ip_address_string()
        // } else {
        //     self.ip_address_and_port_string()
        // }
    }

    pub fn port(&self) -> u16 {
        todo!()
        // if !self.provider_update_service_transactions.is_empty() {
        //     self.provider_update_service_transactions.last().unwrap().port
        // } else if let Some(tx) = &self.provider_registration_transaction {
        //     tx.port
        // } else {
        //     self.port
        // }
    }
    pub fn port_string(&self) -> String {
        format!("{}", self.port())
    }

    pub fn payout_address(&self) -> Option<String> {
        todo!()
        // self.provider_registration_transaction.and_then(|tx| Address::with_script_pub_key(
        //     if self.provider_update_registrar_transactions.is_empty() {
        //         &tx.script_payout
        //     } else {
        //         &self.provider_update_registrar_transactions.last().unwrap().script_payout
        //     }, &tx.chain().script()))
    }

    pub fn operator_payout_address(&self) -> Option<String> {
        todo!()
        // self.provider_registration_transaction.filter(|tx| !self.provider_update_service_transactions.is_empty())
        //     .and_then(|tx| Address::with_script_pub_key(&self.provider_update_service_transactions.last().unwrap().script_payout, tx.chain().script()))
    }

    pub fn operator_key_from_seed(&self, seed: Vec<u8>) -> Option<BLSKey> {
        todo!()
        // self.operator_keys_wallet.and_then(|wallet| self.provider_registration_transaction.and_then(|tx| {
        //     let path = AuthenticationKeysDerivationPath::provider_operator_keys_derivation_path_for_wallet(&wallet);
        //     let hash = UInt160::hash160(&tx.operator_key.0);
        //     path.private_key_for_hash160(hash, seed)
        // }))
    }

    pub fn operator_key_string_from_seed(&self, seed: Vec<u8>) -> Option<String> {
        self.operator_key_from_seed(seed)
            .map(|key| key.secret_key_string())
    }

    pub fn owner_key_from_seed(&self, seed: Option<Vec<u8>>) -> Option<ECDSAKey> {
        todo!()
        // self.owner_keys_wallet.and_then(|wallet|
        //     seed.and_then(|seed|
        //         self.provider_registration_transaction.and_then(|pro_reg_tx|
        //             AuthenticationKeysDerivationPath::provider_owner_keys_derivation_path_for_wallet(&wallet)
        //                 .private_key_for_hash160::<ECDSAKey>(pro_reg_tx.owner_key_hash, seed))))
    }

    pub fn owner_key_string_from_seed(&self, seed: Option<Vec<u8>>) -> Option<String> {
        self.owner_key_from_seed(seed)
            .map(|key| key.secret_key_string())
    }

    pub fn voting_key_from_seed(&self, seed: Option<Vec<u8>>) -> Option<ECDSAKey> {
        todo!()
        // if let Some(wallet) = &self.voting_keys_wallet {
        //     if let Some(seed) = seed {
        //         if let Some(pro_reg_tx) = self.provider_registration_transaction {
        //             let path = AuthenticationKeysDerivationPath::provider_voting_keys_derivation_path_for_wallet(wallet);
        //             return path.private_key_for_hash160::<ECDSAKey>(pro_reg_tx.voting_key_hash, seed);
        //         }
        //     }
        // }
        // None
    }

    pub fn owner_public_key_data(&self) -> Vec<u8> {
        todo!()
        // if let Some(wallet) = &self.owner_keys_wallet {
        //     if let Some(tx) = &self.provider_registration_transaction {
        //         let mut path = AuthenticationKeysDerivationPath::provider_owner_keys_derivation_path_for_wallet(wallet);
        //         if let Some(data) = path.public_key_data_for_hash160(tx.owner_key_hash) {
        //             return data;
        //         }
        //     }
        // }
        // vec![]
    }

    pub fn operator_public_key_data(&self) -> Vec<u8> {
        todo!()
        // if let Some(wallet) = &self.operator_keys_wallet {
        //     if let Some(tx) = &self.provider_registration_transaction {
        //         let mut path = AuthenticationKeysDerivationPath::provider_operator_keys_derivation_path_for_wallet(wallet);
        //         if let Some(data) = path.public_key_data_for_hash160(UInt160::hash160(&tx.operator_key.0)) {
        //             return data;
        //         }
        //     }
        // }
        // vec![]
    }

    pub fn voting_public_key_data(&self) -> Vec<u8> {
        todo!()
        // if let Some(wallet) = &self.voting_keys_wallet {
        //     if let Some(tx) = &self.provider_registration_transaction {
        //         let mut path = AuthenticationKeysDerivationPath::provider_voting_keys_derivation_path_for_wallet(wallet);
        //         if let Some(data) = path.public_key_data_for_hash160(tx.voting_key_hash) {
        //             return data;
        //         }
        //     }
        // }
        // vec![]
    }

    pub fn voting_key_string_from_seed(&self, seed: Option<Vec<u8>>) -> Option<String> {
        self.voting_key_from_seed(seed)
            .map(|key| key.secret_key_string())
    }


    /// Named Masternodes

    pub fn masternode_identifier_for_name_storage(&self) -> String {
        todo!()
        // format!("{}{}", MASTERNODE_NAME_KEY, self.provider_registration_transaction.unwrap().tx_hash())
    }

    fn associate_name(&mut self) {
        if let Ok(data) = Keychain::get_string(self.masternode_identifier_for_name_storage()) {
            self.name = data
        }
    }

    pub fn register_name(&mut self, name: String) {
        if self.name.ne(&name) {
            let _ = Keychain::set_string(name.clone(), self.masternode_identifier_for_name_storage(), false);
            self.name = name;
        }
    }


    /// Generating Transactions
    pub fn registration_transaction_funded_by_account(&self, funding_account: &Account, payout_address: String, completion: RegistrationCallback) {
        self.registration_transaction_funded_by_account_with_collateral(funding_account, payout_address, UTXO::default(), completion)

    }

    pub fn registration_transaction_funded_by_account_with_collateral(&self, funding_account: &Account, payout_address: String, collateral: UTXO, completion: RegistrationCallback) {
        if self.status != LocalMasternodeStatus::New {
            return;
        }
        let question = format!("Are you sure you would like to register a masternode at {}:{}?", self.ip_address_string(), self.port());
        todo!("impl AuthManager stuff")
    }

    pub fn update_transaction_for_reset_funded_by_account(&self, funding_account: &Account, completion: UpdateServiceCallback) {
        self.update_transaction_funded_by_account(funding_account, self.ip_address(), self.port(), self.operator_payout_address(), completion)
    }

    pub fn update_transaction_funded_by_account(&self, funding_account: &Account, ip_address: &UInt128, port: u16, payout_address: Option<String>, completion: UpdateServiceCallback) {
        todo!()
    }

    pub fn update_transaction_funded_by_account_service(&self, funding_account: &Account, ip_address: &UInt128, port: u16, payout_address: Option<String>, completion: UpdateServiceCallback) {
        if self.status != LocalMasternodeStatus::Registered {
            return;
        }
        let question = format!("Are you sure you would like to update this masternode to {}:{}?", self.ip_address_string(), self.port());
        todo!("impl AuthManager stuff")
    }

    pub fn update_transaction_funded_by_account_registrar(&self, funding_account: &Account, operator_key: &UInt384, voting_key_hash: UInt160, payout_address: Option<String>, completion: UpdateRegistrarCallback) {
        if self.status != LocalMasternodeStatus::Registered {
            return;
        }
        let question = format!("Are you sure you would like to update this masternode to pay to {:?}?", payout_address);
        todo!("impl AuthManager stuff")
    }

    // Update from Transaction
    pub fn reclaim_transaction_to_account(&self, account: &Account, completion: ReclaimCallback) {
        if self.status != LocalMasternodeStatus::Registered {
            return;
        }
        let question = format!("Are you sure you would like to reclaim this masternode?");
        todo!("impl AuthManager stuff")
    }

    pub fn update_with_update_registrar_transaction(&mut self, transaction: &ProviderUpdateRegistrarTransaction, save: bool) {
        todo!()
        // if self.provider_update_registrar_transactions.contains(&transaction) {
        //     return;
        // }
        // self.provider_update_registrar_transactions.push(transaction);
        // let mut operator_new_wallet_index = 0u32;
        // if let Some(wallet) = &self.operator_keys_wallet {
        //     let path = AuthenticationKeysDerivationPath::provider_operator_keys_derivation_path_for_wallet(wallet);
        //     if let Some(op_address) = transaction.operator_address() {
        //         if let Some(index) = path.index_of_known_address(&op_address) {
        //             operator_new_wallet_index = index;
        //         }
        //     }
        // } else if let Some((&wallet, index)) = self.chain().unwrap().wallet_having_provider_operator_authentication_key(&transaction.operator_key) {
        //     operator_new_wallet_index = index;
        //     self.operator_keys_wallet = Some(wallet);
        // }
        // if self.operator_keys_wallet.is_some() && self.operator_wallet_index != operator_new_wallet_index {
        //     if self.operator_wallet_index != u32::MAX && !self.previous_operator_wallet_indexes.contains(&self.operator_wallet_index) {
        //         self.previous_operator_wallet_indexes.insert(self.operator_wallet_index);
        //     }
        //     if self.previous_operator_wallet_indexes.contains(&operator_new_wallet_index) {
        //         self.previous_operator_wallet_indexes.remove(&operator_new_wallet_index);
        //     }
        //     self.operator_wallet_index = operator_new_wallet_index;
        // }
        // let mut voting_new_wallet_index = 0u32;
        // if let Some(wallet) = &self.voting_keys_wallet {
        //     let path = AuthenticationKeysDerivationPath::provider_voting_keys_derivation_path_for_wallet(wallet);
        //     if let Some(address) = transaction.voting_address() {
        //         if let Some(index) = path.index_of_known_address(&address) {
        //             voting_new_wallet_index = index;
        //         }
        //     }
        // } else if let Some((&wallet, index)) = self.chain().unwrap().wallet_having_provider_voting_authentication_hash(&transaction.voting_key_hash) {
        //     operator_new_wallet_index = index;
        //     self.voting_keys_wallet = Some(wallet);
        // }
        // if self.voting_keys_wallet.is_some() && self.voting_wallet_index != voting_new_wallet_index {
        //     if self.voting_wallet_index != u32::MAX && !self.previous_voting_wallet_indexes.contains(&self.voting_wallet_index) {
        //         self.previous_voting_wallet_indexes.insert(self.voting_wallet_index);
        //     }
        //     if self.previous_voting_wallet_indexes.contains(&voting_new_wallet_index) {
        //         self.previous_voting_wallet_indexes.remove(&voting_new_wallet_index);
        //     }
        //     self.voting_wallet_index = voting_new_wallet_index;
        // }
        // if save {
        //     self.save();
        // }
    }

    pub fn update_with_update_revocation_transaction(&mut self, transaction: &ProviderUpdateRevocationTransaction, save: bool) {
        todo!()
        // if !self.provider_update_revocation_transactions.contains(&transaction) {
        //     self.provider_update_revocation_transactions.push(transaction);
        //     if save {
        //         self.save();
        //     }
        // }
    }

    pub fn update_with_update_service_transaction(&mut self, transaction: &ProviderUpdateServiceTransaction, save: bool) {
        todo!()
        // if !self.provider_update_service_transactions.contains(&transaction) {
        //     self.provider_update_service_transactions.push(transaction);
        //     self.ip_address = transaction.ip_address.clone();
        //     self.port = transaction.port.clone();
        //     if save {
        //         self.save();
        //     }
        // }
    }


    // Persistence

    /*pub fn save(&self) {
        self.save_in_context(self.chain().unwrap().chain_context())
    }

    pub fn save_in_context(&self, context: &ManagedContext) {
        context.perform_block_and_wait(|context| {
            LocalMasternodeEntity::save(self, context)
                .expect("Can't save local masternode entity");
        });
    }*/

}

// impl EntityConvertible for LocalMasternode {
//     fn to_entity<T, U>(&self) -> U
//         where
//             T: Table,
//             T::FromClause: QueryFragment<Sqlite>,
//             U: Insertable<T>,
//             U::Values: QueryFragment<Sqlite> + CanInsertInSingleQuery<Sqlite> {
//         NewLocalMasternodeEntity {
//             operator_keys_index: self.operator_wallet_index as i32,
//             owner_keys_index: self.owner_wallet_index as i32,
//             holding_keys_index: self.holding_wallet_index as i32,
//             voting_keys_index: self.voting_wallet_index as i32,
//             operator_keys_wallet_unique_id: self.operator_keys_wallet.map_or("", Wallet::unique_id_as_str),
//             owner_keys_wallet_unique_id: self.owner_keys_wallet.map_or("", Wallet::unique_id_as_str),
//             voting_keys_wallet_unique_id: self.voting_keys_wallet.map_or("", Wallet::unique_id_as_str),
//             holding_keys_wallet_unique_id: self.holding_keys_wallet.map_or("", Wallet::unique_id_as_str),
//             ..Default::default()
//         }
//     }
//
//     fn to_update_values(&self) -> Box<dyn EntityUpdates<bool, ResultType = (bool, )>> {
//         todo!()
//     }
//
//     fn from_entity<T: Entity>(entity: T, context: &ManagedContext) -> QueryResult<Self> {
//         todo!()
//     }
// }
