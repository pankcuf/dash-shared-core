use std::sync::Weak;
use hashes::hex::FromHex;
use crate::chain::common::ChainType;
use crate::chain::derivation::derivation_path::DerivationPath;
use crate::chain::derivation::derivation_path_feature_purpose::DerivationPathFeaturePurpose;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::derivation_path_type::DerivationPathType;
use crate::chain::derivation::index_path::{IIndexPath, IndexPath};
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::ext::wallets::WalletCreation;
use crate::chains_manager::ChainsManager;
use crate::crypto::UInt256;
use crate::keys::{Key, KeyKind};

const SEED_PHRASE: &str = "birth kingdom trash renew flavor utility donkey gasp regular alert pave layer";

#[test]
fn test_256_bit_path_ecdsa_derivation1() {
    let chain_type = ChainType::TestNet;
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(SEED_PHRASE).unwrap();
    let manager = ChainsManager::new();
    manager.testnet.wallet_with_seed(seed.clone(), false, chain_type);
    let wallet = manager.testnet.try_write().unwrap().wallets.first().unwrap();

    // m/0x775d3854c910b7dee436869c4724bed2fe0784e198b8a39f02bbb49d8ebcfc3b/0xf537439f36d04a15474ff7423e4b904a14373fafb37a41db74c84f1dbb5c89a6'/0x4c4592ca670c983fc43397dfd21a6f427fac9b4ac53cb4dcdc6522ec51e81e79/0

    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![
            UInt256::from_hex("775d3854c910b7dee436869c4724bed2fe0784e198b8a39f02bbb49d8ebcfc3b").unwrap(),
            UInt256::from_hex("f537439f36d04a15474ff7423e4b904a14373fafb37a41db74c84f1dbb5c89a6").unwrap(),
            UInt256::from_hex("4c4592ca670c983fc43397dfd21a6f427fac9b4ac53cb4dcdc6522ec51e81e79").unwrap(),
        ],
        vec![false, true, false],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Unknown,
        chain_type,
        Weak::new());
    match path.private_key_at_index_path_from_seed(&IndexPath::index_path_with_index(0), &seed) {
        Some(Key::ECDSA(key)) => {
            assert_eq!(key.secret_key_string(), "e8781fdef72862968cd9a4d2df34edaf9dcc5b17629ec505f0d2d1a8ed6f9f09", "keys should match");
            path.generate_extended_public_key_from_seed_no_store(&seed.data);
            let serialized_ext_pubkey = path.serialized_extended_public_key().unwrap();
            assert_eq!(serialized_ext_pubkey, "dptp1CjRySByBWNBUgwM6mo6RE3zncnqhfSSedX7De8HzSEdoYgzyuUs1Pdbprcu27dEZ6ahLrnHapqswbbMoExT3ZMq7CaaBKPfS2xqwMJLsxU3kLhXp4kfsYcpeB7ksLFseMGGFqaQ8qtpjLGHhx4", "serialized extended public keys should match");
            let serialized_ext_seckey = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
            assert_eq!(serialized_ext_seckey, "dpts1wL7C3vjxN7SNxNTC12E4nmD7VKVSyCQmdwW9yLM8ehJcCPjWuGHYE8wK7tRNWj764Ec7FGB25Aji74VzURCDZusNq3hvszaQmj8C5WxDjDmLgYZuhxrVyiGBXuda3Uzk5qYcnGTZC6KtJvvMo6", "serialized extended private keys should match");
        },
        _ => panic!("Can't get ecdsa key at index path from seed")
    }
}

#[test]
fn test_256_bit_path_ecdsa_derivation2() {
    let chain_type = ChainType::TestNet;
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(SEED_PHRASE).unwrap();
    let manager = ChainsManager::new();
    manager.testnet.wallet_with_seed(seed.clone(), false, chain_type);
    let wallet = manager.testnet.try_write().unwrap().wallets.first().unwrap();
    // m/9'/5'/15'/0'/0x555d3854c910b7dee436869c4724bed2fe0784e198b8a39f02bbb49d8ebcfc3a'/0xa137439f36d04a15474ff7423e4b904a14373fafb37a41db74c84f1dbb5c89b5'/0
    let path = DerivationPath::derivation_path_with_indexes(
        vec![
            UInt256::from(DerivationPathFeaturePurpose::Default),
            UInt256::from(5u64),
            UInt256::from(DerivationPathFeaturePurpose::DashPay),
            UInt256::from(0u64),
            UInt256::from_hex("555d3854c910b7dee436869c4724bed2fe0784e198b8a39f02bbb49d8ebcfc3a").unwrap(),
            UInt256::from_hex("a137439f36d04a15474ff7423e4b904a14373fafb37a41db74c84f1dbb5c89b5").unwrap(),
        ],
        vec![true, true, true, true, true, true],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Unknown,
        chain_type,
        Weak::new());
    match path.private_key_at_index_path_from_seed(&IndexPath::index_path_with_index(0), &seed) {
        Some(Key::ECDSA(key)) => {
            assert_eq!(key.secret_key_string(), "fac40790776d171ee1db90899b5eb2df2f7d2aaf35ad56f07ffb8ed2c57f8e60", "keys should match");
        },
        _ => panic!("Can't get ecdsa key at index path from seed")
    }
}

