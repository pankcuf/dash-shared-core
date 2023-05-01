use std::fmt::Debug;
use std::ptr::null_mut;
use crate::chain::{ScriptMap, derivation::{IIndexPath, IndexPath}};
use crate::crypto::{UInt256, UInt384, UInt768, byte_util::BytesDecodable};
use crate::keys::{BLSKey, CryptoData, ECDSAKey, ED25519Key, IKey};
use crate::keys::crypto_data::DHKey;
use crate::types::opaque_key::{AsOpaqueKey, OpaqueKey};
use crate::util::sec_vec::SecVec;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyKind {
    ECDSA = 0,
    BLS = 1,
    BLSBasic = 2,
    ED25519 = 3,
}

#[derive(Clone, Debug)]
pub enum Key {
    ECDSA(ECDSAKey),
    BLS(BLSKey),
    ED25519(ED25519Key),
}

impl From<ECDSAKey> for Key {
    fn from(value: ECDSAKey) -> Self {
        Key::ECDSA(value)
    }
}

impl From<BLSKey> for Key {
    fn from(value: BLSKey) -> Self {
        Key::BLS(value)
    }
}

impl From<ED25519Key> for Key {
    fn from(value: ED25519Key) -> Self {
        Key::ED25519(value)
    }
}

impl From<Key> for ECDSAKey {
    fn from(value: Key) -> Self {
        match value {
            Key::ECDSA(key) => key,
            _ => panic!("trying to unwrap bls from different key type")
        }
    }
}
impl From<Key> for BLSKey {
    fn from(value: Key) -> Self {
        match value {
            Key::BLS(key) => key,
            _ => panic!("trying to unwrap ecdsa from different key type")
        }
    }
}
impl From<Key> for ED25519Key {
    fn from(value: Key) -> Self {
        match value {
            Key::ED25519(key) => key,
            _ => panic!("trying to unwrap ed25519 from different key type")
        }
    }
}

impl Default for KeyKind {
    fn default() -> Self {
        KeyKind::ECDSA
    }
}

impl From<i16> for KeyKind {
    fn from(orig: i16) -> Self {
        match orig {
            0 => KeyKind::ECDSA,
            1 => KeyKind::BLS,
            2 => KeyKind::BLSBasic,
            3 => KeyKind::ED25519,
            _ => KeyKind::default(),
        }
    }
}

impl From<KeyKind> for i16 {
    fn from(value: KeyKind) -> Self {
        match value {
            KeyKind::ECDSA => 0,
            KeyKind::BLS => 1,
            KeyKind::BLSBasic => 2,
            KeyKind::ED25519 => 3,
        }
    }
}

impl From<&KeyKind> for u8 {
    fn from(value: &KeyKind) -> Self {
        match value {
            KeyKind::ECDSA => 0,
            KeyKind::BLS => 1,
            KeyKind::BLSBasic => 2,
            KeyKind::ED25519 => 3,
        }
    }
}

impl KeyKind {

    pub fn derivation_string(&self) -> String {
        match self {
            KeyKind::ECDSA => "",
            KeyKind::ED25519 => "_ED_",
            KeyKind::BLS | KeyKind::BLSBasic  => "_BLS_",
        }.to_string()
    }

    pub(crate) fn public_key_from_extended_public_key_data(&self, data: &[u8], index_path: &IndexPath<u32>) -> Option<Vec<u8>> {
        match self {
            KeyKind::ECDSA => ECDSAKey::public_key_from_extended_public_key_data(data, index_path),
            KeyKind::ED25519 => ED25519Key::public_key_from_extended_public_key_data(data, index_path),
            KeyKind::BLS => BLSKey::public_key_from_extended_public_key_data(data, index_path, true),
            KeyKind::BLSBasic => BLSKey::public_key_from_extended_public_key_data(data, index_path, false),
        }
    }

