use bls_signatures::bip32::{ExtendedPrivateKey, ExtendedPublicKey};
use bls_signatures::{BasicSchemeMPL, BlsError, G1Element, G2Element, LegacySchemeMPL, PrivateKey, Scheme};
use hashes::{Hash, hex::FromHex, sha256, sha256d};
use crate::chain::{derivation::IIndexPath, ScriptMap};
use crate::consensus::Encodable;
use crate::crypto::{UInt256, UInt384, UInt768, byte_util::{AsBytes, BytesDecodable, Zeroable}, UInt160};
use crate::keys::{IKey, KeyKind, dip14::{IChildKeyDerivation, SignKey}};
use crate::keys::crypto_data::{CryptoData, DHKey};
use crate::models::OperatorPublicKey;
use crate::util::{base58, data_ops::hex_with_data, sec_vec::SecVec};

#[derive(Clone, Debug, Default)]
pub struct BLSKey {
    pub seckey: UInt256,
    pub chaincode: UInt256,
    pub pubkey: UInt384,
    pub extended_private_key_data: SecVec,
    pub extended_public_key_data: Vec<u8>,
    pub use_legacy: bool,
}

impl BLSKey {

    pub fn key_with_secret_hex(string: &str, use_legacy: bool) -> Option<Self> {
        Vec::from_hex(string)
            .ok()
            .map(|data| Self::key_with_seed_data(&data, use_legacy))
    }
    pub fn key_with_private_key(string: &str, use_legacy: bool) -> Option<Self> {
        Vec::from_hex(string)
            .ok()
            .and_then(|data| Self::key_with_private_key_data(&data, use_legacy))
    }

    pub fn key_with_private_key_data(data: &[u8], use_legacy: bool) -> Option<Self> {
        UInt256::from_bytes(data, &mut 0)
            .and_then(|seckey| PrivateKey::from_bytes(data, use_legacy)
                .ok()
                .and_then(|bls_private_key| bls_private_key
                    .g1_element()
                    .ok()
                    .map(|bls_public_key| Self {
                        seckey,
                        pubkey: UInt384(*if use_legacy { bls_public_key.serialize_legacy() } else { bls_public_key.to_bytes() }),
                        use_legacy,
                        ..Default::default()
                    })))
    }

    pub fn key_with_public_key(pubkey: UInt384, use_legacy: bool) -> Self {
        Self { pubkey, use_legacy, ..Default::default() }
    }

    pub fn product(&self, public_key: &BLSKey) -> Option<[u8; 48]> {
        match (self.bls_private_key(), public_key.bls_public_key(), self.use_legacy) {
            (Ok(priv_key), Ok(pub_key), use_legacy) if public_key.use_legacy == use_legacy =>
                (priv_key * pub_key).map(|pk| if use_legacy { *pk.serialize_legacy() } else { *pk.to_bytes() }).ok(),
            _ => None
        }
    }
}

impl IKey for BLSKey {
    fn r#type(&self) -> KeyKind {
        KeyKind::BLS // &KeyType::BLSBasic
    }
    fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.sign_digest(UInt256::from(data)).as_bytes().to_vec()
    }
    fn verify(&mut self, message_digest: &[u8], signature: &[u8]) -> bool {
        self.verify_uint768(UInt256::from(message_digest), UInt768::from(signature))
    }

    fn secret_key(&self) -> UInt256 {
        self.seckey
    }

    fn chaincode(&self) -> UInt256 {
        self.chaincode
    }

    fn fingerprint(&self) -> u32 {
        self.public_key_fingerprint()
    }

    fn private_key_data(&self) -> Option<Vec<u8>> where Self: Sized {
        (!self.seckey.is_zero())
            .then_some(self.seckey.0.to_vec())
    }

    fn public_key_data(&self) -> Vec<u8> {
        self.pubkey.0.to_vec()
    }

    fn extended_private_key_data(&self) -> Option<SecVec> {
        Some(self.extended_private_key_data.clone())
    }

    fn extended_public_key_data(&self) -> Option<Vec<u8>> {
        Some(self.extended_public_key_data.clone())
    }

    fn private_derive_to_path2<SK, PK, PATH, INDEX>(&self, path: &PATH) -> Option<Self>
        where Self: Sized + IChildKeyDerivation<INDEX, SK, PK>,
              PATH: IIndexPath<Item=INDEX>, SK: SignKey {
        todo!()
    }

    fn private_derive_to_path<PATH>(&self, path: &PATH) -> Option<Self>
        where Self: Sized, PATH: IIndexPath<Item = u32> {
        ExtendedPrivateKey::from_bytes(self.extended_private_key_data.as_slice())
            .ok()
            .and_then(|bls_extended_private_key|
                Self::init_with_bls_extended_private_key(&Self::derive(bls_extended_private_key, path, self.use_legacy), self.use_legacy))
    }

    fn serialized_private_key_for_script(&self, script: &ScriptMap) -> String {
        // if (uint256_is_zero(self.secretKey)) return nil;
        // NSMutableData *d = [NSMutableData secureDataWithCapacity:sizeof(UInt256) + 2];
        let mut writer = SecVec::with_capacity(34);
        script.privkey.enc(&mut writer);
        self.seckey.enc(&mut writer);
        b'\x02'.enc(&mut writer);
        base58::check_encode_slice(&writer)
    }

    fn hmac_256_data(&self, data: &[u8]) -> UInt256 {
        UInt256::hmac::<sha256::Hash>(self.seckey.as_bytes(), data)
    }

    fn forget_private_key(&mut self) {
        self.seckey = UInt256::MIN;
    }
}

