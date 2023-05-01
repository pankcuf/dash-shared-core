use std::sync::Weak;
use std::time::SystemTime;
use std::vec;
use base64::{alphabet, Engine};
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use bls_signatures::bip32::ExtendedPrivateKey;
use hashes::hex::{FromHex, ToHex};
// use crate::chain::bip::dip14::{derive_child_private_key, derive_child_private_key_256, derive_child_public_key, derive_child_public_key_256};
use crate::chain::common::ChainType;
use crate::chain::ext::wallets::WalletCreation;
use crate::chain::wallet::seed::Seed;
use crate::chains_manager::ChainsManager;
use crate::crypto::byte_util::AsBytes;
use crate::crypto::{ECPoint, UInt256, UInt512};
use crate::chain::derivation::BIP32_HARD;
use crate::chain::derivation::authentication_keys_derivation_path::AuthenticationKeysDerivationPath;
use crate::chain::derivation::derivation_path::DerivationPath;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::derivation_path_type::DerivationPathType;
use crate::chain::derivation::incoming_funds_derivation_path::IncomingFundsDerivationPath;
use crate::chain::derivation::index_path::{IIndexPath, IndexPath};
use crate::chain::derivation::protocol::IDerivationPath;
use crate::keys::{BLSKey, CryptoData, ECDSAKey, IKey, Key, KeyKind};
use crate::keys::dip14::IChildKeyDerivation;
use crate::util::{base58, TimeUtil};

#[test]
fn fingerprint_for_short_bip32_seed() {
    assert_eq!(ExtendedPrivateKey::from_seed(&[1u8, 50, 6, 244, 24, 199, 1, 25])
                   .expect("cannot generate extended private key")
                   .public_key()
                   .expect("cannot get public key from extended private key")
                   .fingerprint_legacy(), 0xa4700b27);
}

#[test]
fn test_bls_fingerprint_from_seed() {
    let seed = Seed::with_data([1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10].to_vec());
    assert_eq!(
        BLSKey::key_with_seed_data(&seed.data, true)
            .public_key_fingerprint(),
        0xddad59bb, "Testing BLS private child public key fingerprint");
    let seed_data = [1u8, 50, 6, 244, 24, 199, 1, 25].to_vec();
    assert_eq!(BLSKey::extended_private_key_with_seed_data(&seed_data, true)
                .expect("Can't get extended_private_key from seed")
                .public_key_fingerprint(),
            0xa4700b27, "Testing BLS extended private child public key fingerprint");
}

#[test]
fn test_bls_derivation() {
    let chain_type = ChainType::MainNet;
    let seed_data = [1u8, 50, 6, 244, 24, 199, 1, 25].to_vec();
    let mut key_pair = BLSKey::extended_private_key_with_seed_data(&seed_data, true).unwrap();
    let chain_code = key_pair.chaincode;
    assert_eq!(chain_code.0.to_hex(), "d8b12555b4cc5578951e4a7c80031e22019cc0dce168b3ed88115311b8feb1e3", "Testing BLS derivation chain code");
    let derivation_path_indexes1 = vec![UInt256::from(77u64)];
    let hardened1 = vec![true];
    let derivation_path1 = DerivationPath::derivation_path_with_indexes(
        derivation_path_indexes1,
        hardened1,
        DerivationPathType::ClearFunds,
        KeyKind::BLS,
        DerivationPathReference::Unknown,
        chain_type,
        Weak::new()
    );
    let key_pair1 = key_pair.private_derive_to_path(&derivation_path1.base_index_path()).unwrap();
    let chain_code1 = key_pair1.chaincode;
    assert_eq!(chain_code1.0.to_hex(), "f2c8e4269bb3e54f8179a5c6976d92ca14c3260dd729981e9d15f53049fd698b", "Testing BLS private child derivation returning chain code");
    assert_eq!(key_pair1.public_key_fingerprint(), 0xa8063dcf, "Testing BLS extended private child public key fingerprint");

    let derivation_path_indexes2 = vec![UInt256::from(3u64), UInt256::from(17u64)];
    let hardened2 = vec![false, false];
    let derivation_path2 = DerivationPath::derivation_path_with_indexes(
        derivation_path_indexes2,
        hardened2,
        DerivationPathType::ClearFunds,
        KeyKind::BLS,
        DerivationPathReference::Unknown,
        chain_type,
        Weak::new()
    );
    let key_pair2 = key_pair.private_derive_to_path(&derivation_path2.base_index_path()).unwrap();
    assert_eq!(key_pair2.public_key_fingerprint(), 0xff26a31f, "Testing BLS extended private child public key fingerprint");
    let key_pair3 = key_pair.public_derive_to_path(&derivation_path2.base_index_path()).unwrap();
    assert_eq!(key_pair3.public_key_fingerprint(), 0xff26a31f, "Testing BLS extended private child public key fingerprint");
}

