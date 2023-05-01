use std::collections::BTreeMap;
use bls_signatures::{BasicSchemeMPL, G1Element, G2Element, Scheme};
use hashes::hex::FromHex;
use crate::chain::common::{ChainType, LLMQType};
use crate::common::{LLMQVersion, SocketAddress};
use crate::common::MasternodeType::{HighPerformance, Regular};
use crate::consensus::encode::VarInt;
use crate::crypto::{UInt128, UInt160, UInt256, UInt384, UInt768};
use crate::ffi::from::FromFFI;
use crate::keys::BLSKey;
use crate::lib_tests::tests::{assert_diff_result, create_default_context, message_from_file, process_mnlistdiff, process_qrinfo, register_cache, register_default_processor, register_logger};
use crate::models::{LLMQEntry, MasternodeEntry, OperatorPublicKey};

// #[test]
fn test_core19rc10() {
    // 85.209.243.24 (/Dash Core:18.2.1/ protocol 70227)
    let cache = register_cache();
    let context = &mut create_default_context(ChainType::TestNet, false, cache);
    let processor = register_default_processor();
    let result = process_mnlistdiff(message_from_file("testnet/MNT530000.dat"), processor, context, 70219, false, true);
    assert_diff_result(context, result);
    unsafe {
        context.cache.mn_lists.insert(UInt256(*result.block_hash), (*result.masternode_list).decode());
    }
    // let result = process_mnlistdiff(message_from_file("MNL_530000_867700.dat".to_string()), processor, context, 70227);
    let result = process_mnlistdiff(message_from_file("MNL_530000_867700.dat"), processor, context, 70227, false, true);
    // let result = process_mnlistdiff(message_from_file("MNL_530000_867700.dat".to_string()), processor, context, 70227);
    assert_diff_result(context, result);
}

// #[test]
fn test_core19_70224() {
    let cache = register_cache();
    let context = &mut create_default_context(ChainType::TestNet, false, cache);
    let processor = register_default_processor();
    let result = process_mnlistdiff(message_from_file("testnet/MNT530000.dat"), processor, context, 70219, false, true);
    assert_diff_result(context, result);
    unsafe {
        context.cache.mn_lists.insert(UInt256(*result.block_hash), (*result.masternode_list).decode());
    }
    let result = process_mnlistdiff(message_from_file("MNL_530000_868301.dat"), processor, context, 70224, false, true);
    // let result = process_mnlistdiff(message_from_file("MNL_530000_868301.dat"), processor, context, 70227);
    assert_diff_result(context, result);
}

// #[test]
fn test_core19_70227() {
    let cache = register_cache();
    let context = &mut create_default_context(ChainType::TestNet, false, cache);
    let processor = register_default_processor();
    let result = process_mnlistdiff(message_from_file("testnet/MNT530000.dat"), processor, context, 70219, false, true);
    // assert_diff_result(context, result);
    unsafe {
        let list = (*result.masternode_list).decode();
        context.cache.mn_lists.insert(UInt256(*result.block_hash), list);
    }
    let result = process_mnlistdiff(message_from_file("testnet/MNL_530000_868321.dat"), processor, context, 70227, false, true);
    assert_diff_result(context, result);
}

// #[test]
fn test_mnlistdiff_and_qrinfo_core19() {
    register_logger();
    let version = 70227;
    let cache = register_cache();
    let context = &mut create_default_context(ChainType::TestNet, false, cache);
    let processor = register_default_processor();
    let diffs = vec![
        "MNL_0_868888.dat",
        "MNL_0_869464.dat",
        "MNL_0_869760.dat",
        "MNL_868888_869176.dat",
        "MNL_869176_869464.dat",
        "MNL_869464_869752.dat",
        "MNL_869752_869760.dat",
        "MNL_869760_869761.dat",
        "MNL_869761_869762.dat",
        "MNL_869762_869763.dat",
        "MNL_869763_869764.dat",
        "MNL_869764_869765.dat",
        "MNL_869765_869766.dat",
        "MNL_869766_869767.dat",
        "MNL_869767_869768.dat",
        "MNL_869768_869769.dat",
        "MNL_869769_869770.dat",
        "MNL_869770_869771.dat",
        "MNL_869771_869772.dat",
        "MNL_869772_869773.dat",
        "MNL_869773_869774.dat",
        "MNL_869774_869775.dat",
        "MNL_869775_869776.dat",
        "MNL_869776_869777.dat",
        "MNL_869777_869778.dat",
        "MNL_869778_869779.dat",
        "MNL_869779_869780.dat",
        "MNL_869780_869781.dat",
        "MNL_869781_869782.dat",
        "MNL_869782_869783.dat",
        "MNL_869783_869784.dat",
        "MNL_869784_869785.dat",
        "MNL_869785_869786.dat",
        "MNL_869786_869787.dat",
        "MNL_869787_869788.dat",
        "MNL_869788_869789.dat",
        "MNL_869789_869790.dat",
        "MNL_869790_869791.dat",
    ].iter().for_each(|name| {
        let result = process_mnlistdiff(message_from_file(name), processor, context, version, false, true);
        assert_diff_result(context, result);
    });
    context.is_dip_0024 = true;
    let result = process_qrinfo(message_from_file("QRINFO_0_870235.dat"), processor, context, version, false, true);
    assert_diff_result(context, unsafe { *result.result_at_h_4c });
    assert_diff_result(context, unsafe { *result.result_at_h_3c });
    assert_diff_result(context, unsafe { *result.result_at_h_2c });
    assert_diff_result(context, unsafe { *result.result_at_h_c });
    //assert_diff_result(context, unsafe { *result.result_at_h });
    assert_diff_result(context, unsafe { *result.result_at_tip });
}

// #[test]
fn test_qrinfo_core19() {
    register_logger();
    let cache = register_cache();
    let context = &mut create_default_context(ChainType::TestNet, true, cache);
    let processor = register_default_processor();
    let result = process_qrinfo(message_from_file("QRINFO_0_870235.dat"), processor, context, 70227, false, true);
    assert_diff_result(context, unsafe { *result.result_at_h_4c });
    assert_diff_result(context, unsafe { *result.result_at_h_3c });
    assert_diff_result(context, unsafe { *result.result_at_h_2c });
    assert_diff_result(context, unsafe { *result.result_at_h_c });
    assert_diff_result(context, unsafe { *result.result_at_h });
    assert_diff_result(context, unsafe { *result.result_at_tip });

}

