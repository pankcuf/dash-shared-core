use hashes::{Hash, sha256};
use hashes::hex::ToHex;
use crate::consensus::Encodable;
use crate::crypto::{UInt256, UInt512};
use crate::keys::ECDSAKey;
use crate::util::data_ops::short_hex_string_from;

#[derive(Clone, Debug)]
pub struct Seed {
    pub unique_id: String,
    pub data: Vec<u8>
}

impl Seed {
    pub fn new(data: Vec<u8>, unique_id: String) -> Self {
        Self { data, unique_id }
    }

    pub fn with_data(data: Vec<u8>) -> Self {
        Self { data, unique_id: String::new() }
    }

    pub fn from<L: bip0039::Language>(seed: Vec<u8>, genesis: UInt256) -> Seed {
        // let derived_key_data = mnemonic.to_seed("");
        let seed_key = UInt512::bip32_seed_key(&seed);
        println!("Wallet.setSeedPhrase: {}, {}", seed.to_hex(), seed_key);
        let mut unique_id_data = Vec::<u8>::new();
        genesis.enc(&mut unique_id_data);
        if let Some(public_key_data) = ECDSAKey::public_key_data_from_seed(&seed_key.0[..32], true) {
            public_key_data.enc(&mut unique_id_data);
        }
        let unique_id = short_hex_string_from(&sha256::Hash::hash(unique_id_data.as_slice()).into_inner());
        Seed::new(seed, unique_id)
    }

    pub fn from_phrase<L: bip0039::Language>(seed_phrase: &str, genesis: UInt256) -> Option<Self> {
        bip0039::Mnemonic::<L>::from_phrase(seed_phrase)
            .map(|mnemonic| mnemonic.to_seed(""))
            .map(|seed| Seed::from::<L>(seed.to_vec(), genesis))
            .ok()
    }


    pub fn unique_id_as_str(&self) -> &str {
        self.unique_id.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

}

