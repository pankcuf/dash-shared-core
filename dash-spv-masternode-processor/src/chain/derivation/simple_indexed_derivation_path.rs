use std::collections::HashSet;
use std::ops::Range;
use std::sync::Weak;
use crate::chain::ScriptMap;
use crate::chain::common::ChainType;
use crate::chain::derivation::derivation_path::DerivationPath;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::derivation_path_type::DerivationPathType;
use crate::chain::derivation::index_path::{IIndexPath, IndexPath};
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::wallet::seed::Seed;
use crate::crypto::{UInt160, UInt256};
use crate::keys::{Key, KeyKind};
use crate::storage::manager::managed_context::ManagedContext;
use crate::util;
use crate::util::address::address;

pub trait ISimpleIndexedDerivationPath: IDerivationPath {
    fn base(&self) -> &dyn IDerivationPath;
    /// gets addresses to an index, does not use cache and does not add to cache
    fn addresses_to_index(&mut self, index: u32) -> HashSet<String> {
        self.addresses_to_index_using_cache(index, false, false)
    }
    /// gets addresses to an index, does not use cache and does not add to cache
    fn addresses_to_index_using_cache(&mut self, index: u32, use_cache: bool, add_to_cache: bool) -> HashSet<String>;
    /// gets an address at an index
    fn address_at_index(&mut self, index: u32) -> Option<String> {
        self.address_at_index_path(&IndexPath::index_path_with_index(index))
    }
    /// true if the address at the index was previously used as an input or output in any wallet transaction
    fn address_is_used_at_index(&mut self, index: u32) -> bool {
        self.address_is_used_at_index_path(&IndexPath::index_path_with_index(index))
    }
    /// returns the index of an address in the derivation path as long as it is within the gap limit
    fn index_of_known_address(&self, address: &String) -> Option<u32>;
    fn index_of_known_address_hash_for_script(&self, hash: &UInt160, script: &ScriptMap) -> Option<u32>;
    /// gets a public key at an index
    fn public_key_data_at_index(&mut self, index: u32) -> Option<Vec<u8>> {
        self.public_key_data_at_index_path(&IndexPath::index_path_with_index(index))
    }
    /// gets public keys to an index as Vec<u8>
    fn public_key_data_array_to_index(&mut self, index: u32) -> Vec<Vec<u8>> {
        (0..index).filter_map(|i| self.public_key_data_at_index(i)).collect()
    }

