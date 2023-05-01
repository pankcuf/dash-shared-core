use std::collections::HashSet;
use std::sync::Weak;
use std::thread;
use std::time::Duration;
use crate::chain::common::ChainType;
use crate::chain::derivation::derivation_path::DerivationPath;
use crate::chain::derivation::derivation_path_feature_purpose::DerivationPathFeaturePurpose;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::derivation_path_type::DerivationPathType;
use crate::chain::derivation::index_path::{IIndexPath, IndexPath};
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::derivation::sequence_gap_limit::SequenceGapLimit;
use crate::chain::wallet::seed::Seed;
use crate::crypto::UInt256;
use crate::keys::key::{Key, KeyKind};
use crate::platform::identity::identity::Identity;
use crate::storage::manager::managed_context::ManagedContext;
use crate::util;
use crate::util::address::address;

#[derive(Clone, Debug, Default)]
pub struct IncomingFundsDerivationPath {
    pub base: DerivationPath,
    pub contact_source_blockchain_identity_unique_id: UInt256,
    pub contact_destination_blockchain_identity_unique_id: UInt256,
    pub contact_source_blockchain_identity: Identity,
    pub contact_destination_blockchain_identity: Identity,
    pub source_is_local: bool,
    pub destination_is_local: bool,
    account_number: u32,

    external_derivation_path: bool,
    external_addresses: Vec<String>,
}

impl PartialEq for IncomingFundsDerivationPath {
    fn eq(&self, other: &Self) -> bool {
        self.base.standalone_extended_public_key_unique_id.eq(&other.base.standalone_extended_public_key_unique_id)
    }
}

impl IIndexPath for IncomingFundsDerivationPath {
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

impl IDerivationPath for IncomingFundsDerivationPath {

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
        if let Some(context) = self.base.context.upgrade() {
            self.load_addresses_in_context(&context)
        }
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
        self.all_receive_addresses().iter().position(|x| x == address)
            .map(|pos| IndexPath::index_path_with_indexes(vec![pos as u32]))
    }

    fn generate_extended_public_key_from_seed(&mut self, seed: &Seed) -> Option<Key> {
        self.base.generate_extended_public_key_from_seed(seed)
    }

    fn register_transaction_address(&mut self, address: &String) -> bool {
        let contains = self.contains_address(address);
        if contains && !self.used_addresses().contains(address) {
            self.used_addresses().insert(address.clone());
            if let Some(context) = self.base.context.upgrade() {
                self.register_addresses_with_gap_limit(SequenceGapLimit::External.default(), &context)
                    .expect("Error register_addresses_with_gap_limit");
            }
        }
        contains
    }

    fn create_identifier_for_derivation_path(&mut self) -> String {
        format!("{}-{}-{}", self.contact_source_blockchain_identity_unique_id.short_hex(), self.contact_destination_blockchain_identity_unique_id.short_hex(),
                self.base.create_identifier_for_derivation_path())
    }
}

impl IncomingFundsDerivationPath {
    pub fn contact_based_derivation_path_with_destination_identity_unique_id(destination_identity_unique_id: UInt256, source_identity_unique_id: UInt256, account_number: u32, chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        assert_ne!(source_identity_unique_id, destination_identity_unique_id, "source and destination must be different");
        Self {
            base: DerivationPath::derivation_path_with_indexes(
                vec![
                    UInt256::from(DerivationPathFeaturePurpose::Default),
                    UInt256::from(chain_type.coin_type()),
                    UInt256::from(DerivationPathFeaturePurpose::DashPay),
                    UInt256::from(account_number),
                    source_identity_unique_id,
                    destination_identity_unique_id
                ],
                vec![true, true, true, true, false, false],
                DerivationPathType::ClearFunds,
                KeyKind::ECDSA,
                DerivationPathReference::ContactBasedFunds,
                chain_type,
                context
            ),
            account_number,
            contact_source_blockchain_identity_unique_id: source_identity_unique_id,
            contact_destination_blockchain_identity_unique_id: destination_identity_unique_id,
            ..Default::default()
        }
    }
}

impl IncomingFundsDerivationPath  {

