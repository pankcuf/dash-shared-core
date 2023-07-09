use crate::chain::common::{ChainType, DevnetType};
use crate::lib_tests::tests::{assert_diff_result, create_default_context, MerkleBlock, message_from_file, process_mnlistdiff, register_cache, register_default_processor, register_logger};

// #[test]
fn test_quorums_cl_sigs() {
    register_logger();
    let version = 70228;
    let cache = register_cache();
    let context = &mut create_default_context(ChainType::DevNet(DevnetType::Absinthe), false, cache);
    context.blocks = vec![
        MerkleBlock::reversed(1, "53ab7716f36a92068d7bbfa6475681018788a438e028d8bfdf86bfff4f6b78ab", "1160570b6d94a948e25aa8ecef4098b4a6a1f4b4c2ba82659da59159c4f7cf6a"),
        MerkleBlock::new(8561, "c77aafb726e115bceb80f6768d70fa32505c9071c022928c06b12b3b87000000", "0000000000000000000000000000000000000000000000000000000000000000"),
        MerkleBlock::new(8562, "31d70ac30bde9be20279876cbc1302a5d6d11172599d8dbd23ff44a813050000", "0000000000000000000000000000000000000000000000000000000000000000"),
    ];

    let processor = register_default_processor();
    let diffs = vec![
        "MNL_0_0.dat",
    ].iter().for_each(|name| {
        let result = process_mnlistdiff(message_from_file(format!("devnet-absinthe/{}", name).as_str()), processor, context, version, false, true);
        assert_diff_result(context, result);
    });

}
