pub mod bls_key;
pub mod key;
pub mod ecdsa_key;
pub mod ed25519_key;
pub mod dip14;
pub mod crypto_data;

pub use self::key::Key;
pub use self::key::KeyKind;
pub use self::bls_key::BLSKey;
pub use self::crypto_data::{CryptoData, DHKey};
pub use self::ecdsa_key::ECDSAKey;
pub use self::ed25519_key::ED25519Key;

use std::fmt::Debug;
use crate::chain::ScriptMap;
use crate::chain::derivation::index_path::IIndexPath;
use crate::chain::tx::protocol::SIGHASH_ALL;
use crate::consensus::Encodable;
use crate::crypto::UInt256;
use crate::keys::dip14::{IChildKeyDerivation, SignKey};
use crate::util::address::address;
use crate::util::data_append::DataAppend;
use crate::util::script::ScriptElement;
use crate::util::sec_vec::SecVec;

pub trait IKey: Send + Sync + Debug {
    fn r#type(&self) -> KeyKind {
        panic!("Should be overriden in implementation")
    }
    fn address_with_public_key_data(&self, script_map: &ScriptMap) -> String {
        address::with_public_key_data(&self.public_key_data(), script_map)
    }
    fn sign(&self, data: &[u8]) -> Vec<u8> {
        panic!("Should be overriden in implementation")
    }
    fn verify(&mut self, message_digest: &[u8], signature: &[u8]) -> bool {
        panic!("Should be overriden in implementation")
    }
    fn secret_key(&self) -> UInt256 {
        panic!("Should be overriden in implementation")
    }

    fn chaincode(&self) -> UInt256 {
        panic!("Should be overriden in implementation")
    }

    fn fingerprint(&self) -> u32 {
        panic!("Should be overriden in implementation")
    }

    fn private_key_data(&self) -> Option<Vec<u8>> {
        panic!("Should be overriden in implementation")
    }
    fn public_key_data(&self) -> Vec<u8> {
        panic!("Should be overriden in implementation")
    }
    fn extended_private_key_data(&self) -> Option<SecVec> {
        panic!("Should be overriden in implementation")
    }
    fn extended_public_key_data(&self) -> Option<Vec<u8>> {
        panic!("Should be overriden in implementation")
    }
    fn private_derive_to_path2<SK, PK, PATH, INDEX>(&self, path: &PATH) -> Option<Self>
        where Self: Sized + IChildKeyDerivation<INDEX, SK, PK>, PATH: IIndexPath<Item = INDEX>, SK: SignKey {
        panic!("Should be overriden in implementation")
    }

    fn private_derive_to_path<PATH>(&self, path: &PATH) -> Option<Self>
        where Self: Sized, PATH: IIndexPath<Item = u32> {
        panic!("Should be overriden in implementation")
    }
    fn private_derive_to_256bit_derivation_path<PATH>(&self, path: &PATH) -> Option<Self>
        where Self: Sized, PATH: IIndexPath<Item = UInt256> {
        self.private_derive_to_path(&path.base_index_path())
    }
    fn public_derive_to_256bit_derivation_path<PATH>(&mut self, path: &PATH) -> Option<Self>
        where Self: Sized, PATH: IIndexPath<Item = UInt256> {
        self.public_derive_to_256bit_derivation_path_with_offset(path, 0)
    }
    fn public_derive_to_256bit_derivation_path_with_offset<PATH>(&mut self, path: &PATH, offset: usize) -> Option<Self>
        where Self: Sized, PATH: IIndexPath<Item = UInt256> {
        panic!("Should be overriden in implementation")
    }
    // fn public_key_from_extended_public_key_data_at_index_path<PATH>(key: &Self, index_path: &PATH) -> Option<Self>
    //     where Self: Sized, PATH: IIndexPath<Item = u32> {
    //     key.extended_public_key_data().and_then()
    // }

    fn serialized_private_key_for_script(&self, script: &ScriptMap) -> String {
        panic!("Should be overriden in implementation")
    }
    fn hmac_256_data(&self, data: &[u8]) -> UInt256 {
        panic!("Should be overriden in implementation")
    }
    fn forget_private_key(&mut self) {
        panic!("Should be overriden in implementation")
    }

    fn create_signature(&self, tx_input_script: &Vec<u8>, tx_data: &Vec<u8>) -> Vec<u8> {
        let mut sig = Vec::<u8>::new();
        let hash = UInt256::sha256d(tx_data);
        let mut s = self.sign(&hash.0.to_vec());
        let elem = tx_input_script.script_elements();
        (SIGHASH_ALL as u8).enc(&mut s);
        s.append_script_push_data(&mut sig);
        // sig.append_script_push_data(s);
        if elem.len() >= 2 {
            if let ScriptElement::Data([0x88 /*OP_EQUALVERIFY*/, ..], ..) = elem[elem.len() - 2] {
                // pay-to-pubkey-hash scriptSig
                self.public_key_data().append_script_push_data(&mut sig);
                // sig.append_script_push_data(self.public_key_data());
            }
        }
        sig
    }


}