impl BLSKey {

    pub fn init_with_extended_private_key_data(data: &Vec<u8>, use_legacy: bool) -> Option<Self> {
        ExtendedPrivateKey::from_bytes(data)
            .ok()
            .and_then(|pk| Self::init_with_bls_extended_private_key(&pk, use_legacy))
    }

    pub fn init_with_extended_public_key_data(data: &Vec<u8>, use_legacy: bool) -> Option<Self> {
        if use_legacy {
            ExtendedPublicKey::from_bytes_legacy(data)
        } else {
            ExtendedPublicKey::from_bytes(data)
        }.ok().map(|pk| Self::init_with_bls_extended_public_key(&pk, use_legacy))
    }

    /// A little recursive magic since extended private keys can't be re-assigned in the library
    pub fn derive<PATH>(extended_private_key: ExtendedPrivateKey, path: &PATH, use_legacy: bool) -> ExtendedPrivateKey
        where PATH: IIndexPath<Item = u32> {
        if path.is_empty() {
            extended_private_key
        } else {
            let top_index_path = path.index_at_position(0);
            let sk_child = if use_legacy {
                extended_private_key.private_child_legacy(top_index_path)
            } else {
                extended_private_key.private_child(top_index_path)
            };
            Self::derive(sk_child, &path.index_path_by_removing_first_index(), use_legacy)
        }
    }

    // pub fn can_public_derive(index_path: IndexPath<u32>, use_legacy: bool) -> bool {
    //     for i in 0..index_path.length() {
    //         if index_path.index_at_position(0) >> 31 == 1 {
    //             return false;
    //         }
    //     }
    //     true
    // }

    pub fn public_derive<PATH>(extended_public_key: ExtendedPublicKey, index_path: &PATH, use_legacy: bool) -> ExtendedPublicKey
        where PATH: IIndexPath<Item = u32> {
        if index_path.is_empty() {
            extended_public_key
        } else {
            let top_index_path = index_path.index_at_position(0);
            assert_eq!(top_index_path >> 31, 0, "There should be no hardened derivation if you wish to derive extended public keys");
            let pk_child = if use_legacy {
                extended_public_key.public_child_legacy(top_index_path)
            } else {
                extended_public_key.public_child(top_index_path)
            };
            Self::public_derive(pk_child, &index_path.index_path_by_removing_first_index(), use_legacy)
        }
    }

    pub fn key_with_seed_data(seed: &[u8], use_legacy: bool) -> Self {
        let bls_private_key = PrivateKey::from_bip32_seed(seed);
        let bls_public_key = bls_private_key.g1_element().unwrap();
        let seckey = UInt256::from(&*bls_private_key.to_bytes());
        let pubkey = UInt384(*if use_legacy {
            bls_public_key.serialize_legacy()
        } else {
            bls_public_key.to_bytes()
        });
        Self { seckey, pubkey, use_legacy, ..Default::default() }
    }


    pub fn init_with_bls_extended_public_key(bls_extended_public_key: &ExtendedPublicKey, use_legacy: bool) ->  Self {
        let extended_public_key_data = if use_legacy {
            bls_extended_public_key.serialize_legacy()
        } else {
            bls_extended_public_key.serialize()
        }.to_vec();
        let bls_public_key = bls_extended_public_key.public_key();
        let public_key_data = if use_legacy {
            bls_public_key.serialize_legacy()
        } else {
            bls_public_key.to_bytes()
        };
        Self {
            extended_private_key_data: SecVec::new(),
            extended_public_key_data,
            chaincode: UInt256(*bls_extended_public_key.chain_code().serialize()),
            seckey: UInt256::MIN,
            pubkey: UInt384(*public_key_data),
            use_legacy
        }
    }

