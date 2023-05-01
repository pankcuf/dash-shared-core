use crate::chain::Wallet;

pub trait Seeds {
    // fn generate_random_seed_phrase_for_language(language: Language) -> Option<bip39::Mnemonic>;
    // fn generate_random_seed_phrase() -> Option<bip39::Mnemonic>;
    // fn seed_phrase_after_authentication(&self) -> Result<String, AuthenticationError>;
    // fn has_seed_phrase(&self) -> bool;
    // fn set_transient_derived_key_data(derived_key_data: &Vec<u8>, accounts: &Vec<Account>, chain: &Chain) -> String;
    // fn get_seed_for(seed_phrase: &'a str, created_at: u64, store_on_keychain: bool, genesis_hash: UInt256, environment: &'a Environment) -> Option<(String, [u8; 64])>;
    // fn get_seed_for<L: bip0039::Language>(seed_phrase: &'a str, created_at: u64, store_on_keychain: bool, genesis_hash: UInt256) -> Option<(String, bip0039::Mnemonic<L>)>;
    // fn set_seed_phrase(seed_phrase: &'a str, created_at: u64, accounts: Vec<&'a mut Account<'a>>, store_on_keychain: bool, chain: &'a Chain<'a>) -> Option<String>;
    // fn seed_with_prompt(&self, authprompt: Option<String>, amount: u64) -> Result<(Option<Vec<u8>>, bool), util::Error>;
    // fn seed_phrase_if_authenticated(&self) -> Option<String>;
    // fn seed_phrase_after_authentication_with_prompt(&self, authprompt: Option<String>) -> Result<String, AuthenticationError>;

    // fn unique_id_for_seed<L: bip0039::Language>(seed: [u8; 64], genesis_hash: UInt256) -> String {
    //     // let derived_key_data = mnemonic.to_seed("");
    //     let seed_key = UInt512::bip32_seed_key(&seed);
    //     let mut unique_id_data = Vec::<u8>::new();
    //     genesis_hash.enc(&mut unique_id_data);
    //     if let Some(public_key_data) = ECDSAKey::public_key_data_from_seed(&seed_key.0[..32], true) {
    //         public_key_data.enc(&mut unique_id_data);
    //     }
    //     short_hex_string_from(&sha256::Hash::hash(unique_id_data.as_slice()).into_inner())
    // }
}

impl Seeds for Wallet {
    // fn get_seed_for<L: bip0039::Language>(seed_phrase: &'a str, created_at: u64, store_on_keychain: bool, genesis_hash: UInt256) -> Option<(String, bip0039::Mnemonic<L>)> {
    //     bip0039::Mnemonic::<L>::from_phrase(seed_phrase)
    //     // bip39::Mnemonic::parse_normalized(seed_phrase)
    //         .ok()
    //         .and_then(|mnemonic| {
    //             let seed = mnemonic.to_seed("");
    //             let unique_id = Self::unique_id_for_seed::<L>(seed, genesis_hash);
    //             // if not store on keychain then we won't save the extended public keys below.
    //             // let mut store_on_unique_id: Option<&String> = None;
    //             if store_on_keychain {
    //                 if Keychain::set_string(seed_phrase.to_string(), mnemonic_unique_id_for_unique_id(unique_id.as_str()), true).is_err() ||
    //                     (created_at != 0 && Keychain::set_data(creation_time_unique_id_for_unique_id(unique_id.as_str()), Some(created_at.to_le_bytes().to_vec()), false).is_err()) {
    //                     assert!(false, "error setting wallet seed");
    //                     return None;
    //                 }
    //                 // in version 2.0.0 wallet creation times were migrated from reference date,
    //                 // since this is now fixed just add this line so verification only happens once
    //                 Keychain::set_int(1, did_verify_creation_time_unique_id_for_unique_id(unique_id.as_str()), false)
    //                     .expect("Can't store VerifyCreationTimeUniqueID");
    //                 // return Some(unique_id)
    //             }
    //             Some((unique_id, mnemonic))
    //         })
    //
    // }
    // fn set_seed_phrase(seed_phrase: &'a str, created_at: u64, accounts: Vec<&'a mut Account<'a>>, store_on_keychain: bool, chain: &'a Chain<'a>) -> Option<String> {
    //     bip39::Mnemonic::parse_normalized(seed_phrase)
    //         .ok()
    //         .and_then(|mnemonic| {
    //             let derived_key_data = mnemonic.to_seed_normalized("").to_vec();
    //             let seed_key = UInt512::bip32_seed_key(&derived_key_data);
    //
    //             let mut unique_id_data = Vec::<u8>::new();
    //             chain.r#type().genesis_hash().enc(&mut unique_id_data);
    //             if let Some(mut public_key) = ECDSAKey::key_with_secret(&seed_key.0[..32].to_vec(), true) {
    //                 public_key.public_key_data().enc(&mut unique_id_data);
    //             }
    //             let unique_id = short_hex_string_from(&sha256::Hash::hash(unique_id_data.as_slice()).into_inner());
    //             // if not store on keychain then we won't save the extended public keys below.
    //             let mut store_on_unique_id: Option<&String> = None;
    //             if store_on_keychain {
    //                 if Keychain::set_string(seed_phrase.to_string(), mnemonic_unique_id_for_unique_id(unique_id.as_str()), true).is_err() ||
    //                     (created_at != 0 && Keychain::set_data(creation_time_unique_id_for_unique_id(unique_id.as_str()), Some(created_at.to_le_bytes().to_vec()), false).is_err()) {
    //                     assert!(false, "error setting wallet seed");
    //                     return None;
    //                 }
    //                 // in version 2.0.0 wallet creation times were migrated from reference date,
    //                 // since this is now fixed just add this line so verification only happens once
    //                 Keychain::set_int(1, did_verify_creation_time_unique_id_for_unique_id(unique_id.as_str()), false)
    //                     .expect("Can't store VerifyCreationTimeUniqueID");
    //                 store_on_unique_id = Some(&unique_id);
    //             }
    //             accounts.iter()
    //                 .for_each(|account|
    //                     account.generate_extended_public_keys_for_seed(&derived_key_data, store_on_unique_id));
    //             Some(unique_id)
    //         })
    // }
}