    pub(crate) fn private_key_from_extended_private_key_data(&self, data: &Vec<u8>) -> Option<Key> {
        match self {
            KeyKind::ECDSA => ECDSAKey::init_with_extended_private_key_data(data).map(Key::ECDSA),
            KeyKind::ED25519 => ED25519Key::init_with_extended_private_key_data(data).map(Key::ED25519),
            KeyKind::BLS => BLSKey::init_with_extended_private_key_data(data, true).map(Key::BLS),
            KeyKind::BLSBasic => BLSKey::init_with_extended_private_key_data(data, false).map(Key::BLS),
        }
    }

    pub(crate) fn key_with_private_key_data(&self, data: &[u8]) -> Option<Key> {
        match self {
            KeyKind::ECDSA => ECDSAKey::key_with_secret_data(data, true).map(Key::ECDSA),
            KeyKind::ED25519 => ED25519Key::key_with_secret_data(data, true).map(Key::ED25519),
            KeyKind::BLS => BLSKey::key_with_private_key_data(data, true).map(Key::BLS),
            KeyKind::BLSBasic => BLSKey::key_with_private_key_data(data, false).map(Key::BLS),
        }
    }

    pub(crate) fn key_with_seed_data(&self, seed: &[u8]) -> Option<Key> {
        match self {
            KeyKind::ECDSA => ECDSAKey::init_with_seed_data(seed).map(Key::ECDSA),
            KeyKind::ED25519 => ED25519Key::init_with_seed_data(seed).map(Key::ED25519),
            KeyKind::BLS => BLSKey::extended_private_key_with_seed_data(seed, true).map(Key::BLS),
            KeyKind::BLSBasic => BLSKey::extended_private_key_with_seed_data(seed, false).map(Key::BLS),
        }
    }

    pub(crate) fn key_with_public_key_data(&self, data: &[u8]) -> Option<Key> {
        match self {
            KeyKind::ECDSA => ECDSAKey::key_with_public_key_data(data).map(Key::ECDSA),
            KeyKind::ED25519 => ED25519Key::key_with_public_key_data(data).map(Key::ED25519),
            KeyKind::BLS => Some(Key::BLS(BLSKey::key_with_public_key(UInt384::from_bytes(data, &mut 0).unwrap(), true))),
            KeyKind::BLSBasic => Some(Key::BLS(BLSKey::key_with_public_key(UInt384::from_bytes(data, &mut 0).unwrap(), false))),
        }
    }

    pub(crate) fn key_with_extended_public_key_data(&self, data: &Vec<u8>) -> Option<Key> {
        match self {
            KeyKind::ECDSA => ECDSAKey::init_with_extended_public_key_data(data).map(Key::ECDSA),
            KeyKind::ED25519 => ED25519Key::init_with_extended_public_key_data(data).map(Key::ED25519),
            KeyKind::BLS => BLSKey::init_with_extended_public_key_data(data, true).map(Key::BLS),
            KeyKind::BLSBasic => BLSKey::init_with_extended_public_key_data(data, false).map(Key::BLS),
        }
    }

    pub(crate) fn key_with_extended_private_key_data(&self, data: &Vec<u8>) -> Option<Key> {
        match self {
            KeyKind::ECDSA => ECDSAKey::init_with_extended_private_key_data(data).map(Key::ECDSA),
            KeyKind::ED25519 => ED25519Key::init_with_extended_private_key_data(data).map(Key::ED25519),
            KeyKind::BLS => BLSKey::init_with_extended_private_key_data(data, true).map(Key::BLS),
            KeyKind::BLSBasic => BLSKey::init_with_extended_private_key_data(data, false).map(Key::BLS),
        }
    }