    pub fn init_with_bls_extended_private_key(bls_extended_private_key: &ExtendedPrivateKey, use_legacy: bool) -> Option<Self> {
        let extended_private_key_data = bls_extended_private_key.serialize();
        let extended_public_key_opt = if use_legacy {
            bls_extended_private_key.extended_public_key_legacy()
        } else {
            bls_extended_private_key.extended_public_key()
        };
        if extended_public_key_opt.is_err() {
            warn!("Can't restore extended_public_key");
            return None;
        }
        let extended_public_key = extended_public_key_opt.unwrap();
        let extended_public_key_data = if use_legacy {
            extended_public_key.serialize_legacy()
        } else {
            extended_public_key.serialize()
        };
        let chaincode = UInt256(*bls_extended_private_key.chain_code().serialize());
        let bls_private_key = bls_extended_private_key.private_key();
        let bls_public_key_opt = bls_private_key.g1_element();
        if bls_public_key_opt.is_err() {
            warn!("Can't restore bls_public_key");
            return None;
        }
        let bls_public_key = bls_public_key_opt.unwrap();
        let bls_public_key_bytes = if use_legacy {
            bls_public_key.serialize_legacy()
        } else {
            bls_public_key.to_bytes()
        };
        if let Some(seckey) = UInt256::from_bytes(bls_private_key.to_bytes().as_slice(), &mut 0) {
            Some(Self {
                extended_private_key_data: SecVec::with_vec(extended_private_key_data.to_vec()),
                extended_public_key_data: extended_public_key_data.to_vec(),
                chaincode,
                seckey,
                pubkey: UInt384(*bls_public_key_bytes),
                use_legacy,
            })
        } else {
            warn!("Can't restore secret_key");
            return None;
        }
    }

    pub fn extended_private_key_with_seed_data(seed: &[u8], use_legacy: bool) -> Option<Self> {
        ExtendedPrivateKey::from_seed(seed)
            .ok()
            .and_then(|pk| Self::init_with_bls_extended_private_key(&pk, use_legacy))
    }


    pub fn public_key_from_extended_public_key_data<PATH>(data: &[u8], index_path: &PATH, use_legacy: bool) -> Option<Vec<u8>>
        where PATH: IIndexPath<Item = u32> {
        if use_legacy {
            ExtendedPublicKey::from_bytes_legacy(data)
        } else {
            ExtendedPublicKey::from_bytes(data)
        }
            .ok()
            .and_then(|bls_extended_public_key|
                BLSKey::init_with_bls_extended_public_key(&bls_extended_public_key, use_legacy)
                    .public_derive_to_path(index_path)
                    .map(|pk| pk.public_key_data()))
    }

    pub fn public_key_fingerprint(&self) -> u32 {
        if self.use_legacy {
            G1Element::from_bytes_legacy(self.pubkey.as_bytes()).unwrap().fingerprint_legacy()
        } else {
            G1Element::from_bytes(self.pubkey.as_bytes()).unwrap().fingerprint()
        }
    }

    pub fn secret_key_string(&self) -> String {
        if self.seckey.is_zero() {
            String::new()
        } else {
            hex_with_data(self.seckey.as_bytes())
        }
    }

    pub fn serialized_private_key_for_script_map(&self, map: &ScriptMap) -> Option<String> {
        if self.seckey.is_zero() {
            None
        } else {
            // todo: impl securebox here
            let mut writer = Vec::<u8>::with_capacity(34);
            map.privkey.enc(&mut writer);
            self.seckey.enc(&mut writer);
            b'\x02'.enc(&mut writer);
            Some(base58::check_encode_slice(&writer))
        }
    }

    pub fn public_derive_to_path<PATH>(&mut self, index_path: &PATH) -> Option<Self>
        where PATH: IIndexPath<Item = u32> {
        if (self.extended_public_key_data().is_none() || self.extended_public_key_data().unwrap().is_empty()) && self.extended_private_key_data.is_empty() {
            None
        } else if let Some(bls_extended_public_key) = self.bls_extended_public_key() {
            Some(BLSKey::init_with_bls_extended_public_key(&BLSKey::public_derive(bls_extended_public_key, index_path, self.use_legacy), self.use_legacy))
        } else {
            None
        }
    }

