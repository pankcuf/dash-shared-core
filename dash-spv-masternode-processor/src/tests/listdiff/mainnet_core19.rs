use crate::chain::common::ChainType;
use crate::lib_tests::tests::{assert_diff_result, assert_qrinfo_result, create_default_context, message_from_file, process_mnlistdiff, process_qrinfo, register_cache, register_default_processor, register_logger};

// #[test]
fn test_verify_chained_rotation2() {
    register_logger();
    let version = 70227;
    let cache = register_cache();
    let context = &mut create_default_context(ChainType::MainNet, false, cache);
    let processor = register_default_processor();
    let diffs = vec![
        "MNL_0_1870848.dat",
        "MNL_1870848_1871136.dat",
        "MNL_1871136_1871184.dat",
        "MNL_1871184_1871208.dat",
        "MNL_1871208_1871232.dat",
        "MNL_1871232_1871256.dat",
        "MNL_1871256_1871280.dat",
        "MNL_1871280_1871304.dat",
        "MNL_1871304_1871328.dat",
        "MNL_1871328_1871352.dat",
        "MNL_1871352_1871376.dat",
        "MNL_1871376_1871400.dat",
        "MNL_1871400_1871424.dat",
        "MNL_1871424_1871448.dat",
        "MNL_1871448_1871472.dat",
        "MNL_1871472_1871496.dat",
        "MNL_1871496_1871520.dat",
        "MNL_1871520_1871544.dat",
        "MNL_1871544_1871568.dat",
        "MNL_1871568_1871592.dat",
        "MNL_1871592_1871616.dat",
        "MNL_1871616_1871640.dat",
        "MNL_1871640_1871664.dat",
        "MNL_1871664_1871688.dat",
        "MNL_1871688_1871712.dat",
        "MNL_1871712_1871736.dat",
        "MNL_1871736_1871755.dat",
    ].iter().for_each(|name| {
        let result = process_mnlistdiff(message_from_file(format!("mainnet/{}", name).as_str()), processor, context, version, false, true);
        assert_diff_result(context, result);
    });
    context.is_dip_0024 = true;
    let result = process_qrinfo(message_from_file("mainnet/QRINFO_0_1871755.dat"), processor, context, version, false, true);
    assert_diff_result(context, unsafe { *result.result_at_h_4c });
    assert_diff_result(context, unsafe { *result.result_at_h_3c });
    assert_diff_result(context, unsafe { *result.result_at_h_2c });
    assert_diff_result(context, unsafe { *result.result_at_h_c });
    assert_diff_result(context, unsafe { *result.result_at_h });
    assert_diff_result(context, unsafe { *result.result_at_tip });
}

// #[test]
fn test_verify_chained_rotation() {
    register_logger();
    let version = 70227;
    let cache = register_cache();
    let context = &mut create_default_context(ChainType::MainNet, false, cache);
    let processor = register_default_processor();
    let diffs = vec![
        "0_1870840",
        "1870840_1871712",
        "1871712_1871722",
        "1871722_1871724",
        "1871724_1871726",
        "1871726_1871727",
        "1871727_1871730",
        "1871730_1871733",
        "1871733_1871734",
        "1871734_1871738",
        "1871738_1872001",
        "1872001_1872002",
        "1872002_1872003",
        "1872003_1872004",
        "1872004_1872005",
        "1872005_1872006",
        "1872006_1872007",
        "1872007_1872008",
        "1872008_1872009",
        "1872009_1872011",
        "1872011_1872013",
        "1872013_1872016",
        "1872016_1872017",
        "1872017_1872019",
        "1872019_1872020",
        "1872020_1872023",
        "1872023_1872024",
        "1872024_1872025",
        "1872025_1872027",
        "1872027_1872028",
        "1872028_1872029",
        "1872029_1872030",
        "1872030_1872031",
    ].iter().for_each(|name| {
        let result = process_mnlistdiff(message_from_file(format!("mainnet/MNL_{}.dat", name).as_str()), processor, context, version, false, true);
        assert_diff_result(context, result);
    });
    context.is_dip_0024 = true;
    let result = process_qrinfo(message_from_file("mainnet/QRINFO_0_1872425.dat"), processor, context, version, false, true);
    assert_qrinfo_result(context, result);
}
