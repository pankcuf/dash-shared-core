use std::collections::{HashMap, HashSet};
use std::sync::Weak;
use hashes::hex::ToHex;
use crate::chain::common::ChainType;
use crate::chain::derivation::derivation_path::DerivationPath;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::derivation_path_type::DerivationPathType;
use crate::chain::derivation::has_known_balance_unique_id;
use crate::chain::derivation::index_path::{IIndexPath, IndexPath};
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::derivation::sequence_gap_limit::SequenceGapLimit;
use crate::chain::wallet::ext::constants::account_unique_id_from;
use crate::chain::wallet::seed::Seed;
use crate::crypto::UInt256;
use crate::keys::{ECDSAKey, IKey, Key, KeyKind};
use crate::storage::keychain::Keychain;
use crate::storage::manager::managed_context::ManagedContext;
use crate::util;
use crate::util::address::address;

#[derive(Clone, Debug, Default)]
pub struct FundsDerivationPath {
    pub base: DerivationPath,

    account_number: u32,
    internal_addresses: Vec<String>,
    external_addresses: Vec<String>,

    // is_for_first_account: bool,
    has_known_balance_internal: bool,
    checked_initial_has_known_balance: bool,
}

impl PartialEq for FundsDerivationPath {
    fn eq(&self, other: &Self) -> bool {
        self.base.standalone_extended_public_key_unique_id.eq(&other.base.standalone_extended_public_key_unique_id)
    }
}

impl IIndexPath for FundsDerivationPath {
    type Item = UInt256;
    fn new(indexes: Vec<Self::Item>) -> Self {
        Self { base: DerivationPath::new(indexes), ..Default::default() }
    }

    fn new_hardened(indexes: Vec<Self::Item>, hardened: Vec<bool>) -> Self {
        Self { base: DerivationPath::new_hardened(indexes, hardened), ..Default::default() }
    }

    fn indexes(&self) -> &Vec<Self::Item> {
        self.base.indexes()
    }
    fn hardened_indexes(&self) -> &Vec<bool> {
        self.base.hardened_indexes()
    }
}

impl IDerivationPath for FundsDerivationPath {

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

    fn load_addresses(&mut self) {
        todo!()
    }

    fn reload_addresses(&mut self) {
        self.internal_addresses.clear();
        self.external_addresses.clear();
        self.base.used_addresses.clear();
        self.base.addresses_loaded = false;
        self.load_addresses();
    }

    fn standalone_extended_public_key_unique_id(&mut self) -> Option<String> {
        self.base.standalone_extended_public_key_unique_id()
    }

    fn balance(&self) -> u64 {
        self.base.balance()
    }

    fn set_balance(&mut self, amount: u64) {
        self.base.set_balance(amount)
    }

    fn index_path_for_known_address(&self, address: &String) -> Option<IndexPath<u32>> {
        self.internal_addresses.iter().position(|addr| addr == address)
            .map(|pos| IndexPath::index_path_with_indexes(vec![1, pos as u32]))
            .or(self.external_addresses.iter().position(|addr| addr == address)
                .map(|pos| IndexPath::index_path_with_indexes(vec![0, pos as u32])))
    }

    fn generate_extended_public_key_from_seed(&mut self, seed: &Seed) -> Option<Key> {
        self.base.generate_extended_public_key_from_seed(seed)
    }

    fn register_transaction_address(&mut self, address: &String) -> bool {
        // todo: avoid clone & optioning address
        let contains = self.contains_address(address);
        if contains {
            if !self.used_addresses().contains(address) {
                self.used_addresses().insert(address.clone());
                if self.all_change_addresses().contains(address) {
                    self.register_addresses_with_gap_limit(SequenceGapLimit::Internal.default(), true)
                        .expect("Error register_addresses_with_gap_limit");
                } else {
                    self.register_addresses_with_gap_limit(SequenceGapLimit::External.default(), false)
                        .expect("Error register_addresses_with_gap_limit");
                }
            }
        }
        contains
    }

    fn register_addresses(&mut self) -> HashSet<String> {
        let _ = self.register_addresses_with_gap_limit(SequenceGapLimit::Initial.default(), false);
        let _ = self.register_addresses_with_gap_limit(SequenceGapLimit::Initial.default(), true);
        let mut addresses: HashSet<String> = HashSet::new();
        addresses.extend(self.all_receive_addresses());
        addresses.extend(self.all_change_addresses());
        addresses
    }

}

impl FundsDerivationPath {

