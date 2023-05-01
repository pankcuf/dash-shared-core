use std::collections::{BTreeSet, HashMap};
use byte::{BytesExt, TryRead, TryWrite};
use byte::ctx::Endian;
use security_framework::os::macos::keychain::SecKeychain;
use crate::chain::common::ChainType;
use crate::chain::wallet::ext::constants::{accounts_known_key_for_wallet_unique_id, creation_time_unique_id_for_unique_id, did_verify_creation_time_unique_id_for_unique_id, mnemonic_unique_id_for_unique_id};
use crate::consensus::{Encodable, encode::VarInt};
use crate::chain::derivation::{wallet_based_extended_private_key_location_string_for_unique_id, wallet_based_extended_public_key_location_string_for_unique_id_and_key_type};
use crate::keys::KeyKind;
use crate::util::sec_vec::SecVec;

pub const SEC_ATTR_SERVICE: &str = "org.dashfoundation.dash-spv";

#[derive(Clone, Debug, PartialEq)]
pub struct VarString {
    pub length: VarInt,
    pub string: String,
}

impl<'a> TryRead<'a, Endian> for VarString {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let length = bytes.read_with::<VarInt>(offset, endian)?;
        let string = bytes.read_with::<&str>(offset, byte::ctx::Str::Len(length.0 as usize)).unwrap().to_string();
        Ok((Self { length, string }, *offset))
    }
}

impl TryWrite<Endian> for VarString {
    fn try_write(self, bytes: &mut [u8], endian: Endian) -> byte::Result<usize> {
        // let offset = &mut 0;
        // bytes.write_with(offset, self.length, ())?;
        // bytes.write_with(offset, self.string, endian)?;
        self.string.enc(bytes);
        Ok(self.length.0 as usize)
    }
}


impl Encodable for VarString {
    #[inline]
    fn consensus_encode<S: std::io::Write>(&self, mut s: S) -> Result<usize, std::io::Error> {
        self.string.enc(&mut s);
        Ok(std::mem::size_of::<VarString>())
    }
}


#[derive(Debug, Default)]
pub struct Keychain {}

impl Keychain {

    // #[cfg(not(test))]
    // fn default_keychain() -> Result<SecKeychain, security_framework::base::Error> {
    //     SecKeychain::default()
    // }