    pub fn bls_extended_public_key(&mut self) -> Option<ExtendedPublicKey> {
        if let Some(bytes) = self.extended_public_key_data() {
            if self.use_legacy { ExtendedPublicKey::from_bytes_legacy(&bytes) } else { ExtendedPublicKey::from_bytes(&bytes) }.ok()
        } else if let Some(bytes) = self.extended_private_key_data() {
            ExtendedPrivateKey::from_bytes(&bytes).and_then(|pk| pk.extended_public_key()).ok()
        } else {
            None
        }
    }

    pub fn extended_private_key(&self) -> Option<Self> {
        if let Ok(pk) = self.bls_extended_private_key() {
            Self::init_with_bls_extended_private_key(&pk, self.use_legacy)
        } else {
            None
        }
    }

    pub fn bls_extended_private_key(&self) -> Result<ExtendedPrivateKey, BlsError> {
        ExtendedPrivateKey::from_bytes(&self.extended_private_key_data)
    }

    pub(crate) fn bls_private_key(&self) -> Result<PrivateKey, BlsError> {
        if !self.seckey.is_zero() {
            PrivateKey::from_bytes(self.seckey.as_bytes(), true)
        } else {
            ExtendedPrivateKey::from_bytes(self.extended_private_key_data.as_slice()).map(|ext_pk| ext_pk.private_key())
        }
    }

    pub(crate) fn bls_public_key(&self) -> Result<G1Element, BlsError> {
        if self.pubkey.is_zero() {
            self.bls_private_key().and_then(|bls_pk| bls_pk.g1_element())
        } else if self.use_legacy {
            G1Element::from_bytes_legacy(self.pubkey.as_bytes())
        } else {
            G1Element::from_bytes(self.pubkey.as_bytes())
        }
    }

    pub(crate) fn bls_public_key_serialized(&self) -> Option<[u8; 48]> {
        self.bls_public_key()
            .ok()
            .map(|pk| if self.use_legacy { *pk.serialize_legacy() } else { *pk.to_bytes() })
    }

    pub fn public_key_uint(&self) -> UInt384 {
        self.bls_public_key_serialized()
            .map_or(UInt384::MIN, |key| UInt384(key))
    }

    pub fn bls_version(&self) -> u16 {
        if self.use_legacy {
            1
        } else {
            2
        }
    }

    /// Signing
    pub fn sign_data(&self, data: &[u8]) -> UInt768 {
        if self.seckey.is_zero() && self.extended_private_key_data.is_empty() {
            UInt768::MAX
        } else if let Ok(bls_private_key) = self.bls_private_key() {
            let hash = sha256d::Hash::hash(data).into_inner();
            let signature = if self.use_legacy {
                LegacySchemeMPL::new().sign(&bls_private_key, &hash).serialize_legacy()
            } else {
                BasicSchemeMPL::new().sign(&bls_private_key, &hash).to_bytes()
            };
            UInt768(*signature)
        } else {
            UInt768::MAX
        }
    }

    pub fn sign_data_single_sha256(&self, data: &[u8]) -> UInt768 {
        if self.seckey.is_zero() && self.extended_private_key_data.is_empty() {
            UInt768::MAX
        } else if let Ok(bls_private_key) = self.bls_private_key() {
            let message = sha256::Hash::hash(data).into_inner();
            let signature = if self.use_legacy {
                LegacySchemeMPL::new().sign(&bls_private_key, &message).serialize_legacy()
            } else {
                BasicSchemeMPL::new().sign(&bls_private_key, &message).to_bytes()
            };
            UInt768(*signature)
        } else {
            UInt768::MAX
        }
    }

    pub fn sign_digest(&self, md: UInt256) -> UInt768 {
        if self.seckey.is_zero() && self.extended_private_key_data.is_empty() {
            UInt768::MIN
        } else if let Ok(bls_private_key) = self.bls_private_key() {
            let bls_signature = if self.use_legacy {
                LegacySchemeMPL::new().sign(&bls_private_key, md.as_bytes()).serialize_legacy()
            } else {
                BasicSchemeMPL::new().sign(&bls_private_key, md.as_bytes()).to_bytes()
            };
            UInt768(*bls_signature)
        } else {
            UInt768::MIN
        }
    }