// #[test]
fn test_verify_secure() {
    let commitment_hash = UInt256::from_hex("ccabc0460e2dd8ea83966cf675175e6f37bfa20c162ae773b89827664e8dc8e5").unwrap();
    let signature = UInt768::from_hex("88bf53b96714f2d8901922fd8e3e39400c0d85f9ad218ad56344ec5d0dd04b6ab04b668d8ebb814357c716a59363b9f816c5d682c4c8508a44d25cd49fe4248ea216a9410a54fbe6ba931432f2fa88c4b01cf07818331c4fae6e41534feffa3e").unwrap();
    let keys = vec![
        OperatorPublicKey { data: UInt384::from_hex("b25d20af1a6d0ccd3890f0aead4a05a59be22e005b6d732f855311915b351a9153b2c83d84611b2c9958f806c93f7b5f").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("af9cd8567923fea3f6e6bbf5e1b3a76bf772f6a3c72b41be15c257af50533b32cc3923cebdeda9fce7a6bc9659123d53").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("818a6d23ae53d6231f7dd73a058f125340e92f6e97897f017d9d9d4e6671bbd92241170dfcdd5a4ab8ef47ef12ddcad5").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("97a49cea05ca2e18f74af110c5ab52c89a43ced4e056a8af7ca8973401494bdaba26d1c56b46b018091d0dd64f244750").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("95a577f51dc6fd7fa4621f0a4601e48fd65418a89c2af2afef725fb4f053a8ee5841cd3fdae39ebdf5a202e0c4deca23").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b5df04cdd8a9ffdad72b2ffaa0bf752b6fe9adc70c59834cca826a7a0f7264e4cbefd351772e30d527bc5aa9019c4ca5").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("aaba308a8db9e1c6c545f3fdfff00880a973f0685207688d3c0ee4546d732ec16a81fa1aa952fb3190ff5c6febcd1a78").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8ea46d70601eb45319ab495e2462f981debc8316df2bb1a679ae3525c7f517e535b69a02052844374c887a9312a47984").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("81ad0f9be5a88ae62ff54fe938dfceea71be03bd4c6a7aebf75896e8d495d310acc4146aa4820bc0e5f5b06579dedea5").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("92fa57a5676925e8dfe3b340df2132f5844ad9f89594b04efa28fb4fb884fe21f411fa49120ed7a60ce9381a54232a10").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("a9cac656247969ce388f3cd37079f4b5caa9ca1a523c12cefe9ad9e8f74f859c43df6e9448975f307dfd3632b6e495e2").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("91a043738454ea2f87aa38e88912341c806b68dcf4e472fe94a425ed79b4dc15b2bfdf6df0050dba41575acc666472cf").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8feade899f17584ccf85f2d86ccd7ced638d919b2d70d1ec90d3c062d5b5fd3b56b25376a08f3e01611cc5ea67e1f05e").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("ac9c5c77fe321ff0a115d1ba5bf7462063ef21a82ba796415f4ee538bf9e8a6a49707530c72cbb6b60026c46ff1b9443").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8935576848f6ab7e27fff34b671953672012352e36f5147181926b8bbc9e8b43b98458704666df25d36f37d41eb7c694").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("aa0ede82d78a0a8f4c2332d431c7be496c3aa09349ed3b2db30f7eb7dcc7b6e580a9d71f7d76bdaca1b3670e0cf4cd3c").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("81ad0f9be5a88ae62ff54fe938dfceea71be03bd4c6a7aebf75896e8d495d310acc4146aa4820bc0e5f5b06579dedea5").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("90e3caa7ae519505a6e1f3b56d3a99865f70e48f772ac431c3964a33cce7fe1e736d43ec3343ad843faaaa2b2bb3a921").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("838ea498b8c554bdc1f0ce0c107d8d23d27e40b45e6df56793cab951722dd69a958dc14798ae542cf025802f4b84a3f6").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("90d4c6a24d00d70fe961b77d58eff318bb6cd00c122bcfa20f92d65d03b9fd3afa5a0effc90810103a53d53ab155f764").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("888905cc3f99e76b3a1abf714a55978d9930c2abdc77a21bd809e452e8c47c35d38e318ec3118e1944cf1a4a8df907c1").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("afc31972124bb559aecd56dfb361048ab3f5203624c6436b1676b8d440bee777d83b937febacb2d3a651df9dbd20503e").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b6175b59aba8cc0477d4fff78bd90294f31ebd385c39bc254c7995a5dd3ccb8dc1d8869e247bf63bef8ec79317f479a9").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("ac3026b3e3023db1db9ec8e3b7678761820a2a6e96e7a5d9a39b1894170f9cea7765d3d131d60fa9d17492ba560fb1f9").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b6e979f20241cbb73de7451779e8e059d9cb75a74b72ea6862d7ac703dc2ac07d86cec39b6e8923b55fd54dbc6177c3a").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("9502bb884b3437d65c0e025e49fb00ff6ea9f55d5bcdc36330b46c8bd18be9126b7a6d7f35f558ef8040f2c2284500a5").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b44cee83a79fa151527e527f3f4f5ba022e73ae8b0d913c4185a45c2a129aef935a585a7a725edcb36ece72a95758688").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b928fa4e127214ccb2b5de1660b5e371d2f3c9845077bc3900fc6aabe82ddd2e61530be3765cea15752e30fc761ab730").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8406e459bfd155c81f9103a1fe076f1261ee7513275d744c133c5d5dfa956b1449f173bd110bbc03673f376593f32a27").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("97596d7a72b65531fffd5f610752422d6e286c975f30d026092f7900f8015073bd6f6d1b85dd3981814c093910e7dac6").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("9099dcddc6560d1039b0edb91bd700e5deae0cba43163fa289a80c2bd22335b5b0e7a1fb8f5494c0e6360e73a12fe0a8").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("a80f24b5e040dcbf86c3f468dd28bf45d9e41fbcd127fad56669d9afe358dcdc26e42f0f8b19997b1741dbb99c553aa6").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b0051db915bd86bd938746c14440b11ee3b2801cbc6d6c1c912e8b41ea5eb1d8f852abf220ae91ecdb6da094846c1ba8").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8196970badc74d068ec1226ffd4a656313decef59d792237a32e6ff56cd4e43030c436025831a4a3d0306a616f033810").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("852057284a7a9dbccb97fbaea3425104901dc661b69294a55c7ca800ed18d37df7ccc02367b5d6836ee4f6b052249a1d").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8d4b5e1f48d7a77746676f09e2b995389d0f1c18601a6f909a4b542fccce87d9f5f30695d078a9181e142602d2e93f8f").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("9826a6cfb349fd60cb7ff72efa5c4f8249eab0a0274f07e2a3d52b16898b711f43f9c6171ebba6e969ac03c6554c24e6").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8dc75e865b89e96560b38fae96f1d0a5438795778e68b705a506046245ca5dbbedb09e2379eea4c9bde0d0fd4fe05080").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("aea71272ac9a9c891f0987a75e2200a44fc063bca92892c0a174cff4c0a524935e0b870bd091329836e43ca7d7c87e7f").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("93146b3252f408f1cffc875b12b61f56c1ae02113b24c0b5aaedcda4a9b509332c8c4587450074f3e0906aaf3ceca754").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("93fe96f243c4ded9e65467182848e46285b5db0097b5b74be93f590f3d7eb0880c89df70f8a7756aa96a242692cb685f").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b6ee48c7a71a9d8e0813e68ca09846245fa155285f24a62b0ce9cb0102b1994ec58af8ba2a01c09363bdcc395d41f3df").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b888fe437eab3af5a7fee4e0164705458a6fda97ae390d69721a5f1d3830ec330fb53c6a29588f1f94f69adcff04ca09").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("890f1ca955443740346b5b4b0bfb8251f040074b5a2feb77e54add831bf34aaf1d84207691f6f5aa5e702152a496fadc").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8160877a911d8bb7d1e75e2320e98cc3233c1f6972cb642424bfcec7c182c56d2c0ebb59e45f788f4d5dbfa2ebff3e3a").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("924aaa5688cb7220be4600211257ae054554583ad9233e8ca0d58abafe317129dcca9e34a1b9bbfa175b88d9fb31b55e").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("911a30e0a5f2f5135dcc5f09498e4ba5de22c7680f396599f7f29b91ac569c3d4336bc157443cf8c06682bfb5abb2271").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("8d9ee7b7e66124c4b047e1f93aed5a764ed7384292737ea17f3a7e429ce3f24d602d54b97f72d181b6f093da9b3ad3f5").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("af70ff352844250e267b31c0ddb83dffd4cac43532194bd47cbabf410ca29fa7f1ecec08c8fde8c0d13910e903016d5a").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("99b9f0fbeea3822cdc5b3654dea52103b3d9d5f01db4201955ea3689074d37da4711d8f313d4b5458eef3395aa75bfc7").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b6693296894820bdc3c0ae76f357e544847f10a68f0046f53745370dbe861d57e194ddaf7ff7d5e73cc3f240515c448e").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b8a2161c64bfdc7d621df51de569911a219f718bad4d6058dcca9bddf6696d43ddc4c1e3cf91640c93f820e5680efac3").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("918081d1c248d74a0737f36e5bd40aa71b512c6be6f68e3664723849ac47a62fc743c4dc7234694bda1b7701f33d2e81").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b3ea90ebcf0d8e332e37e5ac3c676653bb1203e8db7604bb0ac64a9b655b553de514e9bff5eeb86bb3ef9178375392f9").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("b291debc5e6c56a9e1a9b77cb980115c36a4d3d584826e62fc4b6ad7834cfc21e7c80226d46e90f4fda7771b45111526").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("adfa6ff26fcf77d816b22543003500c3b32bbf888d65eebb6086046f40a13ec537d14e09c3607e6cdce366a6a56bb68a").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("87f818e5c2330ac4e7f0ef820f337addf8ab28b07c9d451304d807feda1d764c7074bccbbd941284b0d0276a96cf5e7f").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("893ac10a0d49377fca566a10881e579c263ce761157165f9a34c18304edc7b6c0c1fe9101a7338c89308510df47bc5aa").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("94eabcb82f2b0b9cda8eaa3cecd39f0058b418cb7a25795f597a811895bfcc23643bb25ae8432a52804dfb53575b649e").unwrap(), version: 2 },
        OperatorPublicKey { data: UInt384::from_hex("94e199beb2a2a59166a12a851ef158928bc5efc25b39eb78b3a428b25384609d8c03548a94e77c0c941c90c68a4187d8").unwrap(), version: 2 },
    ];



    assert!(BLSKey::verify_secure_aggregated(commitment_hash, signature, keys, false));
}