#[test]
fn test_bip32_sequence_private_key_from_string() {
    let manager = ChainsManager::new();
    let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
    let mnemonic = bip0039::Mnemonic::<bip0039::English>::from_phrase(seed_phrase);
    assert!(mnemonic.is_ok(), "Error parsing seed");
    let seed_data = Vec::from_hex("000102030405060708090a0b0c0d0e0f").unwrap();
    let seed = Seed::with_data(seed_data.clone());
    let chain_type = ChainType::MainNet;
    // let wallet_arc = manager.wallet_with_seed_phrase::<bip0039::English>(seed_phrase, true, SystemTime::seconds_since_1970(), chain_type).unwrap();
    // let wallet = wallet_arc.try_write().unwrap();
    manager.mainnet.wallet_with_seed(Seed::with_data(seed_data.clone()), false, chain_type);
    let chain = manager.mainnet.read().unwrap();
    let wallet = chain.wallets.first().unwrap();
    let account = wallet.account_with_number(0).unwrap();
    let derivation_path = account.bip32_derivation_path.as_ref().unwrap();
    let mut pk = derivation_path.private_key_string_at_index(2 | BIP32_HARD, true, &seed).unwrap();
    let mut d = base58::from_check(pk.as_str()).unwrap();
    println!("000102030405060708090a0b0c0d0e0f/0'/1/2' prv = {}", d.to_hex());
    assert_eq!(d.to_hex(), "cccbce0d719ecf7431d88e6a89fa1483e02e35092af60c042b1df2ff59fa424dca01", "private_key_string_at_index");
    // Test for correct zero padding of private keys, a nasty potential bug
    pk = derivation_path.private_key_string_at_index(97, false, &seed).unwrap();
    d = base58::from_check(pk.as_str()).unwrap();
    println!("000102030405060708090a0b0c0d0e0f/0'/0/97 prv = {}", d.to_hex());
    assert_eq!(d.to_hex(), "cc00136c1ad038f9a00871895322a487ed14f1cdc4d22ad351cfa1a0d235975dd701", "private_key_string_at_index");
}

#[test]
fn test_bip32_serializations_basic() {
    let manager = ChainsManager::new();
    let chain_type = ChainType::MainNet;
    let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
    // let wallet_arc = manager.wallet_with_seed_phrase::<bip0039::English>(seed_phrase, true, SystemTime::seconds_since_1970(), chain_type).unwrap();
    // let mut wallet = wallet_arc.try_write().unwrap();
    manager.mainnet.wallet_with_seed_phrase::<bip0039::English>(seed_phrase, false, SystemTime::seconds_since_1970(), chain_type);
    let mut chain = manager.mainnet.try_write().unwrap();
    let wallet = chain.wallets.first_mut().unwrap();
    //--------------------------------------------------------------------------------------------------//
    // m //
    //--------------------------------------------------------------------------------------------------//
    let seed_data = Vec::from_hex("000102030405060708090a0b0c0d0e0f").unwrap();
    let seed = Seed::with_data(seed_data);
    println!("••••••••••••••••••••••••••••••••••••••");
    let indexes_root = Vec::<UInt256>::new();
    let hardened_root = Vec::<bool>::new();
    let mut root_derivation_path = DerivationPath::derivation_path_with_indexes(
        indexes_root,
        hardened_root,
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    root_derivation_path.set_is_transient(true);
    root_derivation_path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    root_derivation_path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_root_extended_public_key = root_derivation_path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_root_extended_public_key, "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8", "serialized extended public key is wrong");
    let serialized_root_extended_private_key = root_derivation_path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_root_extended_private_key, "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi", "serialized extended private key is wrong");
    println!("••••••••••••••••••••••••••••••••••••••");
    //--------------------------------------------------------------------------------------------------//
    // m/0' //
    //--------------------------------------------------------------------------------------------------//
    let account = wallet.account_with_number_mut(0).unwrap();
    let bip32_derivation_path = account.bip32_derivation_path.as_mut().unwrap();
    bip32_derivation_path.generate_extended_public_key_from_seed(&seed)
        .expect("generate_extended_public_key_from_seed");
    let serialized_bip32_extended_public_key = bip32_derivation_path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_bip32_extended_public_key, "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw", "serialized extended public key is wrong");
    let serialized_bip32_extended_private_key = bip32_derivation_path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_bip32_extended_private_key, "xprv9uHRZZhk6KAJC1avXpDAp4MDc3sQKNxDiPvvkX8Br5ngLNv1TxvUxt4cV1rGL5hj6KCesnDYUhd7oWgT11eZG7XnxHrnYeSvkzY7d2bhkJ7", "serialized extended private key is wrong");
    println!("••••••••••••••••••••••••••••••••••••••");
    //--------------------------------------------------------------------------------------------------//
    // m/0'/1 //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![UInt256::from(0u64), UInt256::from(1u64)],
        vec![true, false],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub6ASuArnXKPbfEwhqN6e3mwBcDTgzisQN1wXN9BJcM47sSikHjJf3UFHKkNAWbWMiGj7Wf5uMash7SyYq527Hqck2AxYysAA7xmALppuCkwQ", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprv9wTYmMFdV23N2TdNG573QoEsfRrWKQgWeibmLntzniatZvR9BmLnvSxqu53Kw1UmYPxLgboyZQaXwTCg8MSY3H2EU4pWcQDnRnrVA1xe8fs", "serialized extended private key is wrong");
    println!("••••••••••••••••••••••••••••••••••••••");
    //--------------------------------------------------------------------------------------------------//
    // m/0'/1/2' //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![UInt256::from(0u64), UInt256::from(1u64), UInt256::from(2u64)],
        vec![true, false, true],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,


        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub6D4BDPcP2GT577Vvch3R8wDkScZWzQzMMUm3PWbmWvVJrZwQY4VUNgqFJPMM3No2dFDFGTsxxpG5uJh7n7epu4trkrX7x7DogT5Uv6fcLW5", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprv9z4pot5VBttmtdRTWfWQmoH1taj2axGVzFqSb8C9xaxKymcFzXBDptWmT7FwuEzG3ryjH4ktypQSAewRiNMjANTtpgP4mLTj34bhnZX7UiM", "serialized extended private key is wrong");
    //--------------------------------------------------------------------------------------------------//
    // m/0'/1/2'/2 //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![UInt256::from(0u64), UInt256::from(1u64), UInt256::from(2u64), UInt256::from(2u64)],
        vec![true, false, true, false],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub6FHa3pjLCk84BayeJxFW2SP4XRrFd1JYnxeLeU8EqN3vDfZmbqBqaGJAyiLjTAwm6ZLRQUMv1ZACTj37sR62cfN7fe5JnJ7dh8zL4fiyLHV", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprvA2JDeKCSNNZky6uBCviVfJSKyQ1mDYahRjijr5idH2WwLsEd4Hsb2Tyh8RfQMuPh7f7RtyzTtdrbdqqsunu5Mm3wDvUAKRHSC34sJ7in334", "serialized extended private key is wrong");
    //--------------------------------------------------------------------------------------------------//
    // m/0'/1/2'/2/1000000000 //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![UInt256::from(0u64), UInt256::from(1u64), UInt256::from(2u64), UInt256::from(2u64), UInt256::from(1000000000u64)],
        vec![true, false, true, false, false],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub6H1LXWLaKsWFhvm6RVpEL9P4KfRZSW7abD2ttkWP3SSQvnyA8FSVqNTEcYFgJS2UaFcxupHiYkro49S8yGasTvXEYBVPamhGW6cFJodrTHy", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprvA41z7zogVVwxVSgdKUHDy1SKmdb533PjDz7J6N6mV6uS3ze1ai8FHa8kmHScGpWmj4WggLyQjgPie1rFSruoUihUZREPSL39UNdE3BBDu76", "serialized extended private key is wrong");
}

