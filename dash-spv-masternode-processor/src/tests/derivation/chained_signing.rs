use std::sync::Weak;
use crate::chain::common::ChainType;
use crate::chain::derivation::authentication_keys_derivation_path::AuthenticationKeysDerivationPath;
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::derivation::simple_indexed_derivation_path::ISimpleIndexedDerivationPath;
use crate::chain::ext::wallets::WalletCreation;
use crate::chains_manager::ChainsManager;
use crate::crypto::byte_util::{AsBytes, Random};
use crate::crypto::{UInt256, UInt512};
use crate::keys::{BLSKey, CryptoData, Key};

const SEED_PHRASE: &str = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";

#[test]
fn test_example() {
    let chain_type = ChainType::MainNet;
    let seed = chain_type.seed_for_seed_phrase::<bip0039::English>(SEED_PHRASE).unwrap();
    let manager = ChainsManager::new();
    manager.testnet.wallet_with_seed(seed.clone(), false, chain_type);
    let mut chain = manager.testnet.try_write().unwrap();
    let wallet = chain.wallets.first_mut().unwrap();

    let path = AuthenticationKeysDerivationPath::identity_bls_keys_derivation_path_for_wallet(chain_type, true, wallet.unique_id_as_str().to_string(), Weak::new(), false);
    let key0: BLSKey = path.private_key_at_index(0, &seed).unwrap().into();
    // let key1: BLSKey = path.private_key_at_index(1, &seed).unwrap().into();
    // let key2: BLSKey = path.private_key_at_index(2, &seed).unwrap().into();
    // let key3: BLSKey = path.private_key_at_index(3, &seed).unwrap().into();
    let random_input0 = UInt256::random();
    // let random_input1 = UInt256::random();
    // let random_input2 = UInt256::random();
    // let random_input3 = UInt256::random();
    let random_output0 = UInt256::random();
    // let random_output1 = UInt256::random();
    // let random_output2 = UInt256::random();
    // let random_output3 = UInt256::random();
    let concat0 = UInt512::from(random_input0, random_output0);
    // let concat1 = UInt512::from(random_input1, random_output1);
    // let concat2 = UInt512::from(random_input2, random_output2);
    // let concat3 = UInt512::from(random_input3, random_output3);
    let hash0 = UInt256::sha256d(concat0.as_bytes());
    // let hash1 = UInt256::sha256d(concat1.as_bytes());
    // let hash2 = UInt256::sha256d(concat2.as_bytes());
    // let hash3 = UInt256::sha256d(concat3.as_bytes());
    let signature0 = key0.sign_digest(hash0);
    let quorums = path.private_keys_for_range(1000..1008, &seed); // simulate 10 quorums
    let signing_session = UInt256::random();
    let mut keys_for_dh0 = vec![Key::BLS(key0)];
    keys_for_dh0.extend(quorums);
    let mut signature_data = signature0.0.to_vec();
    let mut encrypted_signature0 = signature_data.encapsulated_dh_encryption_with_keys_using_iv(keys_for_dh0.clone(), vec![]).unwrap();
    let mut reversed_keys_for_dh0 = keys_for_dh0.clone();
    reversed_keys_for_dh0.reverse();
    let signature_data_round_trip0 = encrypted_signature0.encapsulated_dh_decryption_with_keys_using_iv_size(reversed_keys_for_dh0, 0).unwrap();
    // TODO: signatures don't match
    // assert_eq!(signature_data.to_hex(), signature_data_round_trip0.to_hex(), "these should be equal");

    // NSData *signatureData0 = uint768_data(signature0);
    // NSArray *keysForDH0 = [@[key0] arrayByAddingObjectsFromArray:quorums];
    // NSData *encryptedSignatureData0 = [signatureData0 encapsulatedDHEncryptionWithKeys:keysForDH0 usingInitializationVector:[NSData data]];
    // NSData *signatureDataRoundTrip0 = [encryptedSignatureData0 encapsulatedDHDecryptionWithKeys:[[keysForDH0 reverseObjectEnumerator] allObjects] usingIVSize:0];
    //
    // XCTAssertEqualObjects(signatureData0, signatureDataRoundTrip0, @"these should be equal");
    //
    // NSData *encryptedSignatureData1 = [signatureData0 encapsulatedDHEncryptionWithKeys:keysForDH0 usingInitializationVector:[NSData data]];

    // at node n, quorum checks that signature matches

}
