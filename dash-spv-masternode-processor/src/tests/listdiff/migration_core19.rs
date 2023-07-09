use crate::chain::common::{ChainType, DevnetType};
use crate::lib_tests::tests::{assert_diff_result, assert_qrinfo_result, create_default_context, MerkleBlock, message_from_file, process_mnlistdiff, process_qrinfo, register_cache, register_default_processor, register_logger};

#[test]
fn test_verify_chained_rotation() {
    register_logger();
    let version = 70224;
    let cache = register_cache();
    let context = &mut create_default_context(ChainType::DevNet(DevnetType::Screwdriver), false, cache);
    context.blocks = vec![
        MerkleBlock::reversed(4840, "0000012768da68f985b294c80ee5f8077d29aa418fc66f68589069c8c21c0b33", "1debef742e21db3326716c265ffee6fe0883020d4b7d04986c8003affe5e7e8e"),
        MerkleBlock::reversed(4872, "0000029d25df5aff9374c87e303da6032c7e8ec6517842e637a79b9d6d433e46", "149d92e7610e03ea62767a1cea6305aa97e7097ceadbe7ef6c34e01c065aee08"),
        MerkleBlock::reversed(4888, "0000022c2d5726c2a664b96f551f3e83c8cfc574f76ce2e9477075d582cf39dc", "74f1f7f66a576961c50d4742ea12d02de4d7724ab77798c3acbf5b99e224c069"),
        MerkleBlock::reversed(4896, "000001182e5764ae984c46fb2bd78fce01948843c8f03ac9605d036e39c27e79", "e039bac930e912d448499da5bfefd7837629aaa1fb477f3a81dd5e936033ecb0"),
        MerkleBlock::reversed(4920, "000000bf59863ba21575c37b7ef53497453396508f529f101b6e11575e47c234", "9db6eebef676c7672a5d6b63fefc20c51af9db743f539a16b4db2e9f5f84774b"),
        MerkleBlock::reversed(4936, "00000157d5e963945c891cc074f2e449ac487504b94e416d117b7346704b1e01", "4edb31b6abbf9cf70862850d4fbbcdb76f218afac382e13e9d3a3dd4f3df68ad"),
        MerkleBlock::reversed(4944, "000000995145233fe3d01ad30e4a3b857e84c9e64a3aa60279aabbfb883ab643", "eb0fceac6c75fb7a7b31d4d535969968e256c4d28d9ef51541468ba7178d6608"),
        MerkleBlock::reversed(4968, "000002642837447bab0912fe7cc4d83d95ce275f97ffaaaa20cd374f0d39d95d", "1be93f2f91380dca34b339c5f09028d91b11a8379b6e94b1251095a942260253"),
        MerkleBlock::reversed(4984, "000002f7cae2beaeb050f4ea23e02940b0ab3a98722b329b328ac73f9487e6f9", "feecf6a2f5303536b54ce07fe1e1746c7f03a08ada93f53f6d1160f4b68eda04"),
        MerkleBlock::reversed(4992, "000000795a16ffa6de765747c7cd1fd7ad48e3a899adc4f25c2b84b122181a5c", "a971a6d8bd96b1db56f86210950301987c976ddb8ce946f3486e494c7e6ff21a"),
        MerkleBlock::reversed(4993, "0000012f2a8f19afd26771c49f1e6f378cabb8a9d2256c29ed694259b6579732", "8bb97d7c0ceebac65f2c6616e262e09276e206df5a701315b6284ea6150f3b0b"),
        MerkleBlock::reversed(4994, "0000018081e31ed485dbd34b73848dacfd8199d5c157ed8713862d86f8f4d5c5", "b0e5e73b09f19218333b142edf60eec11779a1dc3ec15c03e4f426ef73caa714"),
        MerkleBlock::reversed(4995, "000001f651ecb6cd5e7d7a75d63b87dc553d4d9b9edbf2d395eb6186be59c64c", "a49c2d473960e492de4d997a7049cd2b21c2d15c71c6f1dad75d580b3f5b038c"),
        MerkleBlock::reversed(4996, "000001ef3630a372e0dabfdb9d528bdb117b79039a67619ba4de39339638d4bc", "af1fa60f314f50f718917a9108d167e819061b30555e31516ca86d0713d92199"),
        MerkleBlock::reversed(5016, "00000019ea1f2c0d5db4c929897d70c9d3bd2b12b08e3948341688699b6ae63b", "239095828fbfcb42fdac4bd64c922fac6038e1b7a002fc234f2b8797f6847715"),
        MerkleBlock::reversed(5032, "00000072cae9600a36e1040831a8c266d187b035d0d1366ece465a693df3cfec", "0a57f43c1c8290a7dfb956328799c494d4d0a21ac2138b10ea047c5e89141321"),
        MerkleBlock::reversed(5040, "000001ab3033f1a00490aeb9359747fd6ab1c64fb50937c0357bb5bc0334ca58", "f739d678fc728d5e7391d04431789b3f65bda33badb8a510469ef738abacea45"),
        MerkleBlock::reversed(5064, "000000d1095e75090835ce87246b1dfb105c4c31fa2942a5cf553f3e83aa3faa", "215339c99a1cba055162f8a0468fb6479ba8b40ecc5d553f49c8e059dc6527e7"),
        MerkleBlock::reversed(5088, "00000095908d41ade340072107f74fe6a66b3148b692f3a25df6a29d5ca23e6b", "9c645a0770ff3b47c917b49775a475bcb93cc5871d5c5cafdecf163ebbbf0f2d"),
        MerkleBlock::reversed(5112, "000001ae03fa0003fd6ba6b19b812ef6b50591affb29c959496a803a5a5b00f6", "eb7ddabe2b91a5f6cc7bc4df5aaff199274df53481cfdf0a7f69a025ea1e6660"),         MerkleBlock::reversed(1, "4ac35ceb629e529b2a0eb2e2676983d4b11ebddaff5bd00cae7156a02b521e6f", "3ed1169f5d3e92c4b00322b6da549dbbdfeefceb0aac81dda77a144ccbd61d67"),
        MerkleBlock::reversed(5118, "0000003911af6750e5710238ee9dbab83812bf2af3dc9411e755ef2ae870c843", "b76872fec9896277e39052794cad91051a30759c618acf4c079d8e45180a864d"),
        MerkleBlock::reversed(5140, "000001321487a80eefd7f6137da9bb9b06f19143cbdaa15cfda63753bd8eee46", "c344d520890dc4e1f16be687e359b13578e74ae14d01183ca417650d7498eb81"),
    ];
    let processor = register_default_processor();
    // let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_1_5040.dat"), processor, context, version, false, true);
    // assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_1_4968.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);

    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_4968_4992.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_4992_4993.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_4993_4994.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_4994_4995.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_4995_4996.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_4996_5016.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_5016_5032.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_5032_5040.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);

    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_5040_5064.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_5064_5088.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_5088_5112.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);
    let result = process_mnlistdiff(message_from_file("devnet-screwdriver/MNL_5112_5140.dat"), processor, context, version, false, true);
    assert_diff_result(context, result);


    context.is_dip_0024 = true;
    let result = process_qrinfo(message_from_file("devnet-screwdriver/QRINFO_1_5140.dat"), processor, context, version, false, true);
    assert_qrinfo_result(context, result);
    let result = process_qrinfo(message_from_file("devnet-screwdriver/QRINFO_5032_5140.dat"), processor, context, version, false, true);
    assert_qrinfo_result(context, result);
}