    pub fn load_addresses_in_context(&mut self, context: &ManagedContext) {
        // TODO: store addresses
        // if !self.base.addresses_loaded {
        //     match DerivationPathEntity::derivation_path_entity_matching_derivation_path(self, context) {
        //         Ok(derivation_path_entity) => {
        //             self.base.sync_block_height = derivation_path_entity.sync_block_height as u32;
        //             match derivation_path_entity.get_addresses(context) {
        //                 Ok(addresses) => {
        //                     for e in addresses {
        //                         let mut a = self.external_addresses.clone();
        //                         while e.index as usize >= a.len() {
        //                             a.push(String::new());
        //                         }
        //                         if !Address::is_valid_dash_address_for_script_map(&e.address, self.base.account.unwrap().wallet.unwrap().chain.script()) {
        //                             continue;
        //                         }
        //                         a[e.index as usize] = e.address;
        //                         self.base.all_addresses.push(e.address.clone());
        //                         if e.count_used_in_inputs(context).unwrap_or(0) > 0 || e.count_used_in_outputs(context).unwrap_or(0) > 0 {
        //                             self.base.used_addresses.push(e.address.clone());
        //                         }
        //
        //                         if let Ok(count @ 1..=usize::MAX) = e.count_used_in_inputs(context) {}
        //                     }
        //                     self.base.addresses_loaded = true;
        //                     let _ = self.register_addresses_with_gap_limit(SequenceGapLimit::Initial.dashpay(), context);
        //                 },
        //                 Err(err) => println!("Error retrieving addresses for derivation path entity {:?}", self)
        //             }
        //         },
        //         Err(err) => println!("Error retrieving derivation path entity for {:?}", self)
        //     }
        // }
    }

    // Wallets are composed of chains of addresses. Each chain is traversed until a gap of a certain number of addresses is
    // found that haven't been used in any transactions. This method returns an array of <gapLimit> unused addresses
    // following the last used address in the chain. The internal chain is used for change addresses and the external chain
    // for receive addresses.
    pub fn register_addresses_with_gap_limit(&mut self, gap_limit: u32, context: &ManagedContext) -> Result<Vec<String>, util::Error> {
        if self.wallet_unique_id().is_some() {
            if !self.is_transient() {
                if !self.base.addresses_loaded {
                    // quite hacky, we need to fix this
                    thread::sleep(Duration::from_millis(1));
                    // todo: impl waiting for addresses
                    return self.register_addresses_with_gap_limit(gap_limit, context);
                }
                assert!(self.base.addresses_loaded, "addresses must be loaded before calling this function");
            }
            let mut array = self.external_addresses.clone();
            let mut i = array.len();
            // keep only the trailing contiguous block of addresses with no transactions
            while i > 0 && !self.base.used_addresses.contains(&array[i - 1]) {
                i -= 1;
            }
            if i > 0 {
                array.drain(0..i);
            }
            let limit = gap_limit as usize;
            if array.len() >= limit {
                return Ok(array.drain(0..limit).collect());
            }

            if gap_limit > 1 {
                // get receiveAddress and changeAddress first to avoid blocking
                let _ = self.receive_address_in_context(context);
            }

            // It seems weird to repeat this, but it's correct because of the original call receive address and change address
            array = self.external_addresses.clone();
            i = array.len();

            let mut n = i as u32;

            // keep only the trailing contiguous block of addresses with no transactions
            while i > 0 && !self.used_addresses().contains(array.get(i - 1).unwrap()) {
                i -= 1;
            }
            if i > 0 {
                array.drain(0..i);
            }
            if array.len() >= limit {
                return Ok(array.drain(0..limit).collect());
            }
            let mut upper_limit = limit;
            while array.len() < upper_limit {
                // generate new addresses up to gapLimit
                if let Some(pub_key_data) = self.public_key_data_at_index(n) {
                    let pub_key = KeyKind::ECDSA.key_with_public_key_data(&pub_key_data);
                    let address = address::with_public_key_data(&pub_key_data, &self.chain_type().script_map());
                    let is_used = false;
                    // TODO: impl storage
                    /*if !wallet.is_transient() {
                        // store new address in core data
                        if let Ok(derivation_path_entity) = DerivationPathEntity::derivation_path_entity_matching_derivation_path(self, context) {
                            if let Ok(created) = AddressEntity::create_with(derivation_path_entity.id, address.as_str(), n as i32, false, false, context) {
                                if let Ok(outputs) = TransactionOutputEntity::get_by_address(&address.as_bytes().to_vec(), context) {
                                    if !outputs.is_empty() {
                                        is_used = true;
                                    }
                                }
                            }
                        }
                    }*/
                    if is_used {
                        self.base.used_addresses.push(address.clone());
                        upper_limit += 1;
                    }
                    self.base.all_addresses.push(address.clone());
                    self.external_addresses.push(address.clone());
                    array.push(address.clone());
                    n += 1;
                } else {
                    println!("error generating keys");
                    return Err(util::Error::Default(format!("Error generating public keys")));
                }
            }
            Ok(array)
        } else {
            Err(util::Error::Default(format!("Error generating public keys (no wallet)")))
        }
    }