    /*pub fn private_derive_to_256bit_derivation_path_from_seed_and_store<IPATH, DPATH>(&self, seed: &Seed, derivation_path: &DPATH, store_private_key: bool) -> Option<Key>
        where IPATH: IIndexPath, DPATH: IDerivationPath + IIndexPath<Item = UInt256>  {
        if let Some(seed_key) = self.key_with_seed_data(&seed.data) {
            println!("private_derive_to_256bit_derivation_path_from_seed_and_store: seed_key: {:?}", seed_key.clone());
            let derived = seed_key.private_derive_to_256bit_derivation_path(derivation_path);
            if let Some(mut ext_pk) = derived {
                let ext_pub_data = ext_pk.extended_private_key_data();
                let ext_prv_data = ext_pk.extended_public_key_data();
                println!("private_derive_to_256bit_derivation_path_from_seed_and_store: ext_prv_data: {:?} ext_pub_data: {:?}", ext_prv_data.clone(), ext_pub_data.clone());
                if !seed.unique_id.is_empty() {
                    Keychain::set_data(derivation_path.wallet_based_extended_public_key_location_string_for_wallet_unique_id(seed.unique_id_as_str()), ext_pub_data, false)
                        .expect("");
                    if store_private_key {
                        Keychain::set_data(wallet_based_extended_private_key_location_string_for_unique_id(seed.unique_id_as_str()), ext_prv_data, true)
                            .expect("");
                    }
                }
                ext_pk.forget_private_key();
                Some(ext_pk)
            } else {
                None
            }
        } else {
            None
        }
    }*/
}

impl IKey for Key {
    // type SK = T;

