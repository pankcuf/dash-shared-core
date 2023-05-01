use base64::{alphabet, Engine};
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use hashes::hex::{FromHex, ToHex};
use crate::chain::common::ChainType;
use crate::crypto::UInt256;
use crate::keys::{BLSKey, ECDSAKey, IKey};
use crate::keys::crypto_data::{CryptoData, DHKey};
use crate::util::address::address;

#[test]
fn test_ecdsa_encryption_and_decryption() {
    let alice_secret = UInt256::from_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
    let alice_key_pair = ECDSAKey::key_with_secret(&alice_secret, true).unwrap();
    let bob_secret = UInt256::from_hex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140").unwrap();
    let mut bob_key_pair = ECDSAKey::key_with_secret(&bob_secret, true).unwrap();
    let key = ECDSAKey::init_with_dh_key_exchange_with_public_key(&mut bob_key_pair, &alice_key_pair).unwrap();
    let key_public_key_data_hex = key.public_key_data().to_hex();
    assert_eq!(key_public_key_data_hex, "fbd27dbb9e7f471bf3de3704a35e884e37d35c676dc2cc8c3cc574c3962376d2", "they should be the same data");
    let secret = "my little secret is a pony that never sleeps";
    let mut data = secret.as_bytes().to_vec();
    // Alice is sending to Bob
    let iv = Vec::from_hex("eac5bcd6eb85074759e0261497428c9b").unwrap();
    let encrypted = <Vec<u8> as CryptoData<ECDSAKey>>::encrypt_with_secret_key_using_iv(&mut data, &alice_key_pair, &bob_key_pair, iv);
    assert!(encrypted.is_some(), "Encrypted data is None");
    let mut encrypted = encrypted.unwrap();
    assert_eq!(encrypted.to_hex(), "eac5bcd6eb85074759e0261497428c9b3725d3b9ec4d739a842116277c6ace81549089be0d11a54ee09a99dcf7ac695a8ea56d41bf0b62def90b6f78f8b0aca9");
    // Bob is receiving from Alice
    let decrypted = <Vec<u8> as CryptoData<ECDSAKey>>::decrypt_with_secret_key(&mut encrypted, &bob_key_pair, &alice_key_pair);
    assert!(decrypted.is_some(), "Decrypted data is None");
    let decrypted = decrypted.unwrap();
    let decrypted_str = String::from_utf8(decrypted).unwrap();
    assert_eq!(secret, decrypted_str.as_str(), "they should be the same string");
}

