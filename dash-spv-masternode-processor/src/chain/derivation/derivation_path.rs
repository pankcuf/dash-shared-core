use std::collections::HashSet;
use std::sync::Weak;
use hashes::hex::ToHex;
use crate::chain::bip::bip32;
use crate::chain::common::ChainType;
use crate::chain::derivation::derivation_path_feature_purpose::DerivationPathFeaturePurpose;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::derivation_path_type::DerivationPathType;
use crate::chain::derivation::index_path::{IIndexPath, IndexPath};
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::derivation::uint256_index_path::UInt256IndexPath;
use crate::chain::derivation::{standalone_extended_public_key_location_string_for_unique_id, standalone_info_dictionary_location_string_for_unique_id, wallet_based_extended_private_key_location_string_for_unique_id, wallet_based_extended_public_key_location_string_for_unique_id};
use crate::chain::wallet::seed::Seed;
use crate::crypto::UInt256;
use crate::keys::IKey;
use crate::keys::key::{Key, KeyKind};
use crate::storage::keychain::Keychain;
use crate::storage::manager::managed_context::ManagedContext;

#[derive(Clone, Debug)]
pub struct DerivationPathInfo {
    pub terminal_index: UInt256,
    pub terminal_hardened: bool,
    pub depth: u8,
}

#[derive(Clone, Debug, Default)]
pub struct DerivationPath {
    pub base: UInt256IndexPath,
    // pub chain: Weak<Chain>,
    pub context: Weak<ManagedContext>,
    pub chain_type: ChainType,
    // pub wallet: Option<Weak<Wallet>>,
    pub wallet_unique_id: Option<String>,
    pub is_transient: bool,

    pub hardened_indexes: Vec<bool>,
    /// is this an open account
    pub r#type: DerivationPathType,
    pub signing_algorithm: KeyKind,

    /// extended Public Key
    pub extended_public_key_data: Vec<u8>,
    /// extended Public Key Identifier, which is just the short hex string of the extended public key
    pub standalone_extended_public_key_unique_id: Option<String>,
    /// the wallet_based_extended_public_key_location_string is the key used to store the public key in the key chain
    // pub wallet_based_extended_public_key_location_string: Option<String>,
    /// the wallet_based_extended_public_key_location_string is the key used to store the private key in the key chain,
    /// this is only available on authentication derivation paths
    // pub wallet_based_extended_private_key_location_string: Option<String>,
    /// current derivation path balance excluding transactions known to be invalid
    pub balance: u64,
    /// purpose of the derivation path if BIP 43 based
    pub purpose: u64,
    /// currently the derivationPath is synced to this block height
    pub sync_block_height: u32,

    /// the reference of type of derivation path
    pub reference: DerivationPathReference,
    /// there might be times where the derivationPath is actually unknown, for example when importing from an extended public key
    pub derivation_path_is_known: bool,

    pub addresses_loaded: bool,

    pub all_addresses: Vec<String>,
    pub used_addresses: Vec<String>,

    pub standalone_extended_public_key_location_string: Option<String>,
    pub wallet_based_extended_public_key_location_string: Option<String>,
    pub wallet_based_extended_private_key_location_string: Option<String>,

    // @property (nonatomic, readonly) DSDerivationPathEntity *derivationPathEntity;

    /// private
    pub depth: u8,
    pub string_representation: Option<String>,

    // master public key used to generate wallet addresses
    extended_public_key: Option<Key>,
}

impl PartialEq for DerivationPath {
    fn eq(&self, other: &Self) -> bool {
        self.standalone_extended_public_key_unique_id.eq(&other.standalone_extended_public_key_unique_id)
    }
}

impl IIndexPath for DerivationPath {
    type Item = UInt256;

    fn new(indexes: Vec<Self::Item>) -> Self {
        Self { base: UInt256IndexPath::new(indexes), ..Default::default() }
    }

    fn new_hardened(indexes: Vec<Self::Item>, hardened: Vec<bool>) -> Self {
        Self { base: UInt256IndexPath::new_hardened(indexes, hardened), ..Default::default() }
    }

    fn indexes(&self) -> &Vec<Self::Item> {
        &self.base.indexes
    }

    fn hardened_indexes(&self) -> &Vec<bool> {
        &self.base.hardened_indexes
    }
}

impl IDerivationPath for DerivationPath {

    fn chain_type(&self) -> ChainType {
        self.chain_type
    }

    fn context(&self) -> Weak<ManagedContext> {
        self.context.clone()
    }