    fn r#type(&self) -> KeyKind {
        match self {
            Key::ECDSA(..) => KeyKind::ECDSA,
            Key::ED25519(..) => KeyKind::ED25519,
            Key::BLS(key) => if key.use_legacy { KeyKind::BLS } else { KeyKind::BLSBasic }
        }
    }

    fn sign(&self, data: &[u8]) -> Vec<u8> {
        match self {
            Key::ECDSA(key) => key.compact_sign(UInt256::from(data)).to_vec(),
            Key::BLS(key) => key.sign(data),
            Key::ED25519(key) => key.sign(data)
        }
    }

    fn verify(&mut self, message_digest: &[u8], signature: &[u8]) -> bool {
        match self {
            Key::ECDSA(key) => key.verify(message_digest, signature),
            Key::BLS(key) => key.verify_uint768(UInt256::from(message_digest), UInt768::from(signature)),
            Key::ED25519(key) => key.verify(message_digest, signature),
        }
    }

    fn secret_key(&self) -> UInt256 {
        match self {
            Key::ECDSA(key) => key.seckey,
            Key::BLS(key) => key.secret_key(),
            Key::ED25519(key) => key.secret_key(),
        }
    }

    fn chaincode(&self) -> UInt256 {
        match self {
            Key::ECDSA(key) => key.chaincode(),
            Key::BLS(key) => key.chaincode(),
            Key::ED25519(key) => key.chaincode(),
        }
    }

    fn fingerprint(&self) -> u32 {
        match self {
            Key::ECDSA(key) => key.fingerprint(),
            Key::BLS(key) => key.fingerprint(),
            Key::ED25519(key) => key.fingerprint(),
        }
    }

    fn private_key_data(&self) -> Option<Vec<u8>> {
        match self {
            Key::ECDSA(key) => key.private_key_data(),
            Key::BLS(key) => key.private_key_data(),
            Key::ED25519(key) => key.private_key_data(),
        }
    }

    fn public_key_data(&self) -> Vec<u8> {
        match self {
            Key::ECDSA(key) => key.public_key_data(),
            Key::BLS(key) => key.public_key_data(),
            Key::ED25519(key) => key.public_key_data(),
        }
    }

    fn extended_private_key_data(&self) -> Option<SecVec> {
        match self {
            Key::ECDSA(key) => key.extended_private_key_data(),
            Key::BLS(key) => key.extended_private_key_data(),
            Key::ED25519(key) => key.extended_private_key_data(),
        }
    }

    fn extended_public_key_data(&self) -> Option<Vec<u8>> {
        match self {
            Key::ECDSA(key) => key.extended_public_key_data(),
            Key::BLS(key) => key.extended_public_key_data(),
            Key::ED25519(key) => key.extended_public_key_data(),
        }
    }

    fn private_derive_to_path<PATH>(&self, index_path: &PATH) -> Option<Self>
        where PATH: IIndexPath<Item = u32> {
        match self {
            Key::ECDSA(key) => key.private_derive_to_path(index_path).map(Into::into),
            Key::BLS(key) => key.private_derive_to_path(index_path).map(Into::into),
            Key::ED25519(key) => key.private_derive_to_path(index_path).map(Into::into),
        }
    }

    fn private_derive_to_256bit_derivation_path<PATH>(&self, path: &PATH) -> Option<Self>
        where Self: Sized, PATH: IIndexPath<Item=UInt256> {
        match self {
            Key::ECDSA(key) => key.private_derive_to_256bit_derivation_path(path).map(Into::into),
            Key::BLS(key) => key.private_derive_to_256bit_derivation_path(path).map(Into::into),
            Key::ED25519(key) => key.private_derive_to_256bit_derivation_path(path).map(Into::into),
        }
    }

    fn public_derive_to_256bit_derivation_path<PATH>(&mut self, derivation_path: &PATH) -> Option<Self>
        where Self: Sized, PATH: IIndexPath<Item = UInt256> {
        match self {
            Key::ECDSA(key) => key.public_derive_to_256bit_derivation_path(derivation_path).map(Into::into),
            Key::BLS(key) => key.public_derive_to_256bit_derivation_path(derivation_path).map(Into::into),
            Key::ED25519(key) => key.public_derive_to_256bit_derivation_path(derivation_path).map(Into::into),
        }
    }

    fn public_derive_to_256bit_derivation_path_with_offset<PATH>(&mut self, derivation_path: &PATH, offset: usize) -> Option<Self>
        where Self: Sized, PATH: IIndexPath<Item = UInt256> {
        match self {
            Key::ECDSA(key) => key.public_derive_to_256bit_derivation_path_with_offset(derivation_path, offset).map(Into::into),
            Key::BLS(key) => key.public_derive_to_256bit_derivation_path_with_offset(derivation_path, offset).map(Into::into),
            Key::ED25519(key) => key.public_derive_to_256bit_derivation_path_with_offset(derivation_path, offset).map(Into::into),
        }
    }

    fn serialized_private_key_for_script(&self, script: &ScriptMap) -> String {
        match self {
            Key::ECDSA(key) => key.serialized_private_key_for_script(script),
            Key::BLS(key) => key.serialized_private_key_for_script(script),
            Key::ED25519(key) => key.serialized_private_key_for_script(script),
        }
    }

    fn hmac_256_data(&self, data: &[u8]) -> UInt256 {
        match self {
            Key::ECDSA(key) => key.hmac_256_data(data),
            Key::BLS(key) => key.hmac_256_data(data),
            Key::ED25519(key) => key.hmac_256_data(data),
        }
    }

    fn forget_private_key(&mut self) {
        match self {
            Key::ECDSA(key) => key.forget_private_key(),
            Key::BLS(key) => key.forget_private_key(),
            Key::ED25519(key) => key.forget_private_key(),
        }
    }
}

// TODO: do we need it for ED25519?
impl DHKey for Key {
    fn init_with_dh_key_exchange_with_public_key(public_key: &mut Self, private_key: &Self) -> Option<Self> where Self: Sized {
        match (public_key, private_key) {
            (Key::ECDSA(public_key), Key::ECDSA(private_key)) =>
                ECDSAKey::init_with_dh_key_exchange_with_public_key(public_key, private_key)
                    .map(Into::into),
            (Key::BLS(public_key), Key::BLS(private_key)) =>
                BLSKey::init_with_dh_key_exchange_with_public_key(public_key, private_key)
                    .map(Into::into),
            _ => None
        }
    }
}