#[test]
fn test_bip32_serializations_advanced() {
    let manager = ChainsManager::new();
    let chain_type = ChainType::MainNet;
    let seed_data = Vec::from_hex("fffcf9f6f3f0edeae7e4e1dedbd8d5d2cfccc9c6c3c0bdbab7b4b1aeaba8a5a29f9c999693908d8a8784817e7b7875726f6c696663605d5a5754514e4b484542").unwrap();
    let seed = chain_type.seed_for_seed_data::<bip0039::English>(seed_data);
    manager.mainnet.wallet_with_seed(seed.clone(), false, chain_type);
    let chain = manager.mainnet.read().unwrap();
    let wallet = chain.wallets.first().unwrap();
    //--------------------------------------------------------------------------------------------------//
    // m //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![],
        vec![],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprv9s21ZrQH143K31xYSDQpPDxsXRTUcvj2iNHm5NUtrGiGG5e2DtALGdso3pGz6ssrdK4PFmM8NSpSBHNqPqm55Qn3LqFtT2emdEXVYsCzC2U", "serialized extended private key is wrong");
    //--------------------------------------------------------------------------------------------------//
    // m/0 //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![UInt256::from(0u64)],
        vec![false],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub69H7F5d8KSRgmmdJg2KhpAK8SR3DjMwAdkxj3ZuxV27CprR9LgpeyGmXUbC6wb7ERfvrnKZjXoUmmDznezpbZb7ap6r1D3tgFxHmwMkQTPH", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprv9vHkqa6EV4sPZHYqZznhT2NPtPCjKuDKGY38FBWLvgaDx45zo9WQRUT3dKYnjwih2yJD9mkrocEZXo1ex8G81dwSM1fwqWpWkeS3v86pgKt", "serialized extended private key is wrong");
    //--------------------------------------------------------------------------------------------------//
    // m/0/2147483647' //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![UInt256::from(0u64), UInt256::from(2147483647u64)],
        vec![false, true],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub6ASAVgeehLbnwdqV6UKMHVzgqAG8Gr6riv3Fxxpj8ksbH9ebxaEyBLZ85ySDhKiLDBrQSARLq1uNRts8RuJiHjaDMBU4Zn9h8LZNnBC5y4a", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprv9wSp6B7kry3Vj9m1zSnLvN3xH8RdsPP1Mh7fAaR7aRLcQMKTR2vidYEeEg2mUCTAwCd6vnxVrcjfy2kRgVsFawNzmjuHc2YmYRmagcEPdU9", "serialized extended private key is wrong");
    //--------------------------------------------------------------------------------------------------//
    // m/0/2147483647'/1 //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![UInt256::from(0u64), UInt256::from(2147483647u64), UInt256::from(1u64)],
        vec![false, true, false],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub6DF8uhdarytz3FWdA8TvFSvvAh8dP3283MY7p2V4SeE2wyWmG5mg5EwVvmdMVCQcoNJxGoWaU9DCWh89LojfZ537wTfunKau47EL2dhHKon", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprv9zFnWC6h2cLgpmSA46vutJzBcfJ8yaJGg8cX1e5StJh45BBciYTRXSd25UEPVuesF9yog62tGAQtHjXajPPdbRCHuWS6T8XA2ECKADdw4Ef", "serialized extended private key is wrong");
    //--------------------------------------------------------------------------------------------------//
    // m/0/2147483647'/1/2147483646' //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![UInt256::from(0u64), UInt256::from(2147483647u64), UInt256::from(1u64), UInt256::from(2147483646u64)],
        vec![false, true, false, true],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub6ERApfZwUNrhLCkDtcHTcxd75RbzS1ed54G1LkBUHQVHQKqhMkhgbmJbZRkrgZw4koxb5JaHWkY4ALHY2grBGRjaDMzQLcgJvLJuZZvRcEL", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprvA1RpRA33e1JQ7ifknakTFpgNXPmW2YvmhqLQYMmrj4xJXXWYpDPS3xz7iAxn8L39njGVyuoseXzU6rcxFLJ8HFsTjSyQbLYnMpCqE2VbFWc", "serialized extended private key is wrong");
    //--------------------------------------------------------------------------------------------------//
    // m/0/2147483647'/1/2147483646'/2 //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![UInt256::from(0u64), UInt256::from(2147483647u64), UInt256::from(1u64), UInt256::from(2147483646u64), UInt256::from(2u64)],
        vec![false, true, false, true, false],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub6FnCn6nSzZAw5Tw7cgR9bi15UV96gLZhjDstkXXxvCLsUXBGXPdSnLFbdpq8p9HmGsApME5hQTZ3emM2rnY5agb9rXpVGyy3bdW6EEgAtqt", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprvA2nrNbFZABcdryreWet9Ea4LvTJcGsqrMzxHx98MMrotbir7yrKCEXw7nadnHM8Dq38EGfSh6dqA9QWTyefMLEcBYJUuekgW4BYPJcr9E7j", "serialized extended private key is wrong");
}