    pub fn bip32_derivation_path(account_number: u32, chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self {
            base: DerivationPath::derivation_path_with_indexes(
                vec![UInt256::from(account_number)],
                vec![true],
                DerivationPathType::ClearFunds,
                KeyKind::ECDSA,
                DerivationPathReference::BIP32,
                chain_type,
                context
            ),
            account_number,
            ..Default::default()
        }
    }

    pub fn bip44_derivation_path(account_number: u32, chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self {
            base: DerivationPath::derivation_path_with_indexes(
                vec![
                    UInt256::from(44u32),
                    UInt256::from(chain_type.coin_type()),
                    UInt256::from(account_number)
                ],
                vec![true, true, true],
                DerivationPathType::ClearFunds,
                KeyKind::ECDSA,
                DerivationPathReference::BIP44,
                chain_type,
                context
            ),
            account_number,
            ..Default::default()
        }
    }

    pub fn should_use_reduced_gap_limit(&mut self) -> bool {
        if !self.checked_initial_has_known_balance {
            if let Ok(has_known_balance) = Keychain::get_int(self.has_known_balance_unique_id_string()) {
                self.has_known_balance_internal = has_known_balance != 0;
                self.checked_initial_has_known_balance = true;
            }
        }
        !self.has_known_balance_internal &&
            !(self.is_for_first_account() && DerivationPathReference::BIP44.eq(&self.reference()))
    }

    pub fn set_has_known_balance(&mut self) {
        if !self.has_known_balance_internal {
            Keychain::set_int(1, self.has_known_balance_unique_id_string(), false)
                .expect("Can't save balance flag in keychain");
            self.has_known_balance_internal = true;
        }
    }

    fn is_for_first_account(&self) -> bool {
        self.account_number == 0
    }

    fn has_known_balance_unique_id_string(&self) -> String {
        self.wallet_unique_id()
            .map_or(String::new(), |unique_id|
                has_known_balance_unique_id(account_unique_id_from(unique_id, self.account_number), u32::from(self.reference())))
    }

    /// Wallets are composed of chains of addresses. Each chain is traversed until a gap of a certain number of addresses is
    /// found that haven't been used in any transactions. This method returns an array of <gapLimit> unused addresses
    /// following the last used address in the chain. The internal chain is used for change addresses and the external chain
    /// for receive addresses.
    pub fn register_addresses_with_gap_limit(&mut self, gap_limit: u32, internal: bool) -> Result<HashSet<String>, util::Error> {
        if self.wallet_unique_id().is_some() {
            if !self.is_transient() {
                assert!(self.base.addresses_loaded, "addresses must be loaded before calling this function");
            }
            let mut arr = if internal { self.internal_addresses.clone() } else { self.external_addresses.clone() };
            let mut i = arr.len();
            // keep only the trailing contiguous block of addresses with no transactions
            while i > 0 && !arr.iter().last().map_or(false, |value| self.used_addresses().contains(value)) {
                i -= 1;
            }
            if i > 0 {
                arr.drain(..i);
            }
            let limit = gap_limit as usize;
            if arr.len() >= limit {
                return Ok(arr.iter().take(limit).cloned().collect());
            }
            if limit > 1 { // get receiveAddress and changeAddress first to avoid blocking
                self.receive_address();
                self.change_address();
            }
            // It seems weird to repeat this, but it's correct because of the original call receive address and change address
            arr = if internal { self.internal_addresses.clone() } else { self.external_addresses.clone() };
            i = arr.len();
            let mut n = i as u32;
            // keep only the trailing contiguous block of addresses with no transactions
            while i > 0 && !arr.iter().last().map_or(false, |value| self.used_addresses().contains(value)) {
                i -= 1;
            }
            if i > 0 {
                arr.drain(..i);
            }
            if arr.len() >= limit {
                return Ok(arr.iter().take(limit).cloned().collect());
            }
            let mut add_addresses = HashMap::<u32, String>::new();
            while arr.len() < limit { // generate new addresses up to gapLimit
                if let Some(addr) = self.public_key_data_at_index(n, internal)
                    .and_then(|pub_key| ECDSAKey::key_with_public_key_data(&pub_key)
                        .map(|key| address::with_public_key_data(&key.public_key_data(), &self.chain_type().script_map()))) {
                    self.base.all_addresses.push(addr.clone());
                    if internal {
                        self.internal_addresses.push(addr.clone());
                    } else {
                        self.external_addresses.push(addr.clone());
                    }
                    arr.push(addr.clone());
                    add_addresses.insert(n, addr.clone());
                    n += 1;
                } else {
                    println!("error generating keys");
                    // return None;
                    return Err(util::Error::DefaultWithCode(format!("Error generating public keys"), 500));
                }
            }
            // TODO: store addresses
            // if !wallet.is_transient() {
            //     match DerivationPathEntity::derivation_path_entity_matching_derivation_path(self, self.context()) {
            //         Ok(derivationPathEntity) => {
            //             for (n, addr) in add_addresses {
            //                 match AddressEntity::create_with(
            //                     derivationPathEntity.id,
            //                     addr.as_str(),
            //                     n as i32,
            //                     internal,
            //                     false,
            //                     self.context()
            //                 ) {
            //                     Ok(created) => {},
            //                     Err(err) => { return Err(util::Error::Default(format!("Can't retrieve derivation path"))); }
            //                 }
            //             }
            //         },
            //         Err(err) => {
            //             return Err(util::Error::Default(format!("Can't retrieve derivation path")));
            //         }
            //     }
            // }
            Ok(HashSet::from_iter(arr.into_iter()))
        } else {
            Err(util::Error::DefaultWithCode(format!("Error generating public keys (no wallet)"), 500))
        }
    }