impl CryptoData<Key> for Vec<u8> {
    fn encrypt_with_secret_key_using_iv(&mut self, secret_key: &Key, public_key: &Key, initialization_vector: Vec<u8>) -> Option<Vec<u8>> {
        match (secret_key, public_key) {
            (Key::ECDSA(secret_key), Key::ECDSA(public_key)) =>
                <Vec<u8> as CryptoData<ECDSAKey>>::encrypt_with_secret_key_using_iv(self, secret_key, public_key, initialization_vector),
            (Key::BLS(secret_key), Key::BLS(public_key)) =>
                <Vec<u8> as CryptoData<BLSKey>>::encrypt_with_secret_key_using_iv(self, secret_key, public_key, initialization_vector),
            // (Key::ED25519(secret_key), Key::ED25519(public_key)) =>
            //     <Vec<u8> as CryptoData<ED25519Key>>::encrypt_with_secret_key_using_iv(self, secret_key, public_key, initialization_vector),
            _ => None
        }
    }

    fn decrypt_with_secret_key_using_iv_size(&mut self, secret_key: &Key, public_key: &Key, iv_size: usize) -> Option<Vec<u8>> {
        match (secret_key, public_key) {
            (Key::ECDSA(secret_key), Key::ECDSA(public_key)) =>
                <Vec<u8> as CryptoData<ECDSAKey>>::decrypt_with_secret_key_using_iv_size(self, secret_key, public_key, iv_size),
            (Key::BLS(secret_key), Key::BLS(public_key)) =>
                <Vec<u8> as CryptoData<BLSKey>>::decrypt_with_secret_key_using_iv_size(self, secret_key, public_key, iv_size),
            // (Key::ED25519(secret_key), Key::ED25519(public_key)) =>
            //     <Vec<u8> as CryptoData<ED25519Key>>::decrypt_with_secret_key_using_iv_size(self, secret_key, public_key, iv_size),
            _ => None
        }
    }

    fn encrypt_with_dh_key_using_iv(&self, key: &Key, initialization_vector: Vec<u8>) -> Option<Vec<u8>> where Key: DHKey {
        match key {
            Key::ECDSA(key) =>
                <Vec<u8> as CryptoData<ECDSAKey>>::encrypt_with_dh_key_using_iv(self, key, initialization_vector),
            Key::BLS(key) =>
                <Vec<u8> as CryptoData<BLSKey>>::encrypt_with_dh_key_using_iv(self, key, initialization_vector),
            _ => None
            // Key::ED25519(key) =>
            //     <Vec<u8> as CryptoData<ED25519Key>>::encrypt_with_dh_key_using_iv(self, key, initialization_vector),
        }
    }
    fn decrypt_with_dh_key_using_iv_size(&self, key: &Key, iv_size: usize) -> Option<Vec<u8>> where Key: DHKey {
        match key {
            Key::ECDSA(key) =>
                <Vec<u8> as CryptoData<ECDSAKey>>::decrypt_with_dh_key_using_iv_size(self, key, iv_size),
            Key::BLS(key) =>
                <Vec<u8> as CryptoData<BLSKey>>::decrypt_with_dh_key_using_iv_size(self, key, iv_size),
            _ => None
            // Key::ED25519(key) =>
            //     <Vec<u8> as CryptoData<ED25519Key>>::decrypt_with_dh_key_using_iv_size(self, key, iv_size),
        }
    }
}
impl AsOpaqueKey for Key {
    fn to_opaque_ptr(self) -> *mut OpaqueKey {
        match self {
            Key::ECDSA(key) => key.to_opaque_ptr(),
            Key::BLS(key) => key.to_opaque_ptr(),
            Key::ED25519(key) => key.to_opaque_ptr(),
        }
    }
}

impl AsOpaqueKey for Option<Key> {
    fn to_opaque_ptr(self) -> *mut OpaqueKey {
        match self {
            Some(Key::ECDSA(key)) => key.to_opaque_ptr(),
            Some(Key::BLS(key)) => key.to_opaque_ptr(),
            Some(Key::ED25519(key)) => key.to_opaque_ptr(),
            _ => null_mut()
        }
    }
}