#[test]
fn test_bip32_serializations_leading_zeroes() {
    let manager = ChainsManager::new();
    let chain_type = ChainType::MainNet;
    let seed_data = Vec::from_hex("4b381541583be4423346c643850da4b320e46a87ae3d2a4e6da11eba819cd4acba45d239319ac14f863b8d5ab5a0d0c64d2e8a1e7d1457df2e5a3c51c73235be").unwrap();
    let seed = chain_type.seed_for_seed_data::<bip0039::English>(seed_data);
    manager.mainnet.wallet_with_seed(seed.clone(), false, chain_type);
    let chain = manager.mainnet.read().unwrap();
    let wallet = chain.wallets.first().unwrap();
    //--------------------------------------------------------------------------------------------------//
    // m //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![],
        vec![],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprv9s21ZrQH143K25QhxbucbDDuQ4naNntJRi4KUfWT7xo4EKsHt2QJDu7KXp1A3u7Bi1j8ph3EGsZ9Xvz9dGuVrtHHs7pXeTzjuxBrCmmhgC6", "serialized extended private key is wrong");
    //--------------------------------------------------------------------------------------------------//
    // m/0' //
    //--------------------------------------------------------------------------------------------------//
    let mut path = DerivationPath::derivation_path_with_indexes(
        vec![UInt256::from(0u64)],
        vec![true],
        DerivationPathType::Unknown,
        KeyKind::ECDSA,
        DerivationPathReference::Root,
        chain_type,
        Weak::new());
    path.set_is_transient(true);
    path.set_wallet_unique_id(wallet.unique_id_as_str().to_string());
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let serialized_extended_public_key = path.serialized_extended_public_key().unwrap();
    assert_eq!(serialized_extended_public_key, "xpub68NZiKmJWnxxS6aaHmn81bvJeTESw724CRDs6HbuccFQN9Ku14VQrADWgqbhhTHBaohPX4CjNLf9fq9MYo6oDaPPLPxSb7gwQN3ih19Zm4Y", "serialized extended public key is wrong");
    let serialized_extended_private_key = path.serialized_extended_private_key_from_seed(&seed.data).unwrap();
    assert_eq!(serialized_extended_private_key, "xprv9uPDJpEQgRQfDcW7BkF7eTya6RPxXeJCqCJGHuCJ4GiRVLzkTXBAJMu2qaMWPrS7AANYqdq6vcBcBUdJCVVFceUvJFjaPdGZ2y9WACViL4L", "serialized extended private key is wrong");
}
#[test]
fn test_bip32_sequence_master_public_key_from_seed() {
    let manager = ChainsManager::new();
    let chain_type = ChainType::MainNet;
    let seed_data = Vec::from_hex("000102030405060708090a0b0c0d0e0f").unwrap();
    let seed = chain_type.seed_for_seed_data::<bip0039::English>(seed_data);
    manager.mainnet.wallet_with_seed(seed.clone(), false, chain_type);
    let mut chain = manager.mainnet.try_write().unwrap();
    let wallet = chain.wallets.first_mut().unwrap();
    let account = wallet.account_with_number_mut(0).unwrap();
    let bip32_derivation_path = account.bip32_derivation_path.as_mut().unwrap();
    bip32_derivation_path.generate_extended_public_key_from_seed(&seed)
        .expect("generate_extended_public_key_from_seed");
    let mpk = bip32_derivation_path.extended_public_key_data().unwrap();
    println!("000102030405060708090a0b0c0d0e0f/0' pub+chain = {}", mpk.to_hex());
    assert_eq!(mpk.to_hex(), "3442193e47fdacbd0f1097043b78c63c20c34ef4ed9a111d980047ad16282c7ae6236141035a784662a4a20a65bf6aab9ae98a6c068a81c52e4b032c0fb5400c706cfccc56", "Wrong extended public key data for bip32 derivation path");
}