    fn is_transient(&self) -> bool {
        self.is_transient
    }

    fn set_is_transient(&mut self, is_transient: bool) {
        self.is_transient = is_transient;
    }

    fn wallet_unique_id(&self) -> Option<String> {
        self.wallet_unique_id.clone()
    }

    fn set_wallet_unique_id(&mut self, unique_id: String) {
        self.wallet_unique_id = Some(unique_id);
    }

    fn signing_algorithm(&self) -> KeyKind {
        self.signing_algorithm
    }

    fn reference(&self) -> DerivationPathReference {
        self.reference
    }

    fn extended_public_key(&self) -> Option<Key> {
        if let Some(key) = self.extended_public_key.as_ref() {
            return Some(key.clone());
        } else if let Some(wallet_unique_id) = self.wallet_unique_id() {
            let key = self.public_key_location_string_for_wallet_unique_id(wallet_unique_id.as_str());
            println!("extended_public_key.key: {}", key);
            Keychain::get_data(key)
                .ok()
                .and_then(|data| {
                    println!("extended_public_key.data: {}", data.to_hex());
                    self.signing_algorithm().key_with_extended_public_key_data(&data)
                })
        } else {
            None
        }
    }
    fn extended_public_key_mut(&mut self) -> Option<Key> {
        if let Some(key) = self.extended_public_key.as_ref() {
            return Some(key.clone());
        } else if let Some(wallet_unique_id) = self.wallet_unique_id() {
            Keychain::get_data(self.public_key_location_string_for_wallet_unique_id(wallet_unique_id.as_str()))
                .ok()
                .and_then(|data| {
                    self.extended_public_key = self.signing_algorithm().key_with_extended_public_key_data(&data);
                    self.extended_public_key.clone()
                })
        } else {
            None
        }
    }

    fn has_extended_public_key(&self) -> bool {
        if let Some(wallet_unique_id) = self.wallet_unique_id() {
            self.extended_public_key.is_some() || Keychain::has_data(if self.is_wallet_based() {
                wallet_based_extended_public_key_location_string_for_unique_id(wallet_unique_id.as_str())
            } else {
                standalone_extended_public_key_location_string_for_unique_id(wallet_unique_id.as_str())
            }).is_ok()
        } else {
            false
        }
    }

    fn depth(&self) -> u8 {
        if self.depth != 0 {
            self.depth
        } else {
            self.length() as u8
        }
    }

    fn all_addresses(&self) -> HashSet<String> {
        HashSet::from_iter(self.all_addresses.clone().into_iter())
    }

    fn used_addresses(&self) -> HashSet<String> {
        HashSet::from_iter(self.used_addresses.clone().into_iter())
    }

    fn standalone_extended_public_key_unique_id(&mut self) -> Option<String> {
        self.standalone_extended_public_key_unique_id.clone().or({
            if self.extended_public_key.is_none() && self.wallet_unique_id.is_some() {
                assert!(false, "we really should have a wallet");
                None
            } else {
                let id = Some(self.create_identifier_for_derivation_path());
                self.standalone_extended_public_key_unique_id = id.clone();
                id
            }
        })
    }

    fn balance(&self) -> u64 {
        self.balance
    }

    fn set_balance(&mut self, amount: u64) {
        self.balance = amount;
    }

    fn index_path_for_known_address(&self, address: &String) -> Option<IndexPath<u32>> {
        panic!("This must be implemented in subclasses")
    }

    fn generate_extended_public_key_from_seed(&mut self, seed: &Seed) -> Option<Key> {
        self.generate_extended_public_key_from_seed_and_store_private_key(seed, false)
    }

    fn register_transaction_address(&mut self, address: &String) -> bool {
        let has_addr = self.contains_address(address);
        if has_addr && !self.used_addresses.contains(address) {
            self.used_addresses.push(address.clone());
        }
        has_addr
    }
}

impl DerivationPath {
    pub fn master_identity_contacts_derivation_path_for_account_number(account_number: u32, chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self::derivation_path_with_indexes(
            vec![
                UInt256::from(DerivationPathFeaturePurpose::Default),
                UInt256::from(chain_type.coin_type()),
                UInt256::from(DerivationPathFeaturePurpose::DashPay),
                UInt256::from(account_number),
            ],
            vec![true, true, true, true],
            DerivationPathType::PartialPath,
            KeyKind::ECDSA,
            DerivationPathReference::ContactBasedFundsRoot,
            chain_type,
            context
        )
    }

