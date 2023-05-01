use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Weak;
use hashes::{Hash, sha256};
use crate::chain::bip::bip32;
use crate::chain::common::ChainType;
use crate::chain::derivation;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::index_path::{IIndexPath, IndexHardSoft, IndexPath};
use crate::chain::derivation::uint256_index_path::UInt256IndexPath;
use crate::chain::derivation::{standalone_extended_public_key_location_string_for_unique_id, wallet_based_extended_public_key_location_string_for_unique_id};
use crate::chain::wallet::seed::Seed;
use crate::crypto::UInt256;
use crate::keys::{IKey, Key, KeyKind};
use crate::storage::manager::managed_context::ManagedContext;
use crate::util;
use crate::util::address::address;
use crate::util::data_ops::short_hex_string_from;

pub trait IDerivationPath<IPATH: IIndexPath = UInt256IndexPath>: Send + Sync + Debug {
    // fn chain(&self) -> Weak<Chain>;
    fn chain_type(&self) -> ChainType;
    fn context(&self) -> Weak<ManagedContext>;
    // fn wallet(&self) -> &Option<Weak<Wallet>>;
    // fn set_wallet(&mut self, wallet: Weak<Wallet>);
    fn is_transient(&self) -> bool;
    fn set_is_transient(&mut self, is_transient: bool);
    fn wallet_unique_id(&self) -> Option<String>;
    fn set_wallet_unique_id(&mut self, unique_id: String);
    // fn set_wallet_with_unique_id(&mut self, wallet: Weak<Wallet>, unique_id: String) {
    //     self.set_wallet_unique_id(unique_id);
    //     self.set_wallet(wallet);
    // }
    // https://github.com/rust-lang/rust/issues/94980
    // fn params(&self) -> &Params;
    // fn context(&self) -> Weak<ManagedContext>;
    fn signing_algorithm(&self) -> KeyKind;
    fn reference(&self) -> DerivationPathReference;
    fn extended_public_key(&self) -> Option<Key>;
    fn extended_public_key_mut(&mut self) -> Option<Key>;
    fn extended_public_key_data(&self) -> Option<Vec<u8>> {
        self.extended_public_key().and_then(|key| key.extended_public_key_data())
    }
    fn extended_public_key_data_mut(&mut self) -> Option<Vec<u8>> {
        self.extended_public_key_mut().and_then(|key| key.extended_public_key_data())
    }
    fn private_key_from_seed(&self, seed: &Vec<u8>) -> Option<Key> where Self: IIndexPath<Item = UInt256> {
        self.signing_algorithm()
            .key_with_seed_data(seed)
            .and_then(|seed_key| seed_key.private_derive_to_256bit_derivation_path(self))
    }

    fn has_extended_public_key(&self) -> bool;
    fn to_bip32_key_with_key_data(&self, key_data: Vec<u8>) -> Option<bip32::Key> where Self: IIndexPath<Item = UInt256> {
        (key_data.len() >= 36).then_some({
            let (child, hardened) = if self.is_empty() {
                (UInt256::MIN, false)
            } else {
                (self.last_index(), self.last_hardened())
            };
            bip32::Key::new(
                self.depth(),
                u32::from_le_bytes(key_data[..4].try_into().unwrap()),
                child,
                UInt256::from(&key_data[4..36]),
                key_data[36..].to_vec(),
                hardened)
        })
    }
    fn serialized_extended_public_key(&self) -> Option<String> where Self: IIndexPath<Item = UInt256> {
        self.extended_public_key_data()
            .and_then(|key_data| self.to_bip32_key_with_key_data(key_data)
                .map(|key| key.serialize(self.chain_type())))
    }
    fn serialized_extended_public_key_mut(&mut self) -> Option<String> where Self: IIndexPath<Item = UInt256> {
        self.extended_public_key_data_mut()
            .and_then(|key_data| self.to_bip32_key_with_key_data(key_data)
                .map(|key| key.serialize(self.chain_type())))
        // todo make sure this works with BLS keys
    }
    fn serialized_extended_private_key_from_seed(&self, seed: &Vec<u8>) -> Option<String> where Self: IIndexPath<Item = UInt256> {
        self.private_key_from_seed(seed)
            .map(|seed_key| {
                let (child, hardened) = if self.is_empty() {
                    (UInt256::MIN, false)
                } else {
                    (self.last_index(), self.last_hardened())
                };
                bip32::Key::new(
                    self.length() as u8,
                    seed_key.fingerprint(),
                    child,
                    seed_key.chaincode(),
                    seed_key.secret_key().0.to_vec(),
                    hardened)
                    .serialize(self.chain_type())
            })
    }
    fn is_derivation_path_equal(&self, other: &Self) -> bool where Self: Sized + PartialEq {
        self == other
    }
    fn is_wallet_based(&self) -> bool where Self: IIndexPath {
        !self.is_empty() || self.reference() == DerivationPathReference::Root
    }
    fn public_key_location_string_for_wallet_unique_id(&self, unique_id: &str) -> String where Self: IIndexPath {
        if self.is_wallet_based() {
            wallet_based_extended_public_key_location_string_for_unique_id(unique_id)
        } else {
            standalone_extended_public_key_location_string_for_unique_id(unique_id)
        }
    }
    /// Purpose
    fn is_bip32_only(&self) -> bool where Self: IIndexPath {
        self.length() == 1
    }