#[test]
fn test_bip32_sequence_public_key() {
    let manager = ChainsManager::new();
    let chain_type = ChainType::MainNet;
    let seed_data = Vec::from_hex("000102030405060708090a0b0c0d0e0f").unwrap();
    let seed = chain_type.seed_for_seed_data::<bip0039::English>(seed_data);
    manager.mainnet.wallet_with_seed(seed.clone(), false, chain_type);
    let mut chain = manager.mainnet.try_write().unwrap();
    let wallet = chain.wallets.first_mut().unwrap();
    let account = wallet.account_with_number_mut(0).unwrap();
    let bip32_derivation_path = account.bip32_derivation_path.as_mut().unwrap();
    bip32_derivation_path.generate_extended_public_key_from_seed(&seed);
    let pubkeydata = bip32_derivation_path.public_key_data_at_index(0, false).unwrap();
    println!("000102030405060708090a0b0c0d0e0f/0'/0/0 pub = {}", pubkeydata.to_hex());
    assert_eq!(pubkeydata, Vec::from_hex("027b6a7dd645507d775215a9035be06700e1ed8c541da9351b4bd14bd50ab61428").unwrap(), "can't get external public key data at index: 0");
}

#[test]
fn test_bip32_sequence_serialized_private_master_from_seed() {
    let chain_type = ChainType::MainNet;
    let seed_data = Vec::from_hex("bb22c8551ef39739fa007efc150975fce0187e675d74c804ab32f87fe0b9ad387fe9b044b8053dfb26cf9d7e4857617fa66430c880e7f4c96554b4eed8a0ad2f").unwrap();
    let xprv = ECDSAKey::serialized_private_master_key_from_seed(&seed_data, chain_type);
    println!("bb22c8551ef39739fa007efc150975fce0187e675d74c804ab32f87fe0b9ad387fe9b044b8053dfb26cf9d7e4857617fa66430c880e7f4c96554b4eed8a0ad2f xprv = {}", xprv);
    assert_eq!(xprv, "xprv9s21ZrQH143K27s8Yy6TJSKmKUxTBuXJr4RDTjJ5Jqq13d9v2VzYymSoM4VodDK7nrQHTruX6TuBsGuEVXoo91GwZnmBcTaqUhgK7HeysNv", "not as expected");
}

#[test]
fn test_bip32_sequence_serialized_master_public_key() {
    // stay issue box trade stock chaos raccoon candy obey wet refuse carbon silent guide crystal
    let manager = ChainsManager::new();
    let chain_type = ChainType::MainNet;
    {
        let seed_data = Vec::from_hex("bb22c8551ef39739fa007efc150975fce0187e675d74c804ab32f87fe0b9ad387fe9b044b8053dfb26cf9d7e4857617fa66430c880e7f4c96554b4eed8a0ad2f").unwrap();
        let seed = chain_type.seed_for_seed_data::<bip0039::English>(seed_data);
        manager.mainnet.wallet_with_seed(seed.clone(), false, chain_type);
        let mut chain = manager.mainnet.try_write().unwrap();
        let wallet = chain.wallets.first_mut().unwrap();
        let path = wallet.account_with_number_mut(0).unwrap().bip32_derivation_path.as_mut().unwrap();
        path.generate_extended_public_key_from_seed(&seed);
        let xpub = path.serialized_extended_public_key().unwrap();
        println!("bb22c8551ef39739fa007efc150975fce0187e675d74c804ab32f87fe0b9ad387fe9b044b8053dfb26cf9d7e4857617fa66430c880e7f4c96554b4eed8a0ad2f xpub = {}", xpub);
        assert_eq!(xpub, "xpub6949NHhpyXW7qCtj5eKxLG14JgbFdxUwRdmZ4M51t2Bcj95bCREEDmvdWhC6c31SbobAf5X86SLg76A5WirhTYFCG5F9wkeY6314q4ZtA68", "wrong serialized extended public key");
    }
    {
        let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
        let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(seed_phrase).unwrap();
        assert_eq!(seed.data.to_hex(), "467c2dd58bbd29427fb3c5467eee339021a87b21309eeabfe9459d31eeb6eba9b2a1213c12a173118c84fd49e8b4bf9282272d67bf7b7b394b088eab53b438bc", "wrong key derived from phrase");
        manager.mainnet.wallet_with_seed_phrase::<bip0039::English>(seed_phrase, false, SystemTime::seconds_since_1970(), chain_type);
        let mut chain = manager.mainnet.try_write().unwrap();
        // chain = manager.mainnet.try_write().unwrap();
        let wallet2 = chain.wallets.get_mut(1).unwrap();
        let path = wallet2.account_with_number_mut(0).unwrap().bip32_derivation_path.as_mut().unwrap();
        path.generate_extended_public_key_from_seed(&seed);
        let mpk = path.extended_public_key_data().unwrap();
        assert_eq!(mpk.to_hex(), "c93fa1867e984d7255df4736e7d7d6243026b9744e62374cbb54a0a47cc0fe0c334f876e02cdfeed62990ac98b6932e0080ce2155b4f5c7a8341271e9ee9c90cd87300009c", "extended public key data is wrong");
        let xpub = path.serialized_extended_public_key().unwrap();
        assert_eq!(xpub, "xpub69NHuRQrRn5GbT7j881uR64arreu3TFmmPAMnTeHdGd68BmAFxssxhzhmyvQoL3svMWTSbymV5FdHoypDDmaqV1C5pvnKbcse1vgrENbau7", "serialized extended public key is wrong");
    }
}