    // #[cfg(test)]
    fn default_keychain() -> Result<SecKeychain, security_framework::base::Error> {
        use security_framework::os::macos::keychain::CreateOptions;
        use std::env;
        use std::path::Path;
        let exe_path = env::current_exe().unwrap();
        let exe_name = Path::new(&exe_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let keychain_name = if exe_name.len() >= 16 {
            format!("dash-spv-{}.keychain", &exe_name[exe_name.len() - 16..])
        } else {
            format!("dash-spv-main.keychain")
        };
        let keychain_path = exe_path.parent().unwrap().join(keychain_name);
        // let keychain_path = Path::new(env::current_dir().unwrap().as_path())
        //     .join(format!("target/debug/deps/dash-spv-{}.keychain", &exe_name[exe_name.len() - 16..]));
        match CreateOptions::new()
            .password("")
            .create(keychain_path.clone()) {
            Ok(keychain) => SecKeychain::disable_user_interaction().map(|_| keychain),
            Err(err) if err.code() == -25296 => match SecKeychain::open(keychain_path) {
                Ok(keychain) => SecKeychain::disable_user_interaction().map(|_| keychain),
                Err(err) =>  panic!("Keychain::open: {:?} {:?}", err.code(), err.message())
            },
            Err(err) => panic!("Keychain::create: {:?} {:?}", err.code(), err.message())
        }
    }

    pub fn set_data(key: String, data: Option<impl AsRef<[u8]>>, authenticated: bool) -> Result<(), security_framework::base::Error> {
        let account = key.as_str();
        // Here we also check keychain item for presence (-25299)
        // This scheme allows not to use single-threaded testing (cargo test -- --test-threads=1)
        Self::default_keychain()
            .and_then(|keychain| {
                match (data, keychain.find_generic_password(SEC_ATTR_SERVICE, account)) {
                    (Some(data), Ok((_, mut item))) =>
                        item.set_password(data.as_ref()),
                    (Some(data), Err(err)) =>
                        match keychain.add_generic_password(SEC_ATTR_SERVICE, account, data.as_ref()) {
                            Ok(..) => Ok(()),
                            Err(err) if err.code() == -25299 => Ok(()),
                            Err(err) => Err(err),
                    },
                    (None, Ok((_, item))) => {
                        item.delete();
                        Ok(())
                    },
                    (None, Err(err)) => {
                        println!("Keychain error {:?}: {:?}", err.code(), err.message());
                        Err(err)
                    }
                }
            })
    }

    pub fn has_data(key: String) -> Result<(), security_framework::base::Error> {
        let account = key.as_str();
        Self::default_keychain()
            .and_then(|keychain|
                match keychain.find_generic_password(SEC_ATTR_SERVICE, account) {
                    Ok((_, _)) => Ok(()),
                    Err(err) => Err(err)
                })
    }

    pub fn get_data(key: String) -> Result<Vec<u8>, security_framework::base::Error> {
        let account = key.as_str();
        Self::default_keychain()
            .and_then(|keychain|
                keychain.find_generic_password(SEC_ATTR_SERVICE, account)
                    .map(|(password, item)| password.to_vec()))
    }

    pub fn set_int(i: i64, key: String, authenticated: bool) -> Result<(), security_framework::base::Error> {
        Self::set_data(key, Some(i.to_le_bytes().to_vec()), authenticated)
    }

    pub fn get_int(key: String) -> Result<i64, security_framework::base::Error> {
        Self::get_data(key)
            .map(|data| data.try_into()
                .map(|bytes: [u8; 8]| i64::from_le_bytes(bytes)).unwrap_or(0))
    }

    pub fn set_string(s: String, key: String, authenticated: bool) -> Result<(), security_framework::base::Error> {
        Self::set_data(key, Some(s.as_bytes().to_vec()), authenticated)
    }

    pub fn get_string(key: String) -> Result<String, security_framework::base::Error> {
        Self::get_data(key).map(|data| String::from_utf8(data).unwrap_or("".to_string()))
    }

    pub fn set_string_array(arr: Vec<String>, key: String, authenticated: bool) -> Result<(), security_framework::base::Error> {
        let mut writer = Vec::<u8>::new();
        arr.iter().for_each(|string| {
            string.enc(&mut writer);
        });
        Self::set_data(key, Some(writer), authenticated)
    }

    pub fn get_string_array(key: String) -> Result<Vec<String>, security_framework::base::Error> {
        Self::get_data(key).map(|data| {
            let mut offset = &mut 0;
            let mut result = Vec::<String>::new();
            let mut iter: byte::Iter<VarString, _> = data.read_iter(&mut offset, byte::LE);
            while let Some(var_string) = iter.next() {
                result.push(var_string.string.clone())
            }
            result
        })
    }

    pub fn set_dict<K, V>(dict: HashMap<K, V>, key: String, authenticated: bool) -> Result<(), security_framework::base::Error> {
        todo!("Implement bindings for keychain")
    }

    pub fn get_dict<K, V>(key: String) -> Result<HashMap<K, V>, security_framework::base::Error> {
        todo!("Implement bindings for keychain")
    }

    pub fn set_array<T>(arr: Vec<T>, key: String, authenticated: bool) -> Result<(), security_framework::base::Error> where T: Encodable {
        let mut writer = Vec::<u8>::new();
        VarInt(arr.len() as u64).enc(&mut writer);
        arr.iter().for_each(|item| {
            item.enc(&mut writer);
        });

        Self::set_data(key, Some(writer), authenticated)
    }

    pub fn get_array<'a, T>(key: String) -> Result<Vec<T>, security_framework::base::Error> where T: TryRead<'a, Endian> {
        todo!()
        // Self::get_data(key).and_then(|data| {
        //     let mut offset = &mut 0;
        //     let bytes = data.into_boxed_slice();
        //     let len = bytes.read_with::<VarInt>(offset, byte::LE).unwrap().0 as usize;
        //     // let mut iter = ;
        //     let mut items = Vec::<T>::new();
        //     while let Some(item) = bytes.read_iter::<T>(&mut offset, byte::LE).next() {
        //         items.push(item);
        //     }
        //     for _i in 0..len {
        //         items.push(bytes.read_with::<T>(offset, byte::LE).unwrap());
        //     }
        //     Ok(items)
        // })
    }
// impl TryInto<Key> for (&str, ChainType) {
//     type Error = Error;
//
//     fn try_into(self) -> Result<Key, Self::Error> {
//         base58::from(self.0)
//             .map_err(base58::Error::into)
//             .and_then(|message| message.read_with::<Key>(&mut 0, self.1)
//                 .map_err(byte::Error::into))
//     }
// }


    pub fn set_ordered_set<V>(arr: BTreeSet<V>, key: String, authenticated: bool) -> Result<(), security_framework::base::Error> {
        todo!("Implement bindings for keychain")
    }

    pub fn get_ordered_set<V>(key: String, classes: Vec<String>) -> Result<BTreeSet<V>, security_framework::base::Error> {
        todo!("Implement bindings for keychain")
    }

    pub fn set_object<T>(object: T, key: String, authenticated: bool) -> Result<(), security_framework::base::Error> {
        todo!("Implement bindings for keychain")
    }

    pub fn get_object<T>(key: String) -> Result<T, security_framework::base::Error> {
        todo!("Implement bindings for keychain")
    }

    // pub fn set_json(dict: serde_json::Value, key: String, authenticated: bool) -> Result<bool, KeychainError> {
    //     todo!("Implement bindings for keychain")
    // }
    //
    // pub fn get_json(key: String) -> Result<serde_json::Value, KeychainError> {
    //     todo!("Implement bindings for keychain")
    // }

}

impl Keychain {
    pub fn save_seed_phrase(phrase: &str, created_at: u64, unique_id: &str) -> Result<(), security_framework::base::Error> {
        assert_ne!(created_at, 0, "error setting wallet");
        // in version 2.0.0 wallet creation times were migrated from reference date,
        // since this is now fixed just add this line so verification only happens once
        Self::set_string(phrase.to_string(), mnemonic_unique_id_for_unique_id(unique_id), true)
            .and(Self::set_data(creation_time_unique_id_for_unique_id(unique_id), Some(created_at.to_le_bytes().to_vec()), false))
            .and(Self::set_int(1, did_verify_creation_time_unique_id_for_unique_id(unique_id), false))

        // match Self::set_string(phrase.to_string(), mnemonic_unique_id_for_unique_id(unique_id), true) {
        //     Ok(()) => {
        //         match Self::set_data(creation_time_unique_id_for_unique_id(unique_id), Some(created_at.to_le_bytes().to_vec()), false) {
        //             Ok(()) => {
        //                 match Self::set_int(1, did_verify_creation_time_unique_id_for_unique_id(unique_id), false) {
        //                     Ok(()) => Ok(()),
        //                     Err(err) => {
        //                         println!("Keychain error.3 {:?}: {:?}", err.code(), err.message());
        //                         Err(err)
        //                     }
        //
        //                 }
        //             },
        //             Err(err) => {
        //                 println!("Keychain error.2 {:?}: {:?}", err.code(), err.message());
        //                 Err(err)
        //             }
        //         }
        //     },
        //     Err(err) if err.code() == -25299 => match Self::set_data(creation_time_unique_id_for_unique_id(unique_id), Some(created_at.to_le_bytes().to_vec()), false) {
        //         Ok(()) => {
        //             match Self::set_int(1, did_verify_creation_time_unique_id_for_unique_id(unique_id), false) {
        //                 Ok(()) => Ok(()),
        //                 Err(err) => {
        //                     println!("Keychain error.23 {:?}: {:?}", err.code(), err.message());
        //                     Err(err)
        //                 }
        //
        //             }
        //         },
        //         Err(err) => {
        //             println!("Keychain error.22 {:?}: {:?}", err.code(), err.message());
        //             Err(err)
        //         }
        //     },
        //     Err(err) => {
        //         println!("Keychain error.1 {:?}: {:?}", err.code(), err.message());
        //         Err(err)
        //     }
        // }

    }

