use bls_signatures::{PrivateKey, G1Element, Scheme, BasicSchemeMPL, G2Element, LegacySchemeMPL};
use byte::BytesExt;
use hashes::hex::{FromHex, ToHex};
use hashes::{Hash, sha256d};
use secp256k1::rand::{Rng, thread_rng};
use crate::keys::{BLSKey, IKey};
use crate::crypto::{UInt256, UInt384, UInt768};
use crate::crypto::byte_util::Random;
use crate::models::OperatorPublicKey;

#[test]
pub fn test_bls_sign() {
    // In dash we use SHA256_2, however these test vectors from the BLS library use a single SHA256
    let seed1 = vec![1u8,2,3,4,5];
    let seed2 = vec![1u8,2,3,4,5,6];
    let keypair1 = BLSKey::key_with_seed_data(&seed1, true);
    let keypair2 = BLSKey::key_with_seed_data(&seed2, true);
    let message1: Vec<u8> = vec![7,8,9];
    let message2: Vec<u8> = vec![1,2,3];
    let message3: Vec<u8> = vec![1,2,3,4];
    let message4: Vec<u8> = vec![1,2];
    let fingerprint1 = keypair1.public_key_fingerprint();
    let fingerprint2 = keypair2.public_key_fingerprint();
    assert_eq!(fingerprint1, 0x26d53247, "Testing BLS private child public key fingerprint");
    assert_eq!(fingerprint2, 0x289bb56e, "Testing BLS private child public key fingerprint");
    let signature1 = keypair1.sign_data_single_sha256(&message1);
    assert_eq!(signature1.0.to_hex(), "93eb2e1cb5efcfb31f2c08b235e8203a67265bc6a13d9f0ab77727293b74a357ff0459ac210dc851fcb8a60cb7d393a419915cfcf83908ddbeac32039aaa3e8fea82efcb3ba4f740f20c76df5e97109b57370ae32d9b70d256a98942e5806065", "Testing BLS signing");
    assert_eq!(keypair1.seckey.0.to_hex(), "022fb42c08c12de3a6af053880199806532e79515f94e83461612101f9412f9e", "Testing BLS private key");
    let signature2 = keypair2.sign_data_single_sha256(&message1);
    assert_eq!(signature2.0.to_hex(), "975b5daa64b915be19b5ac6d47bc1c2fc832d2fb8ca3e95c4805d8216f95cf2bdbb36cc23645f52040e381550727db420b523b57d494959e0e8c0c6060c46cf173872897f14d43b2ac2aec52fc7b46c02c5699ff7a10beba24d3ced4e89c821e", "Testing BLS signing");
}

#[test]
fn test_bls_verify() {
    let seed1 = vec![1u8,2,3,4,5];
    let message1: Vec<u8> = vec![7, 8, 9];
    let mut key_pair1 = BLSKey::key_with_seed_data(&seed1, true);
    assert_eq!(key_pair1.public_key_data().to_hex(), "02a8d2aaa6a5e2e08d4b8d406aaf0121a2fc2088ed12431e6b0663028da9ac5922c9ea91cde7dd74b7d795580acc7a61");
    assert_eq!(key_pair1.private_key_data().unwrap().to_hex(), "022fb42c08c12de3a6af053880199806532e79515f94e83461612101f9412f9e");
    let signature1 = key_pair1.sign_data(&message1);
    assert_eq!(signature1.0.to_hex(), "023f5c750f402c69dab304e5042a7419722536a38d58ce46ba045be23e99d4f9ceeffbbc6796ebbdab6e9813c411c78f07167a3b76bef2262775a1e9f95ff1a80c5fa9fe8daa220d4d9da049a96e8932d5071aaf48fbff27a920bc4aa7511fd4");
    assert!(key_pair1.verify(&sha256d::Hash::hash(&message1).into_inner().to_vec(), &signature1.0.to_vec()), "Testing BLS signature verification");
}