#[test]
fn test_bls_encryption_and_decryption() {
    let base64_engine = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());
    let script_map = ChainType::TestNet.script_map();
    let alice_seed = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let alice_key_pair = BLSKey::key_with_seed_data(&alice_seed, true);
    let alice_public_key_data = alice_key_pair.public_key_data();
    let alice_private_key_data = alice_key_pair.private_key_data().unwrap();
    let alice_address = address::with_public_key_data(&alice_public_key_data, &script_map);
    assert_eq!(alice_public_key_data.to_hex(), "1790635de8740e9a6a6b15fb6b72f3a16afa0973d971979b6ba54761d6e2502c50db76f4d26143f05459a42cfd520d44", "BLS publicKeyData is incorrect");
    assert_eq!(base64_engine.encode(alice_public_key_data), "F5BjXeh0DppqaxX7a3LzoWr6CXPZcZeba6VHYdbiUCxQ23b00mFD8FRZpCz9Ug1E", "BLS publicKeyData is incorrect");
    assert_eq!(alice_private_key_data.to_hex(), "46891c2cec49593c81921e473db7480029e0fc1eb933c6b93d81f5370eb19fbd", "BLS privateKeyData is incorrect");
    assert_eq!(base64_engine.encode(alice_private_key_data), "RokcLOxJWTyBkh5HPbdIACng/B65M8a5PYH1Nw6xn70=", "BLS privateKeyData is incorrect");
    assert_eq!(alice_address, "yi4HkZyrJQTKRD6p6p6Akiq7d1j1uBMYFP", "BLS address for testnet is incorrect");

    let bob_seed = [10, 9, 8, 7, 6, 6, 7, 8, 9, 10];
    let bob_key_pair = BLSKey::key_with_seed_data(&bob_seed, true);
    let bob_public_key_data = bob_key_pair.public_key_data();
    let bob_private_key_data = bob_key_pair.private_key_data().unwrap();
    let bob_address = address::with_public_key_data(&bob_public_key_data, &script_map);

    assert_eq!(bob_public_key_data.to_hex(), "0e2f9055c17eb13221d8b41833468ab49f7d4e874ddf4b217f5126392a608fd48ccab3510548f1da4f397c1ad4f8e01a", "BLS publicKeyData is incorrect");
    assert_eq!(base64_engine.encode(bob_public_key_data), "Di+QVcF+sTIh2LQYM0aKtJ99TodN30shf1EmOSpgj9SMyrNRBUjx2k85fBrU+OAa", "BLS publicKeyData is incorrect");
    assert_eq!(bob_private_key_data.to_hex(), "2513a9d824e763f8b3ff4304c5d52d05154a82b4c975da965f124e5dcf915805", "BLS privateKeyData is incorrect");
    assert_eq!(base64_engine.encode(bob_private_key_data), "JROp2CTnY/iz/0MExdUtBRVKgrTJddqWXxJOXc+RWAU=", "BLS privateKeyData is incorrect");
    assert_eq!(bob_address, "yMfTGcBjCLxyefxAdSSyFnSYgU6cJzmrs2", "BLS address for testnet is incorrect");

    let secret = "my little secret is a pony that never sleeps";
    let mut data = secret.as_bytes().to_vec();
    // Alice is sending to Bob
    let iv = Vec::from_hex("eac5bcd6eb85074759e0261497428c9b").unwrap();
    let encrypted = <Vec<u8> as CryptoData<BLSKey>>::encrypt_with_secret_key_using_iv(&mut data, &alice_key_pair, &bob_key_pair, iv);
    assert!(encrypted.is_some(), "Encrypted data is None");
    let mut encrypted = encrypted.unwrap();
    assert_eq!(encrypted.to_hex(), "eac5bcd6eb85074759e0261497428c9bd72bd418ce96e69cbb6766e59f8d1f8138afb0686018bb4d401369e77ba47367f93a49a528f4cc9e3f209a515e6dd8f2");
    // Bob is receiving from Alice
    let decrypted = <Vec<u8> as CryptoData<BLSKey>>::decrypt_with_secret_key(&mut encrypted, &bob_key_pair, &alice_key_pair);
    assert!(decrypted.is_some(), "Decrypted data is None");
    let decrypted = decrypted.unwrap();
    let decrypted_str = String::from_utf8(decrypted).unwrap();
    assert_eq!(secret, decrypted_str.as_str(), "they should be the same string");
}

#[test]
pub fn test_base64_extended_public_key_size() {
    let base64_engine = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());
    let alice_seed = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let extended_private_key_data = Vec::from_hex("0000000106f68e209200000000ba3b3032119e384bc0f89ef795bfeebaa738be14c04733a6bace32b4a9370928064b824c2c47493a3a123356688033fc12d2464dca59ebe9978c801d14a84b7c").unwrap();
    let extended_public_key_data = Vec::from_hex("0000000106f68e209200000000ba3b3032119e384bc0f89ef795bfeebaa738be14c04733a6bace32b4a937092892a8b8dc09228e56932aafed768fb5ec8579f284917be9f489c222950e62497ddaeafa87a15219df72627796a3588b08").unwrap();
    // let alice_key_pair = BLSKey::key_with_extended_private_key_data(&extended_private_key_data, true).unwrap();
    let alice_key_pair = BLSKey::key_with_secret_hex("064b824c2c47493a3a123356688033fc12d2464dca59ebe9978c801d14a84b7c", true).unwrap();

    let bob_seed = [10, 9, 8, 7, 6, 6, 7, 8, 9, 10];
    let bob_key_pair = BLSKey::key_with_seed_data(&bob_seed, true);
    let mut ext_pk_data = Vec::from_hex("351973adaa8073a0ac848c08ba1c6df9a14d3c52033febe9bf4c5b365546a163bac5c8180240b908657221ebdc8fde7cd3017531159a7c58b955db380964c929dc6a85ac86").unwrap();
    let encrypted = <Vec<u8> as CryptoData<BLSKey>>::encrypt_with_secret_key(&mut ext_pk_data, &alice_key_pair, &bob_key_pair).unwrap();
    let base64_data = base64_engine.encode(encrypted);
    assert_eq!(base64_data.len(), 128, "BLS privateKeyData is incorrect");
}