#[test]
fn test_256_bit_path_ecdsa_derivation3() {
    let chain_type = ChainType::TestNet;
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(SEED_PHRASE).unwrap();
    let manager = ChainsManager::new();
    manager.testnet.wallet_with_seed(seed.clone(), false, chain_type);
    let wallet = manager.testnet.try_write().unwrap().wallets.first().unwrap();
    //m/0x775d3854c910b7dee436869c4724bed2fe0784e198b8a39f02bbb49d8ebcfc3b
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![
            UInt256::from_hex("775d3854c910b7dee436869c4724bed2fe0784e198b8a39f02bbb49d8ebcfc3b").unwrap(),
        ],
        vec![false],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Unknown,
        chain_type,
        Weak::new());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_ext_pubkey = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_ext_pubkey, "dptp1C5gGd8NzvAke5WNKyRfpDRyvV2UZ3jjrZVZU77qk9yZemMGSdZpkWp7y6wt3FzvFxAHSW8VMCaC1p6Ny5EqWuRm2sjvZLUUFMMwXhmW6eS69qjX958RYBH5R8bUCGZkCfUyQ8UVWcx9katkrRr", "serialized extended public keys should match");
    let serialized_ext_seckey = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_ext_seckey, "dpts1vgMVEs9mmv1YLwURCeoTn9CFMZ8JMVhyZuxQSKttNSETR3zydMFHMKTTNDQPf6nnupCCtcNnSu3nKZXAJhaguyoJWD4Ju5PE6PSkBqAKWci7HLz37qmFmZZU6GMkLvNLtST2iV8NmqqbX37c45", "serialized extended private keys should match");
}

#[test]
fn test_256_bit_path_ecdsa_derivation4() {
    let chain_type = ChainType::TestNet;
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(SEED_PHRASE).unwrap();
    let manager = ChainsManager::new();
    manager.testnet.wallet_with_seed(seed.clone(), false, chain_type);
    let wallet = manager.testnet.try_write().unwrap().wallets.first().unwrap();
    //m/0x775d3854c910b7dee436869c4724bed2fe0784e198b8a39f02bbb49d8ebcfc3b/0xf537439f36d04a15474ff7423e4b904a14373fafb37a41db74c84f1dbb5c89a6'
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![
            UInt256::from_hex("775d3854c910b7dee436869c4724bed2fe0784e198b8a39f02bbb49d8ebcfc3b").unwrap(),
            UInt256::from_hex("f537439f36d04a15474ff7423e4b904a14373fafb37a41db74c84f1dbb5c89a6").unwrap(),
        ],
        vec![false, true],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Unknown,
        chain_type,
        Weak::new());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_ext_pubkey = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_ext_pubkey, "dptp1CLkexeadp6guoi8Fbiwq6CLZm3hT1DJLwHsxWvwYSeAhjenFhcQ9HumZSftfZEr4dyQjFD7gkM5bSn6Aj7F1Jve8KTn4JsMEaj9dFyJkYs4Ga5HSUqeajxGVmzaY1pEioDmvUtZL3J1NCDCmzQ", "serialized extended public keys should match");
    let serialized_ext_seckey = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_ext_seckey, "dpts1vwRsaPMQfqwp59ELpx5UeuYtdaMCJyGTwiGtr8zgf6qWPMWnhPpg8R73hwR1xLibbdKVdh17zfwMxFEMxZzBKUgPwvuosUGDKW4ayZjs3AQB9EGRcVpDoFT8V6nkcc6KzksmZxvmDcd3MqiPEu", "serialized extended private keys should match");
}