// #[test]
fn test_verify_secure2() {
    let commitment_hash = UInt256::from_hex("348da373550c14a3e398724cb213fe4c4011ef74b0f78c26ae61fdd1b7ce9d67").unwrap();
    let signature = G2Element::from_bytes(&Vec::from_hex("89c7cac0c3a9338c1fe67d3f715d7c0b7346633bce766420d536be5964f28aa7a7c7ea68062a66e076f36139b49068da12dc94bf3f5c6e917f64eb15ab9ecd8de2189059a57d355897a2f5d7cb4e2c7bf3602313dc855f08b4cbf32d5fc03ddb").unwrap()).unwrap();
    let public_keys = [
        "b32f6fc90c9dcaacdf9d836a2a7e60d090fe5e55b0b02f5a4f608a4b8235ba5aa7abc4e05f9387d1d942adc57c87f5b7",
        "8c9bf080a96d13b356b01e734618a77225f03b3e92684f252ccbd313764a9fd9247bde6b00d92f6b5669043e77860453",
        "b0051db915bd86bd938746c14440b11ee3b2801cbc6d6c1c912e8b41ea5eb1d8f852abf220ae91ecdb6da094846c1ba8",
        "92fa57a5676925e8dfe3b340df2132f5844ad9f89594b04efa28fb4fb884fe21f411fa49120ed7a60ce9381a54232a10",
        "93cc0376467317a70dac3d49ca1ce0c666b3f2630396fa208cc6e7d6401c691178b4cdf20deeccfe7d597e40cfba0f18",
        "95f9da603c572257802a689964ca8f4d96f9b94f33ab75968c9cb6c730a28d50b7bb72ac2cfceee6ab0755ead9cb53cd",
        "918081d1c248d74a0737f36e5bd40aa71b512c6be6f68e3664723849ac47a62fc743c4dc7234694bda1b7701f33d2e81",
        "87f3bb14d4e16bb20ebdfc97b3b067ebbcea5b0b9725b796e6c62d8a0818eff300261a62b6fbc3bdbaa97b895b66137c",
        "b4a637afe3810d73e3402b5d6a398e45222ba846a339f1c3570aa8e3f7f5b9d7acef08ac234cce4f706671498330a599",
        "93943908436a934c08582583b08cbcc50b4478bb79b7718789c25eb0ad2f3e5713ad4c152d4b1fd13cfd12bf896072e6",
        "83d890d1bc938b48547c8730000c4dcd5a940e55e82d1322ea600cc90c319111603fa52f928ec9d5dd29784b96f34af9",
        "8acab8f54530816bce0366a4f1a2319a445c535d074b53d5824a3e90b542e8b5a77181aad2e77560fb9f6fd7eb76532b",
        "983e7c881c6c556701b21eb3f837e2661ad4ae1ad5b9f11faf6cb1246daf99157f3da6491b8dca8517b33b32abce82a3",
        "80f8efb42f65ed9650078785be5d13e6e90eb9df87a99261d4de34df2b4b79a9c9b8c5e1aec7ac068ebef14636ceac4c",
        "b675a1940be872b6a0d4e1696bb39ea38179933a1bae02ae1eaf4b47f625bd939482f8791eb38925af47f73be027a64c",
        "ac1cc0f5e0a5aea680a170ec945074f5b83d83db4d208854204f57c0de220ceb63b0121bd2a7bdb214228338c575ff6f",
        "84c0ca8535c114f8f1b369f339b2653e7126610f5170b223970f4e63ad7b55ea2f61a08e263b51fb03f6940d655690f9",
        "afc31972124bb559aecd56dfb361048ab3f5203624c6436b1676b8d440bee777d83b937febacb2d3a651df9dbd20503e",
        "aaba308a8db9e1c6c545f3fdfff00880a973f0685207688d3c0ee4546d732ec16a81fa1aa952fb3190ff5c6febcd1a78",
        "8aeb5c2757211202b3afd2033ec1b4ef2dfe376ba5c6c07b45e6a7460afa4086423c4a704eb9a781514fbc513e190a62",
        "8f2df81ba65db70eaab625c5fe46f0f5e52a45b25c761686db23b4f18e547cb0d161912dd187302eb6f7c4a9a666a323",
        "94e199beb2a2a59166a12a851ef158928bc5efc25b39eb78b3a428b25384609d8c03548a94e77c0c941c90c68a4187d8",
        "b6ee48c7a71a9d8e0813e68ca09846245fa155285f24a62b0ce9cb0102b1994ec58af8ba2a01c09363bdcc395d41f3df",
        "a26fc7f30c49215b98d5cb47a350f888a306c52fa42c77e765b55288e622f03859273cae7e1cac99e67f7a9a96a6aa2c",
        "ac290b31d2e878c2d7235efb0c61f423aa37742a31318e61f8bb0bd6c110a892dc244512fec12a8b0fe7cbb08e12be28"
    ].iter().map(|s| { G1Element::from_bytes(&Vec::from_hex(s).unwrap()).unwrap() }).collect::<Vec<_>>();
    let schema = BasicSchemeMPL::new();
    let public_key = schema.aggregate_public_keys(public_keys.iter().collect::<Vec<&G1Element>>());
    let verified = schema.verify(&public_key, commitment_hash.as_ref(), &signature);
    assert!(verified);
}