    pub fn mnemonic(wallet_unique_id: &str) -> Result<String, security_framework::base::Error> {
        Self::get_string(mnemonic_unique_id_for_unique_id(wallet_unique_id))
    }

    pub fn save_last_account_number(number: u32, wallet_unique_id: &str) -> Result<(), security_framework::base::Error> {
        Self::set_int(number as i64, accounts_known_key_for_wallet_unique_id(wallet_unique_id), false)
    }

    pub fn last_account_number(wallet_unique_id: &str) -> Result<u32, security_framework::base::Error> {
        Self::get_int(accounts_known_key_for_wallet_unique_id(wallet_unique_id))
            .map(|i| i as u32)
    }

    pub fn save_extended_public_key(wallet_unique_id: &str, r#type: KeyKind, index_path: String, data: Option<Vec<u8>>) -> Result<(), security_framework::base::Error> {
        Self::set_data(
            wallet_based_extended_public_key_location_string_for_unique_id_and_key_type(
                wallet_unique_id,
                r#type,
                index_path), data, false)
    }

    pub fn save_extended_private_key(wallet_unique_id: &str, data: Option<SecVec>) -> Result<(), security_framework::base::Error> {
        Self::set_data(wallet_based_extended_private_key_location_string_for_unique_id(wallet_unique_id), data, true)
    }


    pub fn get_wallet_ids(chain_type: ChainType) -> Result<Vec<String>, security_framework::base::Error> {
        Self::get_string_array(chain_type.chain_wallets_key())
    }

    pub fn get_standalone_derivation_path_ids(chain_type: ChainType) -> Result<Vec<String>, security_framework::base::Error> {
        Self::get_string_array(chain_type.chain_wallets_key())
    }
}