    fn is_bip43_based(&self) -> bool where Self: IIndexPath {
        self.length() != 1
    }
    fn purpose(&self) -> u64 where Self: IIndexPath {
        if self.is_bip43_based() {
            self.index_at_position(0).softened()
        } else {
            0
        }
    }
    fn depth(&self) -> u8;

    /// all previously generated addresses
    fn all_addresses(&self) -> HashSet<String>;
    /// all previously used addresses
    fn used_addresses(&self) -> HashSet<String>;
    /// true if the address is controlled by the wallet
    fn contains_address(&self, address: &String) -> bool {
        self.all_addresses().contains(address)
    }
    // gets an address at an index path
    fn address_at_index_path(&mut self, index_path: &IndexPath<u32>) -> Option<String> {
        self.public_key_data_at_index_path(index_path)
            .map(|data| address::with_public_key_data(&data, &self.chain_type().script_map()))
    }
    // true if the address was previously used as an input or output in any wallet transaction
    fn address_is_used(&self, address: &String) -> bool {
        self.used_addresses().contains(address)
    }
    // true if the address at index path was previously used as an input or output in any wallet transaction
    fn address_is_used_at_index_path(&mut self, index_path: &IndexPath<u32>) -> bool {
        if let Some(address) = self.address_at_index_path(index_path) {
            self.address_is_used(&address)
        } else {
            false
        }
    }

    fn load_addresses(&mut self) {}
    fn reload_addresses(&mut self) {}
    // this returns the derivation path's visual representation (e.g. m/44'/5'/0')
    // fn string_representation(&mut self) -> &str;
    fn standalone_extended_public_key_unique_id(&mut self) -> Option<String>;
    // fn kind(&self) -> DerivationPathKind;
    fn balance(&self) -> u64;
    fn set_balance(&mut self, amount: u64);
    /// gets a private key at an index
    fn private_key_at_index(&self, index: u32, seed: &Seed) -> Option<Key>
        where Self: Sized + IDerivationPath + IIndexPath<Item = UInt256> {
        // where Self: Sized + IDerivationPath + ChildKeyDerivation {
        <Self as IDerivationPath<IPATH>>::private_key_at_index_path_from_seed(self, &IndexPath::index_path_with_index(index), seed)
        // self.private_key_at_index_path_from_seed(&IndexPath::index_path_with_index(index), seed)
    }
    fn private_key_at_index_path_from_seed(&self, index_path: &IndexPath<u32>, seed: &Seed) -> Option<Key>
        // where Self: Sized + IDerivationPath + ChildKeyDerivation {
        where Self: Sized + IDerivationPath + IIndexPath<Item = UInt256> {
        <Self as IDerivationPath<IPATH>>::signing_algorithm(self)
            .key_with_seed_data(&seed.data)
            .and_then(|top_key| top_key.private_derive_to_256bit_derivation_path(self)
                .and_then(|key| key.private_derive_to_path(index_path)))

    }