// #[test]
fn test_verify_secure3() {
    let llmq_type = LLMQType::Llmqtype25_67;
    // let block_hash = "000000e6b51b9aba9754e6b4ef996ef1d142d6cfcc032c1fd7fc78ca6663ee0a";
//////////// validate_quorum 869760: 36d6a3181dd94d461a8242a1ab48bb9cf932f0d2e49af9afc943d2b3b8000000:
    let mut quorum = LLMQEntry {
        version: LLMQVersion::BLSBasicDefault,
        llmq_hash: UInt256::from_hex("36d6a3181dd94d461a8242a1ab48bb9cf932f0d2e49af9afc943d2b3b8000000").unwrap(),
        index: None,
        public_key: UInt384::from_hex("a0cf31b8cf35cd23d4ba8f1837db08a03ea70b7184896926978849faaaa83d61c79a1295103e0be3cb75e7dfa8e616f6").unwrap(),
        threshold_signature: UInt768::from_hex("aaf771a7032aa198b3dd3d58186dbd3b87e50ffb20ea53b0ab5907079109aea144db87141c9ca779802b08909e3ded280c318084f92035eec4570f1cdf1f3150079fd2452c40700b0965c97edb06e8b17dbd343fbfaa1da587a4c9091df3e5cf").unwrap(),
        verification_vector_hash: UInt256::from_hex("8587b73f6bd63382dc5c97c321135742ec878eb5feccaec2b7d2b2daabb82bbc").unwrap(),
        all_commitment_aggregated_signature: UInt768::from_hex("85c5262ab4c8e58cde79054b49d477274e1ef554ac9433c391110aa65813c7574e3982d734fb4593f646c70d62c9b68b0d666db5bc27251ea17caa8c0f7d77f1be5ffaeebf88824997a8ab342a2549ac617e9b549a4e375863cb38223952366e").unwrap(),
        llmq_type: LLMQType::Llmqtype25_67,
        signers_bitset: Vec::from_hex("ffffff01").unwrap(),
        signers_count: VarInt(25),
        valid_members_bitset: Vec::from_hex("ffffff01").unwrap(),
        valid_members_count: VarInt(25),
        entry_hash: UInt256::from_hex("514af6a43477f5354d1301fe9302c29f0cdcecfcb9cdbecbc72cd806543afc88").unwrap(),
        verified: false,
        saved: false,
        commitment_hash: None
    };

    let valid_masternodes = [
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("3ba330d521ffd8d0d7fee89e5b1222a91edd5ee077751bc171cec7b9157bcf9f").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("64e5082da5b5871e199e9607b9901a9bd69be06fd98d779e1da8937e1ea71147").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff12edaa20").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("8b04bcb5cf6d2d6df5979234611da42854a5e69374a29e0c85128caedb53d9c818042613d2f30f3ef782ed37bd8ce161").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("f1b18d69579c40dc8292842f02ff5f511e61609d").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("0b2c6e528a7512c8b504f34c30c38373d34a935d044546405ac8654ee58669e2").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("d9e2dc9e851176b315c763e988d7dfb1164153ef2ad98f5565686acbc2b8b2df").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("69d39d8413d5e25a4048b8269b7f6a2eeaa4b80d3b04812c7e88ef44f0024fff").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff36b838e0").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("a397c930f1927ffb7f7b13101f04e932d13d210de3a6254718cca8e5748941bab4fdb27a48f61f161f0d6cb63b7d3c85").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("849765a81c234c82ecef58cbfb96f2b1fcacf686").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("75007256476bb229815257529aa59e9cf5625e7f8137147de0d50c53e80ce711").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("432ab33e7663f79c28d25eb500edc0a6a17fa7ae2a718ce7efc2e79d3a070ad0").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("d3f738fefbe8e3185edf8f939a06b5bd77130da822fd0dd2b750ad8d1457e86b").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff36d63bae").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("93dac269908111b8b091edbda123d5884f4d47d21225fa319d344b350762a85c6cdbe21804ef9b2cc53a878c72a001d6").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("bae896aae3de540d18e5ea8573182860471082f9").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("f20d3c94aaec665afe13623784afe5e4d6c44edc2c274af5b01a0209e08956bc").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("0378aaa2e2ec71b47ab267aa9b55ec5422740471ce1651c2aee7cad48d8f0f61").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("4b5db3e037a861de2890b1bfaa79bd23fdd678519f4c45a8e9473de35b40b7c5").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff36ba9112").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("b0ea47f22be1644834d8756793f2308f2c5b40afd16ebb98d29a3bd37e437990d4d5930ccfa56c1ea0b4e51d05a49f23").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("bec1b55112fe6dee11593a3ec19e9d7128555218").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("ae3a244b4d5c5b668091aa089e19d2b012cea3d5650671bf247913fbc70558dd").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("2f51f56b6a87988b682020bc75f6e827f36d43559f2adb59cf85d9aaf7ef9257").unwrap(),
            confirmed_hash: UInt256::from_hex("795cfdb19aff1aaaed00bb62b95dfef5748253be13d0dfc4d783b02ae2000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("0078c2f3872da817c9901bfe498ec26b9bb023c6f30736ba92668fcc6900533c").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff3425306b").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("81ad0f9be5a88ae62ff54fe938dfceea71be03bd4c6a7aebf75896e8d495d310acc4146aa4820bc0e5f5b06579dedea5").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("cf83406065cb65ce74100f3c35a0324bfc92dcb0").unwrap(),
            is_valid: true,
            mn_type: HighPerformance,
            platform_http_port: 41733,
            platform_node_id: UInt160::from_hex("d96b1b3cefa0553732d9d6cae76666eff869aa02").unwrap(),
            entry_hash: UInt256::from_hex("816b2025864c75d69c90d89c73e043ce239acabb12b1c940b93b661542225691").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("bc973f3a17c5fa3dd59493b7d2bc41ce472be2401459ca2d96fac25bedb2b7aa").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("ec43d02ceb91ca9fea3cb6e6788fb48de7d2258233563681456f0f08f914dd1e").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff22dcf318").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("818a6d23ae53d6231f7dd73a058f125340e92f6e97897f017d9d9d4e6671bbd92241170dfcdd5a4ab8ef47ef12ddcad5").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("853fd590bbfa10868687b7c648238857248adfb7").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("3460032f52e5b57c80827f180263ad5551cde16a37ce1cff9052a0b47b2233dc").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("b793ac00290abfe0ae6e8e96dd228c0a209d774ecb90b1e7f384e3e975f2aade").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("284033f8d205f3a20ad4368b8069366768256578c817ddbd5c284d321352def4").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff22db21e7").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("97f78abcee6d2ed68bf2c82afbf56ef9af67313e2eb655ea5178850907cb3057cae0bb5a1d09f161057bf62f9d4890c6").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("76c44d67639aaa8fb877f1db08d5e582b570c25d").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("e27008fcb83b8a961b29fffe8b738958ac3bd7111ef0fcff4947d8444aced9f5").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("3316e1e2cc137b2eb03f960324df219b2c5314f12296df4b4619460ae9e7f994").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("316f3728969e8f563c3901fa3dbc4c322aab301c65ab3a983561366f5e32f0a2").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff235b787f").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("b5456e15b572b002651ddd30df525367cc830148fd15496c09a538e9562b102d1326e1ae99f7d37a0ac5f8cd1d4e205c").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("de0c01ad5889c4d3d0cd8dac5f08d329f47d06b8").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("107bf4d92436b91bfaf3e3c44af15cb11f994429d5fadfbdce3d7f0683f0291e").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("d62a3f54945dda86301e5b2118e6962c79b8d6461c075d97f1b926eda4e7dcae").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("bb02707fd56805b7507a2f449c2bc6c95715872c23015eeb1b445d2933ad4a6b").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff12eda5f2").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("807b1f3f16835ebfba6f505f43c6de757bb22ecf27a89703e90e43aedafea3df353a5bd1915b27e8db397d53f0a23f60").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("996eefc4c3da0c2c59fceec99aac79122c44d632").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("56c884c0ced7087c7734b6d8a345d516f102c1aa3158040e404332c595318dd7").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("995ed4288da0f7b3e9af9dfa8f2fcb6dc0389df6030e75c4a8fa4db5cede0d88").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("f2dde5275d2f37113433a3412ac5c2715db7de0532804494558f62815af942ea").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff22dc5551").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("911a30e0a5f2f5135dcc5f09498e4ba5de22c7680f396599f7f29b91ac569c3d4336bc157443cf8c06682bfb5abb2271").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("a9cab48330c4f2058fc7d331b9355b91056c68e1").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("a7e82894e3646eaab698bf52af4bcc9d02adb0737e6104473a28c30fc1006fdc").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("c10a2826f3e775d2d7527b131d5e3620647a6926938061a7ff16b7d0b745efb4").unwrap(),
            confirmed_hash: UInt256::from_hex("26cb9af0c2dd8555f03dccbc1f209376c344f2a89d021dad06bbe6aa2c020000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("b128a28a8255c2f2c5203982d349d5a15d8c1835f656082e7ac986dcb8d996b1").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff59280f17").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("8450dbbbe82df6808151b83a46f8c531cb240eccfe65f8f0b49f3717056da7268c14e45f0dd14fff8daed28fd353c1b4").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("1546085b56803db4366c2059d4755c8a28796ca3").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("aa7cb9fe35349094fdaf85e6018eae20d38b3208a42e8ff1cc842780a2e81c85").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("f682389d5e0baed394a65cbdd6a666f59271943d935f85c9d05b3433978c9b28").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("8e502ae310ff8f99c3efe28b76bd5d78e0cb7ee005a78f56e58c8576b406da01").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff22d254a3").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("8e2825781a496023c8be61f2bf352ad1094afd6e4f84c4ef331bc727bc149a6dd7e23d78944b8b047c03da44eef1c796").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("1c6031c212e840f9756b8f08e3801cca20b38a62").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("9b94fa8985507f9870305d431008cecd8feb30460696a557cc963d48dca4a7ab").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("0682c86d9afb864697190081adb9b1c4b0268b079edc428e41368fcb1655eff8").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("e966d7de4befc13b268099f4365c17eede37c4bdfcabdccf0466d74322be7dc6").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff235a9dce").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("815ae7a4f88fd79e4659c4b24b32f24d1e92106b867a2c23d1d084cfedd0e2766edd3f0a77f274acd4d1d53fb1ff0218").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("baee5acad0f663430ff83c4e9b9fcb0122789520").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("3d006dfd19d34fdca40d7f85c98bbd196abcf006442ead76e20d0fd93274de4b").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("eed53ba18185f82a82196e11705d4a0e40576cc1884c060188adbc5560ac8b14").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("29f50ee91bbe82d6a6a099b89d1fc5147cbe3ac70d02d16db64f2b0797204509").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff23a2a0b4").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("b5df04cdd8a9ffdad72b2ffaa0bf752b6fe9adc70c59834cca826a7a0f7264e4cbefd351772e30d527bc5aa9019c4ca5").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("e5ccbb0b6af934a3bcb32e3c1d1afe21aaadd582").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("5a16f1ecbe3cbb0780e7005131cb4ead957b88f1a83d1e21466538b42eefbda4").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("b2b69159973619cef08b9add969ba4d1ae18dd33bd62635429d44d93b62d5753").unwrap(),
            confirmed_hash: UInt256::from_hex("3d7c98b3942c5550e3bfcaa37e37282fd4b29bb40f677a38906e597257000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("4fc89cf01157342238661a79c22bdf80def51eaba53fe98088ed1bed1c83fccc").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffffae22e975").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("87f818e5c2330ac4e7f0ef820f337addf8ab28b07c9d451304d807feda1d764c7074bccbbd941284b0d0276a96cf5e7f").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("ce7f101b7f075273c892063b1b8571311737a576").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("08dd2c380037f78507443f306f60a4b95d888576537a2a5ce7481d0cb45ea30a").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("ab0c1c4d1b59e95ef256598fc2663049f0fcf1a6e4845e80358b7d29c04dcb76").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("eb8b4a877d01aafb185de75812249b709a48d38b0551dba9301acb451fb4408f").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff340a7242").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("87f3bb14d4e16bb20ebdfc97b3b067ebbcea5b0b9725b796e6c62d8a0818eff300261a62b6fbc3bdbaa97b895b66137c").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("93bc1ac3bd2e43eb87e240d2168f56e69b1f1833").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("c965262aa365649a04a917b2eb5cd5f753107c0297448d1a0572dde1e8fe38e3").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("5efba19efb8cb5e105f5b61f9ca2b0f543a0dee8161c0fae961d1e76ee883a07").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("48a1fa1dae598d607a00efc48e52db278426b1f09aad2916c31713257ac571d2").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff22dd5625").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("b8a3692f75e5b9523c38f02c6a7ba91425bf6a6343f8704bccfeefd4844456196bc6b3267b7f3cbb2200c549f4313c42").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("1d8262ba8d79530a35b175b5b47c161e70e0e154").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("320cf222c2340e7463c0a256e8a2e81f41276ace2183a5ead1774453bc9614fc").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("790bd1b645d4cbbb155fe4a4c65b81a285197c6b94ef89b05cd2a024b7a6c441").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("829fe002b9ac51276e4555e47546875cac7c01a17ef0ea84d221eeb8a71be528").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff22dcbbe9").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("87df25a28955c903cc19f836a4daa0842d203cfc0dc5ae9b57b8246a4787ee4c98ea3f2586203315d61f4e77b6c80dc5").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("8164fcd1a488ebde9344cbd205ce705bbe312180").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("edb3cade9b9f40c5ab811743c3a9bb694a78f59f4fb1fdb31caf90011036f239").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("e3fd43389b149f8c4d64f36b1601f05ec3f3ceae0809dbeed137214d11b376a7").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("fda5dbe241b787f39a0e36d6e33a454b5b9d0e5ba12702fc27245250b58753c7").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff22d3acd4").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("89f8a06bd95c1be3cfdcd2516fabc0858c611d63c76da3a5beaa007b9d7c895aa63c0b2887bd584a76892db417a6683f").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("127b867e4c2d040cedaff979c39220c717011dab").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("0c0f28c40747d00857c61d3c661537f7573912a44a74ed51333f224f82a811b0").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("02769272e89a947728aebb2b1ce38a35e47ea334ee53fe29e8a6734cf439ee0c").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("f23d63f18ec98e1e467e5e89f3c22c739bc0729fc9c0d17a27c9a7137d590fa6").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff36bd7deb").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("a80f24b5e040dcbf86c3f468dd28bf45d9e41fbcd127fad56669d9afe358dcdc26e42f0f8b19997b1741dbb99c553aa6").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("b73068d00c048915c175450c366478f783bc455b").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("66ad4b21995106bb17b63a94752e284318d501f34fb1ecd8bea21c64cb3661be").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("d28018e798ccbd797d0b2fc33513d64d60d55c92f4b35f46db169332dae95f4d").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("e9273730016cd6686e327bed5bc61942e409b09ce9acad5c556313f7eb69c80e").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff3422fad6").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("a7be789e5b798cf3a40ff5dc22b0384dc690acadd614067c0f7e6a933b8f0c72c67b3f4b3e666e6fc48369a8161b04e6").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("4bd7db8a067386d8bda99986fd6661186757bf95").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("eea47e59d78ea141719ca2d19358d299c269d8948ec9f5b283e2edc1f2038c66").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("7260b2359075b4e479d94821e1745239384e92da675226464f1fc312682df6b2").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("b486e7118e23a4cb5f7632013217affe8c3ce0e7bfd45d3fbfcf28b4002dba11").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff22de5512").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("875b907b6d6c12aa111da0e102186b9d06f4e065969b60732207f18c2c5d0deb8ecba47cb4c0929647db0e2fae6f08ca").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("a3efa844b6dc22d05e802ebd5d1eb02cca122c25").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("5821d4b7dfe9734350412a08a8b1a8efecead5383a47468a4e98256cec04f08a").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("f5cbcfe4f680596d703098ef1fc0ce9e3950372bda82c25090b79ae880e6932b").unwrap(),
            confirmed_hash: UInt256::from_hex("4f2ff1d62323f7dc16ccdfb3e3740af47a444871f72f09ac252f62753e000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("c2a3dd61f4c8a7b6e5934e409e9a4e9a48ed6cd9cc14424bdcce020bfb20b362").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff23a3ba6d").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("a7afe7674de986aff5e2e0a173be8c29abed8b5d6f878389ea18be0d43c62ad1ba66a59e9e8d8453aa0ed1a696976758").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("762e689e7bfecbe16bfd0e6138eea642ee0513c5").unwrap(),
            is_valid: true,
            mn_type: HighPerformance,
            platform_http_port: 41733,
            platform_node_id: UInt160::from_hex("e3053f65754c630ab036f1370bcc6835fedb9c9b").unwrap(),
            entry_hash: UInt256::from_hex("03430ef59311f9797152779acb03d6c25fca40567b75f84058bf2f5ef3da335a").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("71ce9f30984ba8ff47618bdbb362be8315b11d7be903b37fb9ed2a0010090dca").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("ece3d829df81bd32c1d45bb360600377fa73c619579d1a5daddcb95ec91357bb").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff36d45b94").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("b2823797ad456d53ce1e6bde84e8a19164ff88a73ccd242ec48d9c6a479f2a049e214c7e8ec2243b7ea74ca6144ab2c5").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("b7e99fc3ee4cf3d64c4438fc2a61cba12c0e7319").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("0526b60a35c51286e3685f143e0664d08a155bfdc49f1ae0f76db95709f24e99").unwrap(),
        },
        MasternodeEntry {
            provider_registration_transaction_hash: UInt256::from_hex("608c43a6a23324860eb2d8e6212798506bebac53120c591d6b32aeb0e35edc63").unwrap(),
            confirmed_hash: UInt256::from_hex("042a425ae2d3289646d71765e97cfe099acdf021aaeabd52393477d845000000").unwrap(),
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                UInt256::from_hex("c83d2ef9b1b1772e173a9f088d0151582f9eeaef3ce5c6ba28c5aac025af112c").unwrap(),
            ),
            socket_address: SocketAddress {
                ip_address: UInt128::from_hex("00000000000000000000ffff235bc5da").unwrap(),
                port: 19999,
            },
            operator_public_key: OperatorPublicKey {
                data: UInt384::from_hex("93f411bb160a34b3d8254e7c537e1300afed010d4a245e376b81d889020854fb999fe9cbb7430ddee0faf2fe5e711ebb").unwrap(),
                version: 2,
            },
            previous_operator_public_keys: BTreeMap::from([]),
            previous_entry_hashes: BTreeMap::from([]),
            previous_validity: BTreeMap::from([]),
            known_confirmed_at_height: Some(
                868888,
            ),
            update_height: 868888,
            key_id_voting: UInt160::from_hex("b190542b5c7522f4db19e48033d17c34afd71845").unwrap(),
            is_valid: true,
            mn_type: Regular,
            platform_http_port: 0,
            platform_node_id: UInt160::from_hex("0000000000000000000000000000000000000000").unwrap(),
            entry_hash: UInt256::from_hex("76d162d303800ff542f6b69f938b911712717332c0ea4ca061c3e7f341632f39").unwrap(),
        },
    ];

    quorum.validate(valid_masternodes.to_vec(), 869760);
}