#[test]
fn test_bip44_sequence_serialized_master_public_key() {
    let manager = ChainsManager::new();
    let chain_type = ChainType::MainNet;
    let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(seed_phrase).unwrap();
    manager.mainnet.wallet_with_seed(seed.clone(), false, chain_type);
    let mut chain = manager.mainnet.try_write().unwrap();
    let wallet = chain.wallets.first_mut().unwrap();
    let path = wallet.account_with_number_mut(0).unwrap().bip44_derivation_path.as_mut().unwrap();
    path.generate_extended_public_key_from_seed(&seed);
    let mpk = path.extended_public_key_data().unwrap();
    assert_eq!(mpk.to_hex(), "4687e396a07188bd71458a0e90987f92b18a6451e99eb52f0060be450e0b4b3ce3e49f9f033914476cf503c7c2dcf5a0f90d3e943a84e507551bdf84891dd38c0817cca97a", "wrong bip44 extended public key data");
    let xpub = path.serialized_extended_public_key().unwrap();
    assert_eq!(xpub, "xpub6CAqVZYbGiQCTyzzvvueEoBy8M74VWtPywf2F3zpwbS8AugDSSMSLcewpDaRQxVCxtL4kbTbWb1fzWg2R5933ECsxrEtKBA4gkJu8quduHs", "wrong serialized bip44 extended public key");
    let de_mpk = DerivationPath::deserialized_extended_private_key_for_chain(&xpub, chain_type).unwrap();
    assert_eq!(mpk.to_hex(), de_mpk.to_hex(), "wrong deserialized extended private key for chain");
}

#[test]
fn test_31_bit_derivation() {
    let chain_type = ChainType::MainNet;
    let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(seed_phrase).unwrap();
    let i = UInt512::bip32_seed_key(&seed.data);
    let mut secret = UInt256::from(&i.0[..32]);
    let mut chain = UInt256::from(&i.0[32..]);
    let mut child_chain = chain.clone();
    let parent_secret = ECDSAKey::key_with_secret(&secret, true).unwrap();
    let parent_public_key = parent_secret.public_key_data();
    let index_path = IndexPath::index_path_with_index(0u32);
    ECDSAKey::derive_child_private_key(&mut secret, &mut chain, &index_path, 0);
    let public_key = ECDSAKey::key_with_secret(&secret, true).unwrap().public_key_data();
    let mut pubkey = ECPoint::from(&parent_public_key);
    ECDSAKey::derive_child_public_key(&mut pubkey, &mut child_chain, &index_path, 0);
    assert_eq!(chain, child_chain, "the bip32 chains must match");
    assert_eq!(&public_key, pubkey.as_bytes(), "the public keys must match");
}

#[test]
fn test_31_bit_compatibility_mode_derivation() {
    let chain_type = ChainType::MainNet;
    let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(seed_phrase).unwrap();
    let i = UInt512::bip32_seed_key(&seed.data);
    let mut secret = UInt256::from(&i.0[..32]);
    let mut chain = UInt256::from(&i.0[32..]);
    let mut child_chain = chain.clone();
    let parent_secret = ECDSAKey::key_with_secret(&secret, true).unwrap();
    let parent_public_key = parent_secret.public_key_data();
    let index_path = IndexPath::new_hardened(vec![UInt256::MIN], vec![false]);
    ECDSAKey::derive_child_private_key(&mut secret, &mut chain, &index_path, 0);
    let public_key = ECDSAKey::key_with_secret(&secret, true).unwrap().public_key_data();
    let mut pubkey = ECPoint::from(&parent_public_key);
    ECDSAKey::derive_child_public_key(&mut pubkey, &mut child_chain, &index_path, 0);
    assert_eq!(chain, child_chain, "the bip32 chains must match");
    assert_eq!(&public_key, pubkey.as_bytes(), "the public keys must match");
}

