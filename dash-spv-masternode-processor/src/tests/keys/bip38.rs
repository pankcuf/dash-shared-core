use crate::chain::bip::bip38::BIP38;
use crate::chain::common::ChainType;
use crate::keys::{ECDSAKey, IKey};

#[test]
fn test_key_with_bip38_key() {
    let chain_type = ChainType::MainNet;
    let script = chain_type.script_map();
    let key = ECDSAKey::key_with_bip38_key("6PfV898iMrVs3d9gJSw5HTYyGhQRR5xRu5ji4GE6H5QdebT2YgK14Lu1E5", "TestingOneTwoThree", &script).unwrap();
    // to do compressed/uncompressed BIP38Key tests
    let serialized_key = key.serialized_private_key_for_script(&script);
    println!("privKey = {}", serialized_key);
    assert_eq!("7sEJGJRPeGoNBsW8tKAk4JH52xbxrktPfJcNxEx3uf622ZrGR5k", serialized_key, "key_with_bip38_key: wrong result");
    assert_eq!(key.bip38_key_with_passphrase("TestingOneTwoThree", &script).unwrap(), "6PRT3Wy4p7MZETE3n56KzyjyizMsE26WnMWpSeSoZawawEm7jaeCVa2wMu", "key_with_bip38_key: wrong result");

    let key = ECDSAKey::key_with_bip38_key("6PRW5o9FLp4gJDDVqJQKJFTpMvdsSGJxMYHtHaQBF3ooa8mwD69bapcDQn", "foobar", &script);
    assert!(key.is_none(), "Should be none");
}