//#[test]
fn test_verify_25_67() {
    register_logger();
    let version = 70227;
    let cache = register_cache();
    let context = &mut create_default_context(ChainType::TestNet, false, cache);
    let processor = register_default_processor();
    let result = process_mnlistdiff(message_from_file("MNL_0_871104.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("MNL_0_874011.dat"), processor, context, version, false, true);
    // assert_diff_result(context, result);
}

// #[test]
// fn test_verify_chained_rotation() {
//     register_logger();
//     let version = 70227;
//     let cache = register_cache();
//     let context = &mut create_default_context(ChainType::TestNet, false, cache);
//     let processor = register_default_processor();
//     let diffs = vec![
//         "MNL_0_870600.dat",
//         "MNL_870600_870624.dat",
//         "MNL_870624_870648.dat",
//         "MNL_870648_870672.dat",
//         "MNL_870672_870696.dat",
//         "MNL_870696_870720.dat",
//         "MNL_870720_870744.dat",
//         "MNL_870744_870768.dat",
//         "MNL_870768_870792.dat",
//         "MNL_870792_870816.dat",
//         "MNL_870816_870840.dat",
//         "MNL_870840_870864.dat",
//         "MNL_870864_870888.dat",
//         "MNL_870888_870912.dat",
//         "MNL_870912_870936.dat",
//         "MNL_870936_870960.dat",
//         "MNL_870960_870984.dat",
//         "MNL_870984_871008.dat",
//         "MNL_871008_871032.dat",
//         "MNL_871032_871056.dat",
//         "MNL_871056_871080.dat",
//         "MNL_871080_871104.dat",
//         "MNL_871104_871128.dat",
//         "MNL_871128_871152.dat",
//         "MNL_871152_874488.dat",
//         "MNL_874488_874512.dat",
//         "MNL_874512_874536.dat",
//         "MNL_874536_874560.dat",
//         "MNL_874560_874584.dat",
//         "MNL_874584_874608.dat",
//         "MNL_874608_874632.dat",
//         "MNL_874632_874656.dat",
//         "MNL_874656_874680.dat",
//         "MNL_874680_874704.dat",
//         "MNL_874704_874728.dat",
//         "MNL_874728_874752.dat",
//         "MNL_874752_874776.dat",
//         "MNL_874776_874800.dat",
//         "MNL_874800_874824.dat",
//         "MNL_874824_874848.dat",
//         "MNL_874848_874872.dat",
//         "MNL_874872_874896.dat",
//         "MNL_874896_874920.dat",
//         "MNL_874920_874944.dat",
//         "MNL_874944_874968.dat",
//         "MNL_874968_874992.dat",
//         "MNL_874992_875016.dat",
//         "MNL_875016_875040.dat",
//         "MNL_875040_875064.dat",
//         "MNL_875064_875088.dat",
//         "MNL_875088_875112.dat",
//         "MNL_875112_875136.dat",
//         "MNL_875136_875160.dat",
//         "MNL_875160_875184.dat",
//         "MNL_875184_875208.dat",
//         "MNL_875208_875241.dat",
//         "MNL_875241_875242.dat"
//     ].iter().for_each(|name| {
//         let result = process_mnlistdiff(message_from_file(format!("testnet/{}", name).as_str()), processor, context, version, false, true);
//         assert_diff_result(context, result);
//     });
//     context.is_dip_0024 = true;
//     let result = process_qrinfo(message_from_file("testnet/QRINFO_0_875241.dat"), processor, context, version, false, true);
//     assert_diff_result(context, unsafe { *result.result_at_h_4c });
//     assert_diff_result(context, unsafe { *result.result_at_h_3c });
//     assert_diff_result(context, unsafe { *result.result_at_h_2c });
//     assert_diff_result(context, unsafe { *result.result_at_h_c });
//     assert_diff_result(context, unsafe { *result.result_at_h });
//     assert_diff_result(context, unsafe { *result.result_at_tip });
//
//     let result = process_qrinfo(message_from_file("testnet/QRINFO_875241_875242.dat"), processor, context, version, false, true);
//     assert_diff_result(context, unsafe { *result.result_at_h_4c });
//     assert_diff_result(context, unsafe { *result.result_at_h_3c });
//     assert_diff_result(context, unsafe { *result.result_at_h_2c });
//     assert_diff_result(context, unsafe { *result.result_at_h_c });
//     assert_diff_result(context, unsafe { *result.result_at_h });
//     assert_diff_result(context, unsafe { *result.result_at_tip });
// }
//
// #[test]
// fn test_verify_chained_rotation2() {
//     register_logger();
//     let version = 70227;
//     let cache = register_cache();
//     let context = &mut create_default_context(ChainType::TestNet, false, cache);
//     let processor = register_default_processor();
//     context.is_dip_0024 = true;
//     let result = process_qrinfo(message_from_file("testnet/QRINFO_0_888537.dat"), processor, context, version, false, true);
//     assert_diff_result(context, unsafe { *result.result_at_h_4c });
//     assert_diff_result(context, unsafe { *result.result_at_h_3c });
//     assert_diff_result(context, unsafe { *result.result_at_h_2c });
//     assert_diff_result(context, unsafe { *result.result_at_h_c });
//     assert_diff_result(context, unsafe { *result.result_at_h });
//     assert_diff_result(context, unsafe { *result.result_at_tip });
// }
//
// #[test]
// fn test_verify_chained_rotation3() {
//     register_logger();
//     let version = 70227;
//     let cache = register_cache();
//     let context = &mut create_default_context(ChainType::TestNet, false, cache);
//     let processor = register_default_processor();
//     let diffs = vec![
//         "MNL_0_888192.dat",
//         "MNL_888192_888193.dat",
//         "MNL_888193_888194.dat",
//         "MNL_888194_888195.dat",
//         "MNL_888195_888196.dat",
//         "MNL_888196_888197.dat",
//         "MNL_888197_888198.dat",
//         "MNL_888198_888199.dat",
//         "MNL_888199_888200.dat",
//         "MNL_888200_888201.dat",
//         "MNL_888201_888202.dat",
//         "MNL_888202_888203.dat",
//         "MNL_888203_888204.dat",
//         "MNL_888204_888205.dat",
//         "MNL_888205_888206.dat",
//         "MNL_888206_888207.dat",
//         "MNL_888207_888208.dat",
//         "MNL_888208_888209.dat",
//         "MNL_888209_888210.dat",
//         "MNL_888210_888211.dat",
//         "MNL_888211_888212.dat",
//         "MNL_888212_888213.dat",
//         "MNL_888213_888214.dat",
//         "MNL_888214_888215.dat",
//         "MNL_888215_888216.dat",
//         "MNL_888216_888217.dat",
//         "MNL_888217_888218.dat",
//         "MNL_888218_888219.dat",
//         "MNL_888219_888220.dat",
//         "MNL_888220_888221.dat",
//         "MNL_888221_888222.dat",
//         "MNL_888222_888223.dat",
//       ].iter().for_each(|name| {
//         let result = process_mnlistdiff(message_from_file(format!("testnet/{}", name).as_str()), processor, context, version, false, true);
//         assert_diff_result(context, result);
//     });
//
//     context.is_dip_0024 = true;
//     let result = process_qrinfo(message_from_file("testnet/QRINFO_0_888655.dat"), processor, context, version, false, true);
//     assert_diff_result(context, unsafe { *result.result_at_h_4c });
//     assert_diff_result(context, unsafe { *result.result_at_h_3c });
//     assert_diff_result(context, unsafe { *result.result_at_h_2c });
//     assert_diff_result(context, unsafe { *result.result_at_h_c });
//     assert_diff_result(context, unsafe { *result.result_at_h });
//     assert_diff_result(context, unsafe { *result.result_at_tip });
// }


#[test]
fn test_core19_2() {
    register_logger();
    let version = 70228;
    let cache = register_cache();
    let context = &mut create_default_context(ChainType::TestNet, false, cache);
    let processor = register_default_processor();
    let diffs = vec![
        "MNL_0_530000_70228.dat",
        "MNL_530000_852596.dat",
    ].iter().for_each(|name| {
        let result = process_mnlistdiff(message_from_file(format!("testnet/{}", name).as_str()), processor, context, version, false, true);
        assert_diff_result(context, result);
    });
}