    /// gets an address at an index path
    pub fn address_at_index(&mut self, index: u32) -> Option<String> {
        self.public_key_data_at_index(index)
            .map(|pub_key| address::with_public_key_data(&pub_key, &self.chain_type().script_map()))
    }

    /// returns the first unused external address
    pub fn receive_address(&mut self) -> Option<String> {
        self.base.context().upgrade()
            .and_then(|ref context| self.receive_address_in_context(context))
    }

    pub fn receive_address_in_context(&mut self, context: &ManagedContext) -> Option<String> {
        self.receive_address_at_offset_in_context(0, context)
    }

    pub fn receive_address_at_offset(&mut self, offset: u32) -> Option<String> {
        self.base.context.upgrade().and_then(|ref context| self.receive_address_at_offset_in_context(offset, context))
    }

    pub fn receive_address_at_offset_in_context(&mut self, offset: u32, context: &ManagedContext) -> Option<String> {
        // TODO: limit to 10,000 total addresses and utxos for practical usability with bloom filters
        self.register_addresses_with_gap_limit(offset + 1, context)
            .ok()
            .and_then(|addresses| addresses.iter().last().cloned())
            .or(self.all_receive_addresses().iter().last().cloned())
    }

    /// all previously generated external addresses
    pub fn all_receive_addresses(&self) -> Vec<String> {
        self.external_addresses.clone()
    }

    pub fn used_receive_addresses(&self) -> Vec<String> {
        HashSet::from_iter(self.all_receive_addresses().into_iter()).intersection(&self.used_addresses()).cloned().collect()
    }

    pub fn public_key_data_at_index(&mut self, index: u32) -> Option<Vec<u8>> {
        self.public_key_data_at_index_path(&IndexPath::index_path_with_indexes(vec![index]))
    }

    pub fn private_key_string_at_index(&self, index: u32, seed: &Seed) -> Option<String> {
        self.serialized_private_keys(vec![index], seed)
            .and_then(|keys| keys.iter().last().cloned())
    }

    pub fn private_keys(&self, indexes: Vec<u32>, seed: &Seed) -> Vec<Key> {
        self.private_keys_at_index_paths(
            indexes.iter()
                .map(|&index| IndexPath::index_path_with_indexes(vec![index]))
                .collect(),
            seed)
    }

    pub fn serialized_private_keys(&self, indexes: Vec<u32>, seed: &Seed) -> Option<Vec<String>> {
        self.base.serialized_private_keys_at_index_paths(indexes.iter().map(|&index| IndexPath::index_path_with_indexes(vec![index])).collect(), seed)
    }


    // pub fn contact_source_blockchain_identity(&self) -> Option<Identity> {
    //     self.chain().upgrade().and_then(|chain| chain.identity_for_unique_id_in_wallet_including_foreign_identites(self.contact_source_blockchain_identity_unique_id, true))
    // }
    //
    // pub fn contact_destination_blockchain_identity(&self) -> Option<Identity> {
    //     self.chain().upgrade().and_then(|chain| chain.identity_for_unique_id_in_wallet_including_foreign_identites(self.contact_destination_blockchain_identity_unique_id, true))
    // }
}