    /// gets an address at an index path
    pub fn address_at_index(&mut self, index: u32, internal: bool) -> Option<String> {
        self.public_key_data_at_index(index, internal)
            .and_then(|pub_key| ECDSAKey::key_with_public_key_data(&pub_key)
                .map(|key| address::with_public_key_data(&key.public_key_data(), &self.chain_type().script_map())))
    }

    /// returns the first unused external address
    pub fn receive_address(&mut self) -> Option<String> {
        // TODO: limit to 10,000 total addresses and utxos for practical usability with bloom filters
        self.receive_address_at_offset(0)
    }

    /// returns the first unused external address at offset
    pub fn receive_address_at_offset(&mut self, offset: u32) -> Option<String> {
        // TODO: limit to 10,000 total addresses and utxos for practical usability with bloom filters
        self.register_addresses_with_gap_limit(offset + 1, false)
            .ok()
            .and_then(|addresses| addresses.iter().last().cloned())
            .or(self.all_receive_addresses().iter().last().cloned())
    }

    /// returns the first unused internal address
    pub fn change_address(&mut self) -> Option<String> {
        // TODO: limit to 10,000 total addresses and utxos for practical usability with bloom filters
        self.register_addresses_with_gap_limit(1, true)
            .ok()
            .and_then(|addresses| addresses.iter().last().cloned())
    }

    /// all previously generated external addresses
    pub fn all_receive_addresses(&self) -> Vec<String> {
        self.external_addresses.clone()
    }

    /// all previously generated internal addresses
    pub fn all_change_addresses(&self) -> Vec<String> {
        self.internal_addresses.clone()
    }

    /// whether the internal address is controlled by the wallet
    pub fn contains_change_address(&self, address: &String) -> bool {
        self.internal_addresses.contains(address)
    }

    /// whether the external address is controlled by the wallet
    pub fn contains_receive_address(&self, address: &String) -> bool {
        self.external_addresses.contains(address)
    }

    pub fn used_receive_addresses(&self) -> Vec<String> {
        HashSet::from_iter(self.all_receive_addresses().into_iter()).intersection(&self.used_addresses()).cloned().collect()
    }

    pub fn used_change_addresses(&self) -> Vec<String> {
        HashSet::from_iter(self.all_change_addresses().into_iter()).intersection(&self.used_addresses()).cloned().collect()
    }

    pub fn public_key_data_at_index(&mut self, index: u32, internal: bool) -> Option<Vec<u8>> {
        self.public_key_data_at_index_path(&IndexPath::index_path_with_indexes(vec![u32::from(internal), index]))
    }

    pub fn private_key_string_at_index(&self, index: u32, internal: bool, seed: &Seed) -> Option<String> {
        println!("FundsDerivationPath.private_key_string_at_index: {} {} {}", index, internal, seed.data.to_hex());
        self.serialized_private_keys(vec![index], internal, seed)
            .and_then(|keys| keys.iter().last().cloned())
    }

    pub fn private_keys(&self, indexes: Vec<u32>, internal: bool, seed: &Seed) -> Vec<Key> {
        self.private_keys_at_index_paths(
            indexes.iter()
                .map(|&index| IndexPath::index_path_with_indexes(vec![u32::from(internal), index]))
                .collect(),
            seed)
    }

    pub fn serialized_private_keys(&self, indexes: Vec<u32>, internal: bool, seed: &Seed) -> Option<Vec<String>> {
        self.base.serialized_private_keys_at_index_paths(
            indexes.iter()
                .map(|&index| IndexPath::index_path_with_indexes(vec![u32::from(internal), index])).collect(), seed)
    }}