    pub fn sign_message_digest(&self, digest: UInt256, completion: fn(bool, UInt768)) {
        let signature = self.sign_digest(digest);
        completion(!signature.is_zero(), signature)
    }


    /// Verification

    pub fn verify_uint768(&self, digest: UInt256, signature: UInt768) -> bool {
        if let Ok(bls_public_key) = self.bls_public_key() {
            if self.use_legacy {
                LegacySchemeMPL::new().verify(&bls_public_key, digest.as_bytes(), &G2Element::from_bytes_legacy(signature.as_bytes()).unwrap())
            } else {
                BasicSchemeMPL::new().verify(&bls_public_key, digest.as_bytes(), &G2Element::from_bytes(signature.as_bytes()).unwrap())
            }
        } else {
            false
        }
    }

    pub fn verify_with_public_key(digest: UInt256, signature: UInt768, public_key: UInt384, use_legacy: bool) -> bool {
        if let Ok(bls_public_key) = BLSKey::key_with_public_key(public_key, use_legacy).bls_public_key() {
            if use_legacy {
                LegacySchemeMPL::new().verify(&bls_public_key, digest.as_bytes(), &G2Element::from_bytes_legacy(signature.as_bytes()).unwrap())
            } else {
                BasicSchemeMPL::new().verify(&bls_public_key, digest.as_bytes(), &G2Element::from_bytes(signature.as_bytes()).unwrap())
            }
        } else {
            false
        }
    }

    pub fn verify_secure_aggregated(commitment_hash: UInt256, signature: UInt768, operator_keys: Vec<OperatorPublicKey>, use_legacy: bool) -> bool {
        let message = commitment_hash.as_bytes();
        let public_keys = operator_keys.iter().filter_map(|key| {
            if key.is_legacy() {
                G1Element::from_bytes_legacy(&key.data.0)
            } else {
                G1Element::from_bytes(&key.data.0)
            }.ok()
        }).collect::<Vec<_>>();
        if use_legacy {
            G2Element::from_bytes_legacy(signature.as_bytes())
                .map_or(false, |sig| LegacySchemeMPL::new()
                    .verify_secure(public_keys.iter().collect::<Vec<&G1Element>>(), message, &sig))
        } else {
            G2Element::from_bytes(signature.as_bytes())
                .map_or(false, |sig| BasicSchemeMPL::new()
                    .verify_secure(public_keys.iter().collect::<Vec<&G1Element>>(), message, &sig))
        }
    }

    pub fn verify_quorum_signature(message: &[u8], threshold_signature: &[u8], public_key: &[u8], use_legacy: bool) -> bool {
        if use_legacy {
            G1Element::from_bytes_legacy(public_key)
                .map_or(false, |pk| G2Element::from_bytes_legacy(threshold_signature)
                    .map_or(false, |sig| LegacySchemeMPL::new().verify(&pk, message, &sig)))
        } else {
            G1Element::from_bytes(public_key)
                .map_or(false, |pk| G2Element::from_bytes(threshold_signature)
                    .map_or(false, |sig| BasicSchemeMPL::new().verify(&pk, message, &sig)))
        }
    }


    pub fn verify_aggregated_signature(signature: UInt768, public_keys: Vec<BLSKey>, messages: Vec<Vec<u8>>, use_legacy: bool) -> bool {
        let bls_public_keys = public_keys.iter().filter_map(|key| key.bls_public_key().ok()).collect::<Vec<_>>();
        let keys = bls_public_keys.iter().collect::<Vec<&G1Element>>();
        let messages = messages.iter().map(|m| m.as_slice()).collect::<Vec<_>>();
        let bytes = signature.as_bytes();
        if use_legacy {
            LegacySchemeMPL::new().aggregate_verify(keys, messages, &G2Element::from_bytes_legacy(bytes).unwrap())
        } else {
            BasicSchemeMPL::new().aggregate_verify(keys, messages, &G2Element::from_bytes(bytes).unwrap())
        }
    }

    pub fn public_key_and_signature_from_seed<S: Scheme>(schema: S, seed: &[u8], message: &[u8]) -> (G1Element, G2Element) {
        let private_key = PrivateKey::from_bip32_seed(seed);
        let signature = schema.sign(&private_key, message);
        let public_key = private_key.g1_element().unwrap();
        (public_key, signature)
    }
}