    pub fn derivation_path_with_indexes(indexes: Vec<UInt256>, hardened: Vec<bool>, r#type: DerivationPathType, signing_algorithm: KeyKind, reference: DerivationPathReference, chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        Self {
            base: UInt256IndexPath { indexes, hardened_indexes: hardened },
            r#type,
            signing_algorithm,
            reference,
            chain_type,
            context,
            ..Default::default()
        }
    }

    pub fn derivation_path_with_bip32_key(key: bip32::Key, chain_type: ChainType, context: Weak<ManagedContext>) -> Self {
        let key_type = KeyKind::ECDSA;
        let mut path = Self::derivation_path_with_indexes(
            vec![key.child],
            vec![key.hardened],
            DerivationPathType::ViewOnlyFunds,
            key_type,
            DerivationPathReference::Unknown,
            chain_type,
            context,
        );
        path.extended_public_key = key_type.key_with_extended_public_key_data(&key.extended_key_data());
        path.depth = key.depth;
        path.standalone_save_extended_public_key_to_keychain();
        path.load_addresses();
        path
    }

    // pub fn derivation_path_with_serialized_extended_public_key(key: &String, chain: &SharedChain) -> Option<Self> {
    //     deserialized_extended_public_key_for_chain(key, &chain.params)
    //         .map(|pk| Self::derivation_path_with_bip32_key(pk, Arc::downgrade(chain)))
    //         .ok()
    // }

    /*pub fn init_with_extended_public_key_identifier(extended_public_key_identifier: String, chain: &Chain) -> Option<Self> {
        let key = standalone_info_dictionary_location_string_for_unique_id(&extended_public_key_identifier);
        return if let Ok(info_dictionary) = Keychain::get_dict::<String, KeychainDictValueKind>(key) {
            let terminal_index =
                if let Some(&KeychainDictValueKind::Uint256(terminal_index)) = info_dictionary.get(DERIVATION_PATH_STANDALONE_INFO_TERMINAL_INDEX) {
                    Some(terminal_index)
                } else {
                    None
                };

            let terminal_hardened = if let Some(&KeychainDictValueKind::Bool(terminal_hardened)) = info_dictionary.get(DERIVATION_PATH_STANDALONE_INFO_TERMINAL_HARDENED) {
                Some(terminal_hardened)
            } else {
                None
            };
            if terminal_index.is_none() || terminal_hardened.is_none() {
                return None;
            }
            // TODO: length here is zero! so is not based on indexes length?
            let key_type = KeyKind::ECDSA;
            let mut s = Self {
                base: UInt256IndexPath { indexes: vec![terminal_index.unwrap()], hardened_indexes: vec![terminal_hardened.unwrap()] },
                r#type: DerivationPathType::ViewOnlyFunds,
                signing_algorithm: key_type,
                reference: DerivationPathReference::Unknown,
                chain,
                ..Default::default()
            };
            if let Ok(data) = Keychain::get_data(standalone_extended_public_key_location_string_for_unique_id(&extended_public_key_identifier)) {
                s.extended_public_key = key_type.key_with_extended_public_key_data(&data);
                if let Some(&KeychainDictValueKind::Byte(depth)) = info_dictionary.get(DERIVATION_PATH_STANDALONE_INFO_DEPTH) {
                    s.depth = depth
                } else {
                    return None;
                };
                s.load_addresses();
                Some(s)
            } else {
                None
            }
        } else {
            None
        }
    }*/

    /// Key Generation

    pub fn generate_extended_public_key_from_seed_no_store(&mut self, seed: &Vec<u8>) -> Option<Key> {
        if seed.is_empty() || (self.is_empty() && !DerivationPathReference::Root.eq(&self.reference())) {
            None
        } else {
            self.signing_algorithm()
                .key_with_seed_data(seed)
                .and_then(|seed_key| {
                    self.extended_public_key = seed_key.private_derive_to_256bit_derivation_path(self);
                    assert!(self.extended_public_key.is_some(), "extendedPublicKey should be set");
                    self.extended_public_key.as_mut().map(|extended_public_key| {
                        extended_public_key.forget_private_key();
                        extended_public_key.to_owned()
                    })
                })
        }
    }

    pub fn generate_extended_public_key_from_seed_and_store_private_key(&mut self, seed: &Seed, store_private_key: bool) -> Option<Key> {
        if seed.is_empty() || !self.is_wallet_based() {
            None
        } else {
            let key_type = self.signing_algorithm();
            key_type
                .key_with_seed_data(&seed.data)
                .and_then(|seed_key| {
                    self.extended_public_key = seed_key.private_derive_to_256bit_derivation_path(self);
                    assert!(self.extended_public_key.is_some(), "extendedPublicKey should be set");
                    let index_path_string = self.index_path_enumerated_string();
                    if let Some(pk) = self.extended_public_key.as_mut() {
                        Keychain::save_extended_public_key(
                            seed.unique_id_as_str(),
                            key_type,
                            index_path_string,
                            pk.extended_public_key_data())
                            .expect("Can't store extended_public_key_data in keychain");
                        if store_private_key {
                            Keychain::save_extended_private_key(
                                seed.unique_id_as_str(),
                                pk.extended_private_key_data())
                                .expect("Can't store extended_private_key_data in keychain");
                        }
                        pk.forget_private_key();
                    }
                    self.extended_public_key.to_owned()
                    // self.extended_public_key.as_mut().map(|extended_public_key| {
                    //     Keychain::set_data(
                    //         self.wallet_based_extended_public_key_location_string_for_wallet_unique_id(&wallet_unique_id),
                    //         extended_public_key.extended_public_key_data(),
                    //         false)
                    //         .expect("Can't store extended_public_key_data in keychain");
                    //     if store_private_key {
                    //         Keychain::set_data(
                    //             wallet_based_extended_private_key_location_string_for_unique_id(&wallet_unique_id),
                    //             extended_public_key.extended_private_key_data(),
                    //             true)
                    //             .expect("Can't store extended_private_key_data in keychain");
                    //     }
                    //     extended_public_key.forget_private_key();
                    //     extended_public_key.to_owned()
                    // })
                })
        }
    }
    pub fn generate_extended_public_key_from_parent_derivation_path<DPATH: IDerivationPath + IIndexPath<Item = UInt256>>(&mut self, path: &DPATH, wallet_unique_id: Option<&String>) -> Option<Key> {
        assert_eq!(path.signing_algorithm(), self.signing_algorithm(), "The signing algorithms must be the same");
        assert!(self.length() > path.length(), "length must be inferior to the parent derivation path length");
        assert!(path.has_extended_public_key(), "the parent derivation path must have an extended public key");
        if self.is_empty() ||
            self.length() < path.length() ||
            !path.has_extended_public_key() ||
            path.signing_algorithm() != self.signing_algorithm() {
            return None;
        }
        for i in 0..path.length() {
            let index = self.index_at_position(i);
            assert_eq!(path.index_at_position(i), index, "This derivation path must start with elements of the parent derivation path");
            if path.index_at_position(i) != self.index_at_position(i) {
                return None;
            }
        }
        self.extended_public_key = path.extended_public_key()
            .and_then(|mut ext_pk| ext_pk.public_derive_to_256bit_derivation_path_with_offset(self, path.length()));
        assert!(self.extended_public_key.is_some(), "extendedPublicKey should be set");
        if let Some(unique_id) = wallet_unique_id {
            Keychain::set_data(
                self.wallet_based_extended_public_key_location_string_for_wallet_unique_id(unique_id),
                self.extended_public_key.clone().and_then(|key| key.extended_public_key_data()),
                false)
                .expect("Can't store extended public key");
        }
        self.extended_public_key.clone()
    }

    pub fn serialized_private_keys_at_index_paths(&self, index_paths: Vec<IndexPath<u32>>, seed: &Seed) -> Option<Vec<String>> {
        if seed.is_empty() {
            return None;
        }
        index_paths.is_empty().then_some(vec![])
            .or(self.signing_algorithm()
                .key_with_seed_data(&seed.data)
                .map_or(Some(vec![]), |top_key| top_key.private_derive_to_256bit_derivation_path(self)
                    .map_or(Some(vec![]), |derivation_path_extended_key| Some(index_paths.into_iter()
                            .filter_map(|index_path| derivation_path_extended_key.private_derive_to_path(&index_path)
                                .map(|key| key.serialized_private_key_for_script(&self.chain_type().script_map())))
                            .collect())
                    )))
    }

    pub fn deserialized_extended_private_key_for_chain(extended_private_key_string: &String, chain_type: ChainType) -> Option<Vec<u8>> {
        (extended_private_key_string.as_str(), chain_type)
            .try_into()
            .map(|key: bip32::Key| key.extended_key_data())
            .ok()
    }

    fn standalone_save_extended_public_key_to_keychain(&mut self) {
        /*if let (Some(ex_pk), Some(key)) = (&self.extended_public_key, self.standalone_extended_public_key_location_string()) {
            Keychain::set_data(key, self.extended_public_key_data(), false).expect("");
            let mut map = serde_json::Map::from_iter([
                (DERIVATION_PATH_STANDALONE_INFO_TERMINAL_HARDENED.to_owned(), json!(self.terminal_hardened())),
                (DERIVATION_PATH_STANDALONE_INFO_DEPTH.to_owned(), json!(self.depth)),
            ]);
            if let Some(&terminal_index) = self.indexes().last() {
                map.insert(DERIVATION_PATH_STANDALONE_INFO_TERMINAL_INDEX.to_owned(), json!(terminal_index.0.to_hex()));
            }
            if let Some(key) = self.standalone_info_dictionary_location_string() {
                Keychain::set_json(serde_json::Value::Object(map), key, false).expect("");
            }
            self.context().perform_block_and_wait(|context| {
                DerivationPathEntity::derivation_path_entity_matching_derivation_path(self, context).expect("");
            });
        }*/
    }

    pub fn wallet_based_extended_private_key_location_string(&self) -> String {
        self.wallet_unique_id()
            .map(|unique_id| wallet_based_extended_private_key_location_string_for_unique_id(unique_id.as_str()))
            .unwrap_or(String::new())
    }

    pub fn wallet_based_extended_public_key_location_string(&self) -> String {
        self.wallet_unique_id()
            .map(|unique_id| wallet_based_extended_public_key_location_string_for_unique_id(unique_id.as_str()))
            .unwrap_or(String::new())
    }

    pub fn init_with_extended_public_key_identifier(identifier: &str, chain_type: ChainType, context: Weak<ManagedContext>) -> Option<Self> {
        match Keychain::get_object::<DerivationPathInfo>(standalone_info_dictionary_location_string_for_unique_id(identifier)) {
            Ok(info) => {
                let mut path = DerivationPath::derivation_path_with_indexes(
                    vec![info.terminal_index],
                    vec![info.terminal_hardened],
                    DerivationPathType::ViewOnlyFunds,
                    KeyKind::ECDSA,
                    DerivationPathReference::Unknown,
                    chain_type,
                    context
                );
                path.wallet_based_extended_public_key_location_string = Some(identifier.to_string());
                match Keychain::get_data(standalone_extended_public_key_location_string_for_unique_id(identifier)) {
                    Ok(data) => {
                        path.extended_public_key = path.signing_algorithm().key_with_extended_public_key_data(&data);
                        path.depth = info.depth;
                        path.load_addresses();
                        Some(path)
                    },
                    _ => None
                }
            },
            _ => None
        }
    }