#[test]
fn test_ecdsa_private_derivation() {
    let manager = ChainsManager::new();
    let chain_type = ChainType::MainNet;
    let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(seed_phrase).unwrap();
    manager.mainnet.wallet_with_seed(seed.clone(), false, chain_type);
    let chain = manager.mainnet.try_read().unwrap();
    let wallet = chain.wallets.first().unwrap();
    let mut path = AuthenticationKeysDerivationPath::identity_ecdsa_keys_derivation_path_for_wallet(chain_type, true,wallet.unique_id_as_str().to_string(), Weak::new(), true);
    let key = path.generate_extended_public_key_from_seed(&seed);
    let index_path1 = IndexPath::index_path_with_indexes(vec![1, 5]);
    let index_path2 = IndexPath::index_path_with_indexes(vec![4, 6]);
    let private_key1 = path.private_key_at_index_path_from_seed(&index_path1, &seed).unwrap();
    let public_key1 = path.public_key_at_index_path(&index_path1).unwrap();
    let private_key2 = path.private_key_at_index_path_from_seed(&index_path2, &seed).unwrap();
    let public_key2 = path.public_key_at_index_path(&index_path2).unwrap();
    assert_eq!(private_key1.public_key_data().to_hex(), public_key1.public_key_data().to_hex(), "the public keys must match");
    assert_eq!(private_key2.public_key_data().to_hex(), public_key2.public_key_data().to_hex(), "the public keys must match");
    let private_keys = path.private_keys_at_index_paths(vec![index_path1, index_path2], &seed);
    let private_key1_from_multi_index = private_keys.get(0).unwrap();
    let private_key2_from_multi_index = private_keys.get(1).unwrap();
    assert_eq!(private_key1_from_multi_index.public_key_data().to_hex(), private_key1.public_key_data().to_hex(), "the public keys must match");
    assert_eq!(private_key2_from_multi_index.public_key_data().to_hex(), private_key2.public_key_data().to_hex(), "the public keys must match");
    assert_eq!(private_key1_from_multi_index.private_key_data().unwrap().to_hex(), private_key1.private_key_data().unwrap().to_hex(), "the private keys must match");
    assert_eq!(private_key2_from_multi_index.private_key_data().unwrap().to_hex(), private_key2.private_key_data().unwrap().to_hex(), "the private keys must match");
}

#[test]
fn test_256_bit_derivation() {
    let manager = ChainsManager::new();
    let chain_type = ChainType::MainNet;
    let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(seed_phrase).unwrap();
    manager.mainnet.wallet_with_seed(seed.clone(), false, chain_type);
    let wallet = manager.mainnet.read().unwrap().wallets.first().unwrap();
    let i = UInt512::bip32_seed_key(&seed.data);
    let mut secret = UInt256::from(&i.0[..32]);
    let mut chain = UInt256::from(&i.0[32..]);
    let mut child_chain = chain.clone();
    let parent_secret = ECDSAKey::key_with_secret(&secret, true).unwrap();
    let parent_public_key = parent_secret.public_key_data();
    let derivation = UInt256::from([5, 12, 15, 1337]);
    // let mut private_key_data = UInt512::from(secret, chain);
    let index_path = IndexPath::new_hardened(vec![derivation], vec![false]);
    ECDSAKey::derive_child_private_key(&mut secret, &mut chain, &index_path, 0);
    let public_key = ECDSAKey::key_with_secret(&secret, true).unwrap().public_key_data();
    let mut pubkey = ECPoint::from(&parent_public_key);
    ECDSAKey::derive_child_public_key(&mut pubkey, &mut child_chain, &index_path, 0);
    assert_eq!(chain, child_chain, "the bip32 chains must match");
    assert_eq!(&public_key, pubkey.as_bytes(), "the public keys must match");
    assert_eq!(derivation, UInt256::from_hex("05000000000000000c000000000000000f000000000000003905000000000000").unwrap(), "derivation must match the correct value");
    assert_eq!(public_key.to_hex(), "029d469d2a7070d6367afc099be3d0a8d6467ced43228b8ce3d1723f6f4f78cac7", "the public must match the correct value");
}
#[test]
fn test_dashpay_derivation() {
    let manager = ChainsManager::new();
    let chain_type = ChainType::MainNet;
    let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(seed_phrase).unwrap();
    manager.mainnet.wallet_with_seed(seed.clone(), false, chain_type);
    let mut chain = manager.mainnet.try_write().unwrap();
    let wallet = chain.wallets.first_mut().unwrap();
    let account = wallet.account_with_number_mut(0).unwrap();
    let path = account.master_contacts_derivation_path.as_mut().unwrap();
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let source_identity_unique_id = UInt256::sha256(&[0x01]);
    let destination_identity_unique_id = UInt256::sha256(&[0x02]);
    let mut incoming_path = IncomingFundsDerivationPath::contact_based_derivation_path_with_destination_identity_unique_id(destination_identity_unique_id, source_identity_unique_id, 0, chain_type, Weak::new());
    incoming_path.set_is_transient(true);
    incoming_path.set_wallet_unique_id(seed.unique_id.clone());
    let ext_pubkey_from_master_contact_path = incoming_path.base.generate_extended_public_key_from_parent_derivation_path(path, None).unwrap();
    let ext_pubkey_from_seed = incoming_path.generate_extended_public_key_from_seed(&seed).unwrap();
    assert_eq!(ext_pubkey_from_master_contact_path.extended_public_key_data().unwrap().to_hex(), ext_pubkey_from_seed.extended_public_key_data().unwrap().to_hex(), "The extended public keys should be the same");
    assert_eq!(ext_pubkey_from_master_contact_path.extended_public_key_data().unwrap().to_hex(), "351973adaa8073a0ac848c08ba1c6df9a14d3c52033febe9bf4c5b365546a163bac5c8180240b908657221ebdc8fde7cd3017531159a7c58b955db380964c929dc6a85ac86", "Incorrect value for extended public key");
    assert_eq!(incoming_path.address_at_index(0).unwrap(), "Xs8zNYNY5hT38KFb8tq8EbnPn7GCNaqr45", "First address should match expected value");
}