/// For FFI
impl BLSKey {
    pub fn public_key_from_extended_public_key_data_at_index_path<PATH>(key: &Self, index_path: &PATH) -> Option<Self> where Self: Sized, PATH: IIndexPath<Item=u32> {
        key.extended_public_key_data()
            .and_then(|ext_pk_data| Self::public_key_from_extended_public_key_data(&ext_pk_data, index_path, key.use_legacy))
            .map(|pub_key_data| Self::key_with_public_key(UInt384::from(pub_key_data), key.use_legacy))
    }

    pub fn key_with_extended_public_key_data(bytes: &[u8], use_legacy: bool) -> Option<Self> {
        if use_legacy {
            ExtendedPublicKey::from_bytes_legacy(bytes)
        } else {
            ExtendedPublicKey::from_bytes(bytes)
        }.ok().map(|bls_extended_public_key| Self::init_with_bls_extended_public_key(&bls_extended_public_key, use_legacy))

    }

    pub fn key_with_extended_private_key_data(bytes: &[u8], use_legacy: bool) -> Option<Self> {
        ExtendedPrivateKey::from_bytes(bytes)
            .ok()
            .and_then(|pk| Self::init_with_bls_extended_private_key(&pk, use_legacy))

    }

    pub fn has_private_key(&self) -> bool {
        !self.seckey.is_zero()
    }

    pub fn hash160(&self) -> UInt160 {
        UInt160::hash160(&self.public_key_data())
    }

}


impl DHKey for BLSKey {
    fn init_with_dh_key_exchange_with_public_key(public_key: &mut Self, private_key: &Self) -> Option<Self> where Self: Sized {
        match (public_key.bls_public_key(), private_key.bls_private_key(), private_key.use_legacy) {
            (Ok(bls_public_key), Ok(bls_private_key), use_legacy) if public_key.use_legacy == use_legacy =>
                (bls_private_key * bls_public_key)
                    .ok()
                    .map(|key|
                        BLSKey::key_with_public_key(UInt384(if use_legacy { *key.serialize_legacy() } else { *key.to_bytes() }), use_legacy)),
            _ => None
        }
    }
}

impl CryptoData<BLSKey> for Vec<u8> {

    fn encrypt_with_secret_key_using_iv(&mut self, secret_key: &BLSKey, public_key: &BLSKey, initialization_vector: Vec<u8>) -> Option<Vec<u8>> {
        secret_key.product(public_key)
            .and_then(|sym_key_data| sym_key_data[..32].try_into().ok())
            .map(|key_data: [u8; 32]| {
                let mut destination = initialization_vector.clone();
                let iv: [u8; 16] = initialization_vector[..16].try_into().unwrap();
                let encrypted_data = <Self as CryptoData<BLSKey>>::encrypt(self, key_data, iv).unwrap();
                destination.extend(encrypted_data.clone());
                destination
            })
    }

    fn decrypt_with_secret_key_using_iv_size(&mut self, secret_key: &BLSKey, public_key: &BLSKey, iv_size: usize) -> Option<Vec<u8>> {
        if self.len() < iv_size {
            return None;
        }
        secret_key.product(public_key)
            .and_then(|sym_key_data| sym_key_data[..32].try_into().ok())
            .and_then(|key_data: [u8; 32]|
                <Self as CryptoData<BLSKey>>::decrypt(self[iv_size..self.len()].to_vec(), key_data, &self[..iv_size]))
    }

    fn encrypt_with_dh_key_using_iv(&self, key: &BLSKey, initialization_vector: Vec<u8>) -> Option<Vec<u8>> {
        let mut destination = initialization_vector.clone();
        key.bls_public_key_serialized()
            .and_then(|sym_key_data| sym_key_data[..32].try_into().ok())
            .and_then(|key_data: [u8; 32]| initialization_vector[..16].try_into().ok()
                .and_then(|iv_data: [u8; 16]| <Self as CryptoData<BLSKey>>::encrypt(self, key_data, iv_data))
                .map(|encrypted_data| {
                    destination.extend(encrypted_data);
                    destination
                }))
    }

    fn decrypt_with_dh_key_using_iv_size(&self, key: &BLSKey, iv_size: usize) -> Option<Vec<u8>> {
        if self.len() < iv_size {
            return None;
        }
        key.bls_public_key_serialized()
            .and_then(|sym_key_data| sym_key_data[..32].try_into().ok())
            .and_then(|key_data: [u8; 32]| self[..iv_size].try_into().ok()
                .and_then(|iv_data: [u8; 16]|
                    <Self as CryptoData<BLSKey>>::decrypt(self[iv_size..self.len()].to_vec(), key_data, iv_data)))
    }
}