//     - (instancetype _Nullable)initWithExtendedPublicKeyIdentifier:(NSString *_Nonnull)extendedPublicKeyIdentifier onChain:(DSChain *_Nonnull)chain {
//     NSError *error = nil;
//     NSDictionary *infoDictionary = getKeychainDict([DSDerivationPath standaloneInfoDictionaryLocationStringForUniqueID:extendedPublicKeyIdentifier], @[[NSString class], [NSNumber class]], &error);
//     if (error) return nil;
//
//     UInt256 terminalIndex = [((NSData *)infoDictionary[DERIVATION_PATH_STANDALONE_INFO_TERMINAL_INDEX]) UInt256];
//     BOOL terminalHardened = [((NSNumber *)infoDictionary[DERIVATION_PATH_STANDALONE_INFO_TERMINAL_HARDENED]) boolValue];
//     UInt256 indexes[] = {terminalIndex};
//     BOOL hardenedIndexes[] = {terminalHardened};
//     if (!(self = [self initWithIndexes:indexes hardened:hardenedIndexes length:0 type:DSDerivationPathType_ViewOnlyFunds signingAlgorithm:DSKeyType_ECDSA reference:DSDerivationPathReference_Unknown onChain:chain])) return nil;
//     _walletBasedExtendedPublicKeyLocationString = extendedPublicKeyIdentifier;
//     NSData *data = getKeychainData([DSDerivationPath standaloneExtendedPublicKeyLocationStringForUniqueID:extendedPublicKeyIdentifier], &error);
//     if (error) return nil;
//     _extendedPublicKey = [DSKey keyWithExtendedPublicKeyData:data forKeyType:DSKeyType_ECDSA];
//
//     _depth = infoDictionary[DERIVATION_PATH_STANDALONE_INFO_DEPTH];
//
//     [self loadAddresses];
//     return self;
// }

}