    // /// gets a private key at an index
    // fn private_key_at_index(&self, index: u32, seed: &Vec<u8>) -> Option<&dyn IKey> {
    //     self.private_key_at_index_path_from_seed(&IndexPath::index_path_with_index(index), seed)
    // }
    /// get private keys for a range or to an index
    fn private_keys_to_index(&self, index: u32, seed: &Seed) -> Vec<Key> where Self: IIndexPath<Item = UInt256> {
        self.private_keys_for_range(0..index, seed)
    }
    fn private_keys_for_range(&self, range: Range<u32>, seed: &Seed) -> Vec<Key> where Self: IIndexPath<Item = UInt256> {
        range.filter_map(|i| self.private_key_at_index(i, seed)).collect()
    }
    fn default_gap_limit(&self) -> u32 {
        10
    }
    /// update addresses
    fn register_addresses_with_default_gap_limit(&mut self) -> Result<Vec<String>, util::Error> {
        self.register_addresses_with_gap_limit(self.default_gap_limit())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SimpleIndexedDerivationPath {
    pub base: DerivationPath,
    pub ordered_addresses: Vec<String>,
}

impl IIndexPath for SimpleIndexedDerivationPath {
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



impl IDerivationPath for SimpleIndexedDerivationPath {

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

    fn load_addresses(&mut self) {
        // todo: storage
        /*if !self.base.addresses_loaded {
            self.context().perform_block_and_wait(|context| {
                match DerivationPathEntity::aggregate_addresses_with_their_relationships(self, context) {
                    Ok((entity, mut addresses)) => {
                        self.base.sync_block_height = entity.sync_block_height as u32;
                        addresses.sort_by_key(|(addr, _)| addr.index);
                        addresses.iter().for_each(|(e, used_in_relationships)| {
                            // todo: do we need store nulls??
                            // while (e.index >= self.mOrderedAddresses.count)
                            //  [self.mOrderedAddresses addObject:[NSNull null]];
                            if Address::is_valid_dash_address_for_script_map(&e.address, self.chain().script()) {
                                self.ordered_addresses.push(e.address.clone());
                                self.base.all_addresses.push(e.address.clone());
                                if *used_in_relationships {
                                    self.base.used_addresses.push(e.address.clone());
                                }
                            }
                        });

                    },
                    Err(_) => panic!("Can't load addresses for path")
                }
            });
            self.base.addresses_loaded = true;
            self.register_addresses_with_gap_limit(10)
                .expect("");
        }*/
    }

    fn reload_addresses(&mut self) {
        self.base.all_addresses.clear();
        self.ordered_addresses.clear();
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
        self.base.set_balance(amount);
    }

    fn index_path_for_known_address(&self, address: &String) -> Option<IndexPath<u32>> {
        self.index_of_known_address(address)
            .map(|index| IndexPath::index_path_with_index(index))
    }

    fn generate_extended_public_key_from_seed(&mut self, seed: &Seed) -> Option<Key> {
        self.base.generate_extended_public_key_from_seed(seed)
    }

    fn register_transaction_address(&mut self, address: &String) -> bool {
        let contains = self.contains_address(address);
        if contains && !self.used_addresses().contains(address) {
            self.used_addresses().insert(address.clone());
            let _ = self.register_addresses_with_default_gap_limit();
        }
        contains
    }

    // Wallets are composed of chains of addresses. Each chain is traversed until a gap of a certain number of addresses is
    // found that haven't been used in any transactions. This method returns an array of <gapLimit> unused addresses
    // following the last used address in the chain.
    fn register_addresses_with_gap_limit(&mut self, gap_limit: u32) -> Result<Vec<String>, util::Error> {
        assert_ne!(self.base.r#type, DerivationPathType::MultipleUserAuthentication, "This should not be called for multiple user authentication. Use 'register_addresses_with_gap_limit_and_identity_index()'' instead.");
        if self.wallet_unique_id().is_some() {
            let mut array = self.ordered_addresses.clone();
            if !self.is_transient() {
                assert!(self.base.addresses_loaded, "addresses must be loaded before calling this function");
            }
            let mut i = array.len();
            // keep only the trailing contiguous block of addresses that aren't used
            while i > 0 && !self.used_addresses().contains(array.get(i - 1).unwrap()) {
                i -= 1;
            }
            if i > 0 {
                array.drain(0..i);
            }
            let limit = gap_limit as usize;
            if array.len() >= limit {
                return Ok(array.drain(0..limit).collect());
            }
            // It seems weird to repeat this, but it's correct because of the original call receive address and change address
            array = self.ordered_addresses.clone();
            i = array.len();
            let mut n = i as u32;
            // keep only the trailing contiguous block of addresses with no transactions
            while i > 0 && !self.used_addresses().contains(array.get(i - 1).unwrap()) {
                i -= 1;
            }
            if i > 0 {
                array.drain(0..i);
                if array.len() >= limit {
                    return Ok(array.drain(0..limit).collect());
                }
            }
            while array.len() < limit {
                // generate new addresses up to gapLimit
                if let Some(pub_key) = self.public_key_data_at_index(n) {
                    let addr = address::with_public_key_data(&pub_key, &self.chain_type().script_map());
                    // TODO: impl storage
                    /*if !self.is_transient() {
                        match DerivationPathEntity::derivation_path_entity_matching_derivation_path(self, self.base.context) {
                            Ok(derivationPathEntity) => {
                                // store new address in core data
                                AddressEntity::create_with(derivationPathEntity.id, addr.as_str(), n as i32, false, false, self.base.context)
                                    .expect("Can't store address entity");
                            },
                            Err(err) => {
                                return Err(util::Error::Default(format!("Can't retrieve derivation path entity for {:?}", self)));
                            }
                        }
                    }*/
                    self.base.all_addresses.push(addr.clone());
                    array.push(addr.clone());
                    self.ordered_addresses.push(addr.clone());
                    n += 1;
                }
            }
            Ok(array)
        } else {
            Err(util::Error::Default(format!("Error register_addresses_with_gap_limit")))
        }
    }
}

impl ISimpleIndexedDerivationPath for SimpleIndexedDerivationPath {
    fn base(&self) -> &dyn IDerivationPath {
        &self.base
    }

    fn addresses_to_index_using_cache(&mut self, index: u32, use_cache: bool, add_to_cache: bool) -> HashSet<String> {
        let mut arr = HashSet::<String>::new();
        (0..index).for_each(|i| {
            let idx = i as usize;
            if use_cache && self.ordered_addresses.len() > idx && self.ordered_addresses.get(idx).is_some() {
                arr.insert(self.ordered_addresses[idx].clone());
            } else if let Some(pubkey) = self.public_key_data_at_index(i) {
                let addr = address::with_public_key_data(&pubkey, &self.chain_type().script_map());
                arr.insert(addr.clone());
                if add_to_cache && self.ordered_addresses.len() == idx {
                    self.ordered_addresses.push(addr.clone());
                }
            }
        });
        arr
    }

    fn index_of_known_address(&self, address: &String) -> Option<u32> {
        self.ordered_addresses.iter().position(|x| x == address).map(|pos| pos as u32)
    }

    fn index_of_known_address_hash_for_script(&self, hash: &UInt160, script: &ScriptMap) -> Option<u32> {
        let address = address::from_hash160_for_script_map(hash, script);
        self.index_of_known_address(&address)
    }
}

impl SimpleIndexedDerivationPath {
    pub fn simple_indexed_derivation_path(indexes: Vec<UInt256>, hardened: Vec<bool>, r#type: DerivationPathType, signing_algorithm: KeyKind, reference: DerivationPathReference, chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self {
            base: DerivationPath::derivation_path_with_indexes(indexes, hardened, r#type, signing_algorithm, reference, chain_type, context),
            ..Default::default()
        }
    }

    /// returns the index of the first unused Address;
    pub fn first_unused_index(&self) -> u32 {
        let mut i = self.ordered_addresses.len();
        // keep only the trailing contiguous block of addresses that aren't used
        while i > 0 &&
            self.ordered_addresses.get(i - 1).is_some() &&
            !self.used_addresses().contains(self.ordered_addresses.get(i - 1).unwrap()) {
            i -= 1;
        }
        i as u32
    }

}