#[test]
fn test_base64_extended_public_key_size() {
    let base64_engine = GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::default());
    let manager = ChainsManager::new();
    let chain_type = ChainType::MainNet;
    let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(seed_phrase).unwrap();
    manager.mainnet.wallet_with_seed(seed.clone(), false, chain_type);
    let mut chain = manager.mainnet.try_write().unwrap();
    let wallet = chain.wallets.first_mut().unwrap();
    let account = wallet.account_with_number_mut(0).unwrap();
    let path = account.master_contacts_derivation_path.as_mut().unwrap();
    path.generate_extended_public_key_from_seed_no_store(&seed.data);
    let source_identity_unique_id = UInt256::sha256(&[0x01]);
    let destination_identity_unique_id = UInt256::sha256(&[0x02]);
    let mut incoming_path = IncomingFundsDerivationPath::contact_based_derivation_path_with_destination_identity_unique_id(destination_identity_unique_id, source_identity_unique_id, 0, chain_type, Weak::new());
    incoming_path.set_is_transient(true);
    incoming_path.set_wallet_unique_id(seed.unique_id.clone());
    let ext_pubkey_from_master_contact_path = incoming_path.base.generate_extended_public_key_from_parent_derivation_path(path, None).unwrap();
    let bob_seed = [10u8, 9, 8, 7, 6, 6, 7, 8, 9, 10];

    let bob_keypair_bls = BLSKey::key_with_seed_data(&bob_seed.to_vec(), true);
    let path_bls = AuthenticationKeysDerivationPath::identity_bls_keys_derivation_path_for_wallet(chain_type, true, wallet.unique_id_as_str().to_string(), Weak::new(), true);
    let private_key_bls = path_bls.private_key_at_index(0, &seed).unwrap();
    let mut pubkey_data_bls = ext_pubkey_from_master_contact_path.extended_public_key_data().unwrap();
    println!("pubkey_data_bls: {}", pubkey_data_bls.to_hex());
    <Vec<u8> as CryptoData<Key>>::encrypt_with_secret_key(&mut pubkey_data_bls, &private_key_bls, &Key::BLS(bob_keypair_bls));
    // pubkey_data_bls.encrypt_with_secret_key(&private_key_bls, &Key::BLS(bob_keypair_bls));
    println!("pubkey_data_bls (encrypted): {}", pubkey_data_bls.to_hex());
    // assert_eq!(base64_engine.encode(pubkey_data_bls).len(), 128, "The size of the base64 should be 128");
    assert_eq!(base64_engine.encode(pubkey_data_bls).len(), 92, "The size of the base64 should be 92");

    let bob_secret = UInt256::from_hex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140").unwrap();
    let bob_keypair_ecdsa = ECDSAKey::key_with_secret(&bob_secret, true).unwrap();
    let path_ecdsa = AuthenticationKeysDerivationPath::identity_ecdsa_keys_derivation_path_for_wallet(chain_type, true, wallet.unique_id_as_str().to_string(), Weak::new(), true);
    let private_key_ecdsa = path_ecdsa.private_key_at_index(0, &seed).unwrap();
    let mut pubkey_data_ecdsa = ext_pubkey_from_master_contact_path.extended_public_key_data().unwrap();
    println!("pubkey_data_ecdsa: {}", pubkey_data_ecdsa.to_hex());
    <Vec<u8> as CryptoData<Key>>::encrypt_with_secret_key(&mut pubkey_data_ecdsa, &private_key_ecdsa, &Key::ECDSA(bob_keypair_ecdsa));
    // pubkey_data_ecdsa.encrypt_with_secret_key(&private_key_ecdsa, &Key::ECDSA(bob_keypair_ecdsa));
    println!("pubkey_data_ecdsa (encrypted): {}", pubkey_data_ecdsa.to_hex());
    // assert_eq!(base64_engine.encode(pubkey_data_ecdsa).len(), 128, "The size of the base64 should be 128");
    assert_eq!(base64_engine.encode(pubkey_data_ecdsa).len(), 92, "The size of the base64 should be 128");
}