    fn private_keys_at_index_paths(&self, index_paths: Vec<IndexPath<u32>>, seed: &Seed) -> Vec<Key>
        where Self: Sized + IDerivationPath + IIndexPath<Item = UInt256> {
        // where Self: Sized + IDerivationPath + ChildKeyDerivation {
        if index_paths.is_empty() {
            vec![]
        } else {
            <Self as IDerivationPath<IPATH>>::signing_algorithm(self)
                .key_with_seed_data(&seed.data)
                .and_then(|top_key| top_key.private_derive_to_256bit_derivation_path(self)
                    .map(|key| index_paths.iter()
                        .filter_map(|index_path| key.private_derive_to_path(index_path))
                        .collect::<Vec<_>>()))
                .unwrap_or(vec![])
        }
    }

    fn private_key_for_known_address(&self, address: &String, seed: &Seed) -> Option<Key>
        where Self: Sized + IDerivationPath + IIndexPath<Item = UInt256> {
        // where Self: Sized + IDerivationPath + ChildKeyDerivation {
        <Self as IDerivationPath<IPATH>>::index_path_for_known_address(self, address)
            .and_then(|index_path| <Self as IDerivationPath<IPATH>>::private_key_at_index_path_from_seed(self, &index_path, seed))
    }

    fn public_key_at_index_path(&mut self, index_path: &IndexPath<u32>) -> Option<Key> {
        self.public_key_data_at_index_path(index_path)
            .and_then(|data| self.signing_algorithm().key_with_public_key_data(&data))
    }

    fn public_key_data_at_index_path(&mut self, index_path: &IndexPath<u32>) -> Option<Vec<u8>> {
        self.extended_public_key_data()
            .and_then(|data| self.signing_algorithm().public_key_from_extended_public_key_data(&data, index_path))
    }

    fn index_path_for_known_address(&self, address: &String) -> Option<IndexPath<u32>>;
    fn generate_extended_public_key_from_seed(&mut self, seed: &Seed) -> Option<Key>;

    fn register_transaction_address(&mut self, address: &String) -> bool;
    fn register_addresses_with_gap_limit(&mut self, gap_limit: u32) -> Result<Vec<String>, util::Error> {
        Err(util::Error::Default(format!("Should be overriden")))
    }

    fn register_addresses(&mut self) -> HashSet<String> {
        HashSet::new()
    }

    fn create_identifier_for_derivation_path(&mut self) -> String {
        short_hex_string_from(&sha256::Hash::hash(&self.extended_public_key_data().unwrap_or(vec![])).into_inner())
    }

    fn standalone_extended_public_key_location_string(&mut self) -> Option<String> {
        self.standalone_extended_public_key_unique_id()
            .map(|unique_id| derivation::standalone_extended_public_key_location_string_for_unique_id(&unique_id))
    }

    fn standalone_info_dictionary_location_string(&mut self) -> Option<String> {
        self.standalone_extended_public_key_unique_id()
            .map(|unique_id| derivation::standalone_info_dictionary_location_string_for_unique_id(&unique_id))
    }

    fn wallet_based_extended_public_key_location_string_for_wallet_unique_id<'a>(&self, unique_id: &'a str) -> String where Self: IIndexPath {
        derivation::wallet_based_extended_public_key_location_string_for_unique_id_and_key_type(unique_id, self.signing_algorithm(), self.index_path_enumerated_string())
    }

    /// Storage

    fn store_extended_public_key_under_wallet_unique_id(&mut self, wallet_unique_id: &String) -> bool where Self: IIndexPath {
        /*if let Some(mut key) = self.extended_public_key() {
            Keychain::set_data(self.wallet_based_extended_public_key_location_string_for_wallet_unique_id(wallet_unique_id), key.extended_public_key_data(), false)
                .expect("Can't store extended public key")
        } else {
            false
        }*/
        false
    }

    // fn load_identities(&self, address: &String) -> (Option<&Identity>, Option<&Identity>) {
    //     (None, None)Ar
    // }
}

impl<IPATH: IIndexPath> PartialEq for dyn IDerivationPath<IPATH> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}