#[test]
fn test_bls_multiplication() {
    let private_key_data = Vec::from_hex("46891c2cec49593c81921e473db7480029e0fc1eb933c6b93d81f5370eb19fbd").unwrap();
    let public_key_data = Vec::from_hex("0e2f9055c17eb13221d8b41833468ab49f7d4e874ddf4b217f5126392a608fd48ccab3510548f1da4f397c1ad4f8e01a").unwrap();
    let expected_data = UInt256::from_hex("03fd387c4d4c66ec9dcdb31ef0c08ad881090dcda13d4b2c9cbc5ef264ff4dc7").unwrap();
    println!("private_key: {:?}", private_key_data.as_slice());
    println!("public_key: {:?}", public_key_data.as_slice());
    println!("expected_data: {:?}", expected_data.0);
    let private_key = PrivateKey::from_bytes(&private_key_data, false).unwrap();
    let public_key = G1Element::from_bytes_legacy(&public_key_data).unwrap();
    let result = private_key * public_key;
    let result_serialize = result.unwrap().serialize_legacy().read_with::<UInt256>(&mut 0, byte::LE).unwrap();
    assert_eq!(result_serialize, expected_data);
}

#[test]
fn test_bls_from_bip32_short_seed() {
    let private_key = PrivateKey::from_bip32_seed(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    println!("{:?}", &*private_key.to_bytes());
    println!("{:?}", &*private_key.to_bytes().as_slice());
    assert_eq!(
        private_key.to_bytes().as_slice(),
        Vec::from_hex("46891c2cec49593c81921e473db7480029e0fc1eb933c6b93d81f5370eb19fbd").unwrap().as_slice(),
        "----");
}

#[test]
fn test_bls_from_bip32_long_seed() {
    let seed = Vec::from_hex("0102030405060708090a0102030405060708090a0102030405060708090a0102").unwrap();
    let private_key_test_data = Vec::from_hex("32439470cf06d276897d1b9069bdd6e4445390cd506985de0e1a1c88a76ff176").unwrap();
    println!("{:?}", seed);
    println!("{:?}", private_key_test_data);
    // let seed = [50, 67, 148, 112, 207, 6, 210, 118, 137, 125, 27, 144, 105, 189, 214, 228, 68, 83, 144, 205, 80, 105, 133, 222, 14, 26, 28, 136, 167, 111, 241, 118];
    // let secret =
    let private_key = PrivateKey::from_bip32_seed(&seed);
    println!("{:?}", &*private_key.serialize());
    println!("{:?}", &*private_key.serialize().as_slice());
    assert_eq!(
        private_key.serialize().as_slice(),
        private_key_test_data.as_slice(),
        "----");
}

#[test]
fn test_bls_fingerprint_from_bip32_seed() {
    let seed = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let key_pair = BLSKey::key_with_seed_data(&seed, true);
    let key_pair_fingerprint = key_pair.fingerprint();
    assert_eq!(key_pair_fingerprint, 0xddad59bb, "Testing BLS private child public key fingerprint");
    let seed2 = [1, 50, 6, 244, 24, 199, 1, 25];
    let key_pair2 = BLSKey::extended_private_key_with_seed_data(&seed2, true).unwrap();
    let key_pair2_fingerprint = key_pair2.fingerprint();
    assert_eq!(key_pair2_fingerprint, 0xa4700b27, "Testing BLS extended private child public key fingerprint");
}

#[test]
fn test_bls_signature_verify_secure_aggregated() {
    let public_keys = vec![
        "8f3a813aa68a07fca73c616ea60d0dfbc81667c24a8ac6e6d4c9a64c6d162d5738808c5eab7138742a3d17c814a8bf94",
        "92058ad273ac46e18e4f43a20b5bcfbabdcded712d80387eeabaf190d4351f45749db9a9d1bf4e13e4ae946a03ed4015",
        "08ff9920aa7391cf47e0a1a816ab4c67e037a5d448d2cf28b4d8c7c4008c459eadbe5134f7176804046521ec0b49341e",
        "8b817dcf0c4233d3c71ceae42db90a1b630f1f97285b4ffd265387a088a7d38400cd705ca090bd9c0f4619a225e16c73",
        "91008785993639ba13e4e20981c89ed9a64a0e561da60e7e286f25c397d6e0db06acdded783b247fe26f2f2ff6665184",
        "0936107afd59a0433113ee3d77ef0ed7bc48790f70959460fdcac663f7050b4e48179c68228fe15f91dd6c19c702d0c8",
        "05ee12ead9b2fcbcd20e028acb74226fd75ec271ad8daa431fb9e6fdeea0070aeffb080c21edca34385060a1c8c05bdd",
        "064cd7b508ad623a51d79c8557a667499383e8d723ec79792d08bfce5fb96a6a898f502b74ab79761fed79c652c081d9",
        "919ad9aa930fc2cdafed3db371eb52dede4d14c9d170d1a75714556da791bc6973761ac975163488f258b988eb19d487",
        "02ab52425100d319bc1b5e1382c4eba074f73f2ae94f6e1713ffd9a0f513b541f44d9a0879f48fdcaf3521ebd3b734e3",
        "8f7242bdba0921c2418d4e3be676e320c0ba9ea86b5d185dd1dc1e665587925086e8bd0de4473f9eba0f1487ddf81f86",
        "0a354bd6de479dce0864fced9cc8e3f7ee39f1f77a69b28d75745907a7e76e6565d6b5b12bc7cd64284d910650f8ccb1",
        "921958278157241233fe7e816d06c4bba25583a108507c691d3ee45e3a7231a5606c31161c1c32614f74deff608690d9",
        "944be5269df80a87677dd9c7f6202c58d7b8eeeff712b2581b96956e823f02b7095a0fa27d12f8e10a426fa666abe7d2",
        "81b1f0151edf35e001385496b0b18481d4293eb1218f8105be8068d7864c535825d134f70177922a1c64674c87e10829",
        "11d3f729e18d03589e5795565318007ec11675fbcd970ff72c6d8534f0a9e582f00d6254d897e5563e90286a5ab2197f",
        "81a5c8199317dc83bebfa4b00c3c50c3c3ce9e1af8271016c44821dcb3f4a8181a35c21e3914d765f5a4e2059ebd5bbb",
        "8451624e5fbcdcf1703e9c1e80cef6e07e648ce343952f3e30c82c17c64a934870b54f30249398532e4a3e74a1e07df4",
        "8e1fbec112bc165a30db65e2bfbde2459fad2a590bcdca6c2350355ab9920c7db88655ac6ac25ed190f8e58900cfa507",
        "114923b2ea69b786b8dd5ae2b95f726903735714890b923d42288f8a46d894009749f5acd47340de1a3a4d33d80a3258",
        "0e7e5a1f72524c3bab4d7990a27af8d4451a327109549d876ec522e22305cadf80c9ff0f74d1a200dbcdb1376afd34e5",
        "892bda25e986cfdce112814bff6bb7f01b5bba267f503902d006ed0c30c4c27b782bf3cdfdb761514fba52129e45f76f",
        "99dc46b3b77144740a64f5dc1ca597424ac998d2a00e1eea6b248a02a5a53f9db0e122e598ad93d83b046ba3105d2f1a",
        "8e433404d5169db60433f21db99edcce1afcb548d2b0414c9dbab698148aaaf8d91e1ee94a021404e5d8d3d644835659",
        "0c3cd2a62cf315fb5c34615d8fda0d032d88de74d8100e85c4c07bb636ab609b699e1d593506eb160d4adfcd9f86dad8",
        "0f396fa4c452d8c6eb1eb993bdea8af98f96c65bf0cc37ef5048c895af4e89aa8babaa95111157a0d0aaefe1809282a1",
        "8162cb75478d2328c6af409b3ba0f4f720cd30c340d0b608e62bfb7ed72015a35f1ff5225acbd97af2a33320fe3ede48",
        "8d630e590710227707903890ebb933e2c12cacd477f689a258cc2bcefc481ada7513b9e8a11878481f8aae36fc278fa8",
        "01da056d3b253e6660c98771aea644191640f179dee3674f0c720ae896ebc9a4614f707c6809ad8f33a7226abf65d549",
        "0bba5dc0e216fa128d8701d0ce4de39e2dc39f16a0a3ceeced7fcc89d17b65cee32362bc68b712bfc9cc5490c334c6e6",
        "86f9f8c4738f1e83450f785017b983c6036ddcb23d16ba08735c51c531347dc3aa5ee8471ad883d2b1dd0873f6e18a70",
        "860413b84c02b5bfd97f44a2737dc4bd20404614d74e63da02d3dd91fd211d7c5b4ffc9caa23b277b53b96ec50bd7ff7",
        "05e2e0ff4488026ff18e1700c8378f50e4b84b9222ee46d0898ba7debe7da7121f98edca635bd167345e7904ee08330c",
        "13e6578f575c681f159b019fbc7d425f6e9cee2a4bfc98bad504026418d351aec1f0e91a66de53ef8899ab6fb66a1bed",
        "12e53b9b0f93bdac4d25e78fb5610aa4a10d12906586b1e162598a31718af93d015162ed7bb1d21daab9aa85e164afd1",
        "183ad2fa4d5622e12ef083304461bcc046c41c2a24b2f1ef7b36e2fe8bc50f48dbef75cf51128bfa4e280ba724babf23",
        "90908933bd97769966d74a7a85fad9ce894ec6dd943b71678a2ec87a155a9a0a390707e64d384a6452fe478771262504",
        "1232235225905ae0f2f765dcc3908e2e40d241bf9783ee7e39831bf76b620e3c019fdb522900563dc06a0494b036c27e",
        "0933c9280553bdf898189d3dec95419262433defd4bef9e90611ffc05376e582fe41f57d4b0547852928da79171e29fa",
        "8f2d54ffb351acc9fb8ca90726b02320832dda589a83fae040611d96a0a6917a5fbac2841232e18312f675c6a5aee670",
        "9555f97d16e75a135d98ec2f52a8881a60c790673cad6d9f0ef0e52bca3b0607aaa19cfec7ef4512e85b7fc687d0f3e6",
        "84830a7f9af1b788df3060c089e3e7d6e242e94802dfb8d2eb46d69aa27276a860963c52b20f41cdb4791a71e58b4344",
        "892fc0e02bff6e41f119ddb6f7f0d475a2721f101a26830db026681383fbbfab0cfa488473b51f511edab979ed915b28",
        "8c8440a82f2fa19bcf1a1324de03db6beba690da39c79c7e09835728026c46a59475e2fae6d0fbe20c01a128e796aac8",
        "8614b12d8761ef8fffe132725a9a6b511abe7823df3c0022ab3e4dec221cf8ce2ab6589cd617779023a54056d87f997b",
        "89718b0bcc8233af8df3eab1f3d2003282506e6babe096eae072cb8a435431fb3ca0359ef7ee8bfb3fbe981debdf9c0f",
        "0abc9b9ee35465c024cf4c72ed60dcb600c8657e6deff6f4ad69400b5f3a9d5140bb7c09c5262cd1265c093a7cb6c184",
        "10f912e265e3865b0ca0e7a8514616f541d2526e493212d0e82218f2ec7abce09eeb0316d165cdf006dfb596b37380d9",
        "8624a671b7eb6111e53adf55806ac01d6c0af6da23fe3e964650ed39b017585291cd6e2b3ff20a1f658aa26b4836abf6",
        "069fdc47b17e21a2c12eb27e81ff4c011f8088b3525cb1e6140a6f7db38123232ede01ee4fb2b7be143b756533a77ef6",
        "1053fe7f087d571a864a5a7408002f0ad786b33f06db9b719f39ed37af60270eb0c9e494833b6d2d1f029ec2700d3e6d",
        "954b5017998fe8a16d3946ed13ffa255c546a1dbd478e5bb3a2657de4d331a6abda1587ee83a8c6954bcad8ee43bb16c",
        "0c08500f384056485306bc8ff98a26ede2d20248ea1f7ccbd3ddc8b29a0e46a8fcda6a02d7a12b6ae94207a441411477",
        "8e2c3b1b98e45c78c9ccd7934064da30b18f6891417a7915d8dd7ee3dc5be76baa6e164e3dc0ae7185e9a3b449bfe813",
        "17ac04dcbe4572333decb848d4dcea1c2e5edf24a1e774aa1c1c6f31dbc3261883ad27cacd2efdd2ab91b24a77390b3f",
        "0eda3c087f9a593efe4c8fa7fd4ce02c587952b1bc20a49b2d21d573213c4f47a6db3494b1a33a0749518ba3bc0002d0",
        "022a15f6c1f3af9376cadbf2e99684de157ddcdd0966fac9fddb9772867213b867994bdcb55c8ea30e41b19c385f9fe4"
    ];
    let members_signature = UInt768::from_hex("052f62455ad81786528a2c7b7ab4c22f812982ed99c0799e6cbf9a719a76e9cff2eaca9aefd41f29922c2f85e3c3d70a1100b35bc0d7d25bd54291d99234bf556a5649e8cccf4fddb040ebaca5fa401b0ec409cbd285f6c58a8dc17b521b2093").unwrap();
    let commitment_hash = UInt256::from_hex("656e3b2e895b155da40860ad4c09d48204d0847f1eb20bd1ebbe9416bfbd7961").unwrap();
    let operator_keys = public_keys.iter()
        .map(|s| OperatorPublicKey { data: UInt384::from_hex(s).unwrap(), version: 1})
        .collect::<Vec<_>>();
    assert!(BLSKey::verify_secure_aggregated(commitment_hash, members_signature, operator_keys, true));
}

fn test_bls_verify_random_signature_using_scheme<S: Scheme>(schema: S) {
    let len: usize = thread_rng().gen_range(0..30);
    let mut vec_pks = Vec::with_capacity(len);
    let mut vec_sigs = Vec::with_capacity(len);
    let hash = UInt256::random().0;
    for _i in 0..len {
        let private_key = PrivateKey::from_bip32_seed(&UInt256::random().0);
        let signature = schema.sign(&private_key, &hash);
        let public_key = private_key.g1_element().unwrap();
        vec_sigs.push(signature);
        vec_pks.push( public_key);
    }
    let signature = schema.aggregate_sigs(vec_sigs.iter().collect::<Vec<&G2Element>>());
    let public_key = schema.aggregate_public_keys(vec_pks.iter().collect::<Vec<&G1Element>>());
    assert!(schema.verify(&public_key, &hash, &signature));
}

#[test]
fn test_bls_basic_signature_verify_secure_aggregated() {
    test_bls_verify_random_signature_using_scheme(LegacySchemeMPL::new());
    test_bls_verify_random_signature_using_scheme(BasicSchemeMPL::new());
}

#[test]
fn test_bls_llmq_50_60() {
    // LLMQ::verify at 871584: Llmqtype50_60
    let commitment_hash = UInt256::from_hex("0f602593b3ea2d71d14728edce3e92a29d800e6745baf7de5ac3c3a2a2c627f5").unwrap();
    let signature = UInt768::from_hex("a97b10b1b24fd6aa0f958f73dcbab59e4bfca46647189cf0e186e25e355d74752a3069f6d030118860717068c611fcf513c6b1aafb75c1010bd76085f43e32401a1e58ba7f20acc99aefc7c00f9b04ce346767804095b7e014a68192614da077").unwrap();
    let operator_keys = vec![
        OperatorPublicKey { data: UInt384::from_hex("a2fea620ee07107d6611b7ccf5726e0e8247a131a3f129eb2b3195fe0fc1a91044af42a839915161e2105944252c59aa").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("890f1ca955443740346b5b4b0bfb8251f040074b5a2feb77e54add831bf34aaf1d84207691f6f5aa5e702152a496fadc").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("9130b42d6eb505e811dfb18ff87c4bcacde56b76a7d47a8db88ca26e75f5c2eebdd767d440f375784f9d1f127f57c977").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("94e81203caa5c0cc305b5e0f3ae7a388b974a629358e4e83e50a25b2c2a387e3d114c7c82e2b23c25b65585220e63c99").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("99c6bb8f02fe03ebfe9f3c8900dd764e6f379ca1061b8dbd8ce6b6b139489c9083a84ee60f2aca4ae114797abc07d945").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("ac290b31d2e878c2d7235efb0c61f423aa37742a31318e61f8bb0bd6c110a892dc244512fec12a8b0fe7cbb08e12be28").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("a42c53c3aa11ae4b985a52ae6a3170bdb58f88ec04c62013f9322bd5fda4417939836b6f41741dd864c348103a1155d3").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("96ca29d03ef4897a22fe467bb58c52448c63bb29534502305e8ff142ac03907fae0851ff2528e4878ef51bfa3d5a1f22").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("852057284a7a9dbccb97fbaea3425104901dc661b69294a55c7ca800ed18d37df7ccc02367b5d6836ee4f6b052249a1d").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("97596d7a72b65531fffd5f610752422d6e286c975f30d026092f7900f8015073bd6f6d1b85dd3981814c093910e7dac6").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b05caab51ff07a2f8d69972fd6ec09f6f9893cf6dfc49775f5a2db2ea7a8a525bbaf4e7e369d06590f6f2e8e4658d4dc").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8aeb5c2757211202b3afd2033ec1b4ef2dfe376ba5c6c07b45e6a7460afa4086423c4a704eb9a781514fbc513e190a62").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8c9bf080a96d13b356b01e734618a77225f03b3e92684f252ccbd313764a9fd9247bde6b00d92f6b5669043e77860453").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("94e199beb2a2a59166a12a851ef158928bc5efc25b39eb78b3a428b25384609d8c03548a94e77c0c941c90c68a4187d8").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b2823797ad456d53ce1e6bde84e8a19164ff88a73ccd242ec48d9c6a479f2a049e214c7e8ec2243b7ea74ca6144ab2c5").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8196970badc74d068ec1226ffd4a656313decef59d792237a32e6ff56cd4e43030c436025831a4a3d0306a616f033810").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("adeed4d18add0ef5a7dff743a206786ab2dcc1b4aff679a61577dea99b62fb24dd56e3fe7ff65fa0be964dc5d7967c3e").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("95a577f51dc6fd7fa4621f0a4601e48fd65418a89c2af2afef725fb4f053a8ee5841cd3fdae39ebdf5a202e0c4deca23").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("82ce863d0843ca66b4a64d94c0d84ec15980ea04e4444ac4d4188f38cc0da4d6d2360b8a2046725b682862255af6a48c").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("a26fc7f30c49215b98d5cb47a350f888a306c52fa42c77e765b55288e622f03859273cae7e1cac99e67f7a9a96a6aa2c").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b37befcc17d16d4154ea8bbe82e9bd52e2ecd825dd9a43f58730d594d87cceebcb41e11461319fd71bfc08d0a0545200").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("84c0ca8535c114f8f1b369f339b2653e7126610f5170b223970f4e63ad7b55ea2f61a08e263b51fb03f6940d655690f9").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b3ea90ebcf0d8e332e37e5ac3c676653bb1203e8db7604bb0ac64a9b655b553de514e9bff5eeb86bb3ef9178375392f9").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("875b907b6d6c12aa111da0e102186b9d06f4e065969b60732207f18c2c5d0deb8ecba47cb4c0929647db0e2fae6f08ca").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("a42d732a03847819b1e2675ac48b9af4a1c92b310ecacc42c428ff902099cc47d08ecd4616da55d185463855aee99f79").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("ac9c5c77fe321ff0a115d1ba5bf7462063ef21a82ba796415f4ee538bf9e8a6a49707530c72cbb6b60026c46ff1b9443").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8e9855b2ee991f988e446b60dcd637f33a782baf1e755785ca058f0398133bf3a95e4e77d4168c13c47d7e3fb1e3ecfc").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("93146b3252f408f1cffc875b12b61f56c1ae02113b24c0b5aaedcda4a9b509332c8c4587450074f3e0906aaf3ceca754").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b9d1a6a4f7f817d2e004134d96fa0c831433e9c649726ba8567f447d1b2394209bc1ef184a93c707054fb6816790de30").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b888fe437eab3af5a7fee4e0164705458a6fda97ae390d69721a5f1d3830ec330fb53c6a29588f1f94f69adcff04ca09").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("90d4c6a24d00d70fe961b77d58eff318bb6cd00c122bcfa20f92d65d03b9fd3afa5a0effc90810103a53d53ab155f764").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8fa5377eb256323aace31b45c3e48ea110404b053cb80e8043bd1e44de1705130548e4ab28738816251ea57a7fc10324").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b2cafe1870e043973b2f1fded8de3d5a66dac5ade46aa0995157077efee92d852857bc7f03ed69c92723a58f8bd2926e").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("aa0ede82d78a0a8f4c2332d431c7be496c3aa09349ed3b2db30f7eb7dcc7b6e580a9d71f7d76bdaca1b3670e0cf4cd3c").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("97f78abcee6d2ed68bf2c82afbf56ef9af67313e2eb655ea5178850907cb3057cae0bb5a1d09f161057bf62f9d4890c6").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("80ea87eef15f38c1a844d77348e687794c601277011c933026cdfdb649524632b055feea3539abc48472cb447d281d65").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8ad4f577d067630f6fd15f4d2aefdb9456d648b71cb7253d47511acc81dd5ddb69a03c848322aa11e5242f66afde5a2a").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("92730062f122f937b29f69536db3ad36980b88004eadc2ca341425d432723d67e53a4f55786c54017d77c1bd1df6b310").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("81f24418ec73b09b00514dba8fb18d6d8af1dc2ff93d594bf987911f3b98d659eb43286cd450b7e1ee5978b361660d73").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b0051db915bd86bd938746c14440b11ee3b2801cbc6d6c1c912e8b41ea5eb1d8f852abf220ae91ecdb6da094846c1ba8").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("80a2e66a810493a91b5ed1a8ef8ac4be41543598f5b4765a6f5d6339078ab88030817dc9c9bdb60c7c7a02d7787d6f2e").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("95f9da603c572257802a689964ca8f4d96f9b94f33ab75968c9cb6c730a28d50b7bb72ac2cfceee6ab0755ead9cb53cd").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("90e3caa7ae519505a6e1f3b56d3a99865f70e48f772ac431c3964a33cce7fe1e736d43ec3343ad843faaaa2b2bb3a921").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b7d5f022c3b6c314bde5171ead1616e4c27f0e9a48a9a9dc3a7227a62d42213b93c8a4c32af18bd8ff931b7732782e09").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b32f6fc90c9dcaacdf9d836a2a7e60d090fe5e55b0b02f5a4f608a4b8235ba5aa7abc4e05f9387d1d942adc57c87f5b7").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("911a30e0a5f2f5135dcc5f09498e4ba5de22c7680f396599f7f29b91ac569c3d4336bc157443cf8c06682bfb5abb2271").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b4a637afe3810d73e3402b5d6a398e45222ba846a339f1c3570aa8e3f7f5b9d7acef08ac234cce4f706671498330a599").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8dc75e865b89e96560b38fae96f1d0a5438795778e68b705a506046245ca5dbbedb09e2379eea4c9bde0d0fd4fe05080").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("882cbd9118474316f40b800e43f94a121928f256fd340098ff0ad81a902c4326dda4b42737d52739482f2baa80c487cc").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("92fa57a5676925e8dfe3b340df2132f5844ad9f89594b04efa28fb4fb884fe21f411fa49120ed7a60ce9381a54232a10").unwrap(), version: 2 }
    ];
    let use_legacy = false;

    let all_commitment_aggregated_signature_validated = BLSKey::verify_secure_aggregated(
        commitment_hash,
        signature,
        operator_keys,
        use_legacy);
    assert!(all_commitment_aggregated_signature_validated);
}

