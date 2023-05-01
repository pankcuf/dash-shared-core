use crate::ffi::boxer::boxed;
use crate::lib_tests::tests::{add_insight_lookup_default, get_block_hash_by_height_default, get_llmq_snapshot_by_block_hash_default, get_masternode_list_by_block_hash_default, get_masternode_list_by_block_hash_from_cache, get_merkle_root_by_hash_default, hash_destroy_default, masternode_list_destroy_default, masternode_list_save_default, masternode_list_save_in_cache, message_from_file, process_mnlistdiff_from_message_internal, process_qrinfo_from_message_internal, save_llmq_snapshot_default, save_llmq_snapshot_in_cache, should_process_diff_with_range_default, snapshot_destroy_default, FFIContext};
use crate::chain::common::chain_type::{ChainType, DevnetType};
use crate::crypto::byte_util::{Reversable, UInt256};
use crate::hashes::hex::{FromHex, ToHex};
use crate::bindings::common::{processor_create_cache, register_processor};
use crate::bindings::masternode::process_qrinfo_from_message;

// #[test]
// Deprecated
fn test_llmq_rotation() {
    let bytes = message_from_file("qrinfo--1-5078.dat");
    let length = bytes.len();
    let c_array = bytes.as_ptr();
    let use_insight_as_backup = false;
    let chain = ChainType::DevNet(DevnetType::Devnet333);
    let cache = unsafe { &mut *processor_create_cache() };
    let context = &mut (FFIContext {
        chain,
        is_dip_0024: true,
        cache,
        blocks: vec![]
    }) as *mut _ as *mut std::ffi::c_void;
    let processor = unsafe {
        register_processor(
            get_merkle_root_by_hash_default,
            block_height_lookup_5078,
            get_block_hash_by_height_default,
            get_llmq_snapshot_by_block_hash_default,
            save_llmq_snapshot_default,
            get_masternode_list_by_block_hash_default,
            masternode_list_save_default,
            masternode_list_destroy_default,
            add_insight_lookup_default,
            hash_destroy_default,
            snapshot_destroy_default,
            should_process_diff_with_range_default,
        )
    };
    let result = unsafe { process_qrinfo_from_message(
        bytes.as_ptr(),
        bytes.len(),
        chain,
        use_insight_as_backup,
        false,
        true,
        70221,
        processor,
        cache,
        context,
    )};
    println!("{:?}", result);
    let result_5078 = unsafe { *result };
    let result_at_h = unsafe { *result_5078.result_at_h };
    assert!(
        result_at_h.has_found_coinbase,
        "Did not find coinbase at height 5078"
    );
    // turned off on purpose as we don't have the coinbase block
    // assert!(result.valid_coinbase, "Coinbase not valid at height {}", h);
    // assert!(result_at_h.has_valid_mn_list_root, "mn list root not valid at height {}", h);
    // assert!(result_at_h.has_valid_llmq_list_root, "LLMQ list root not valid at height {}", h);
    // assert!(result_at_h.has_valid_quorums, "validQuorums not valid at height {}", h);
}
pub unsafe extern "C" fn block_height_lookup_5078(
    _block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> u32 {
    5078
}

#[test]
fn test_llmq_rotation_2() {
    let bytes = message_from_file("QRINFO_1_8344.dat");
    let use_insight_as_backup = false;
    let chain = ChainType::DevNet(DevnetType::Devnet333);
    let cache = unsafe { &mut *processor_create_cache() };
    let context = &mut (FFIContext {
        chain,
        is_dip_0024: true,
        cache,
        blocks: vec![]
    }) as *mut _ as *mut std::ffi::c_void;
    println!("test_llmq_rotation_2 {:?}", bytes.to_hex());
    let processor = unsafe {
        register_processor(
            get_merkle_root_by_hash_default,
            block_height_lookup_,
            get_block_hash_by_height_default,
            get_llmq_snapshot_by_block_hash_default,
            save_llmq_snapshot_default,
            get_masternode_list_by_block_hash_default,
            masternode_list_save_default,
            masternode_list_destroy_default,
            add_insight_lookup_default,
            hash_destroy_default,
            snapshot_destroy_default,
            should_process_diff_with_range_default,
        )
    };
    let result = unsafe { process_qrinfo_from_message(
        bytes.as_ptr(),
        bytes.len(),
        chain,
        use_insight_as_backup,
        false,
        true,
        70221,
        processor,
        cache,
        context,
    )};
}

unsafe extern "C" fn block_height_lookup_(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> u32 {
    let h = UInt256(*(block_hash));
    let orig_s = h.clone().to_string();
    let rev = h.reversed();
    let rev_s = rev.to_string();
    match rev_s.as_str() {
        "00000072f3c73d891d86546f259ba2cd87d1aa655c447640a4257f6a8e6f7018" => 5334,
        "000000a451ba6459b3ce6128a5e8f273f9bc2010645dd4721e1b51efce18dda7" => 4207,
        "000000076aeba26f76a5d0e12e11c9b4d35d7232f1bbae6c47b4d8bef4a12b62" => 4192,
        "00000179987c39850ddd901eec6bfd0a508ec54fb6a0cd28481481aa0adf56b6" => 4168,
        "00000028bd64fd360dba79acf7cb3bae6cea18553c7232894a2ace15ada70940" => 4144,
        "000002410622902b361d1e2194f2072c6409c6f22ef5fea854d3326a27075713" => 4120,
        "000001f340d35fe89d1924de57ccbf63a7a09347835e6e4990ee2df12a4a67f9" => 4096,
        _ => u32::MAX,
    }
}

unsafe extern "C" fn get_block_hash_by_height_(
    block_height: u32,
    _context: *const std::ffi::c_void,
) -> *const u8 {
    match block_height {
        5334 => {
            UInt256::from_hex("00000072f3c73d891d86546f259ba2cd87d1aa655c447640a4257f6a8e6f7018")
                .unwrap()
                .reverse()
                .0
                .as_ptr()
        }
        4207 => {
            UInt256::from_hex("000000a451ba6459b3ce6128a5e8f273f9bc2010645dd4721e1b51efce18dda7")
                .unwrap()
                .reverse()
                .0
                .as_ptr()
        }
        4192 => {
            UInt256::from_hex("000000076aeba26f76a5d0e12e11c9b4d35d7232f1bbae6c47b4d8bef4a12b62")
                .unwrap()
                .reverse()
                .0
                .as_ptr()
        }
        4168 => {
            UInt256::from_hex("00000179987c39850ddd901eec6bfd0a508ec54fb6a0cd28481481aa0adf56b6")
                .unwrap()
                .reverse()
                .0
                .as_ptr()
        }
        4144 => {
            UInt256::from_hex("00000028bd64fd360dba79acf7cb3bae6cea18553c7232894a2ace15ada70940")
                .unwrap()
                .reverse()
                .0
                .as_ptr()
        }
        4120 => {
            UInt256::from_hex("000002410622902b361d1e2194f2072c6409c6f22ef5fea854d3326a27075713")
                .unwrap()
                .reverse()
                .0
                .as_ptr()
        }
        4096 => {
            UInt256::from_hex("000001f340d35fe89d1924de57ccbf63a7a09347835e6e4990ee2df12a4a67f9")
                .unwrap()
                .reverse()
                .0
                .as_ptr()
        }
        _ => UInt256::MIN.0.as_ptr(),
    }
}

unsafe extern "C" fn block_height_lookup_333(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> u32 {
    let h = UInt256(*(block_hash));
    let orig_s = h.clone().to_string();
    match orig_s.as_str() {
        "df111a9253e9f2f80d3de7a7274aaaa6d5ac0426b6a048991fc8ed7586000000" => 22030,
        "4a53ef88bad2a12b5af9767bcef10980955971826b5f75ef0f17518a73020000" => 21976,
        "ef176a4a5dbbb041f7e582e4adac44c81e7d1efdc92c2b532bcaa24106000000" => 21928,
        "1efab5ab1fe9d99f90072d7d377b0271511e16b2f833b3314576ae23e4030000" => 21880,
        "88d5f21114fddb86c8ab230f2b45fd90b8cfa7a61b0d819160fdb08345040000" => 21832,
        "5b298e3fcc475e57f20520601057e3686219a2bbe09b0baa34d9759268050000" => 21784,
        "0b06e90c32f0611bce49e4646213f0bb7a43c59b82cc5a87295bf4f15a060000" => 21984,
        "7b9b8fe254a17c31a2315171b8b224ccf2ddf7960d69a87c5400514d19050000" => 22008,
        "af5509f7e1a2c9827eac9487746dd6f363ac56d4659fdf0089f8bca54f020000" => 21936,
        "c970d99fa088f3a1e9bc9eb3b9a4d03263b755149ce39a6db5bdff6d23090000" => 21960,
        "4887de631c97d933c8a52026589ad02aa31ad2a9e6fd66c8b278aa34bd060000" => 21912,
        "99867fdccf5f7b50c648b1ad3c93c1fa7515d4529aa5efde1ad17e0780050000" => 21888,
        "2323ab4e88e0923e8e3bd506192dcaa4286c20340cc7525128665941ac050000" => 21840,
        "e30d315d4a40d82d059a65a900242f393b50ff8fefde58a4a1b482d232010000" => 21864,
        "c90366c6455c23930215ce724062fa1dd2ae89b66c722302dc9d56ba1d030000" => 21792,
        "dbe354545727c9db4cf872765455d1aff09296a14daa4075d18db6bb4a030000" => 21816,
        "4d523423dc19a438e57ee0e150c3f62369063a1bf99274083bd2f055fe020000" => 21768,
        "c244b2775b5fd5ec60d911f7b851ec4e78cf54f678f3d64cc69245fe60070000" => 21744,
        "36a179a25d081bc6836be48b11f10e6e484fd0ef068dd315a9e11482ea030000" => 21696,
        "cd36d310de9b14823aa9aedc00abbe2b5ac4117cb8d00f7693ad22af05060000" => 21720,
        _ => u32::MAX,
    }
}

unsafe extern "C" fn block_height_lookup_333_2(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> u32 {
    let h = UInt256(*(block_hash));
    let orig_s = h.reversed().to_string();
    match orig_s.as_str() {
        "0000086ccc192b07450acf188eba3d0360058dd43793fc622e0b7744cad5cbed" => 24869,
        "000003227cf2f83a1faa683ece5b875abeb555ebf1252f62cb28a96d459bcc11" => 24868,
        "000003bd988f78d4663780b467dc6155f5c1641c16a2d0a09cf62eeaaf9ffb39" => 24867,
        "0000091ffc39c9891a425b7eacb051c026d8e6ab4a86f074b6961f7c4a0ea87c" => 24866,
        "000008804a2cc065da4752c7f6f4b889c682749997bf24c704ab44e932ae0f6b" => 24865,
        "00000aa4e53ca2059b645787d4516dc75f1289708d2a2d3f6b103d171821647b" => 24864,
        "00003130c6ba235a12bf1057c459268ad10924a47d9b34eddc57858d77f72ddb" => 24863,
        "000000b5f4669ac2ebda3607fa627df62656650ee689a8c82705f340ec3d78ea" => 24862,
        "00000692fc267184a6ab60ed6af6a55cdf527773872c601f098569c254f2d189" => 24861,
        "000007991952ca7c32420cd92654ecc41d5716c191cc96cb8978ea3257a69992" => 24856,
        "0000037bbcd8e2af574fff36fd248d2058b3a9582c9a9c3fc6ab1dcee69c25e5" => 24808,
        "000004f78ebfe17c2e78c58ca1e754e02f3e52043bea00f63be7f2cd58d23c96" => 24760,
        "00000597b3da0bdb229b571bfcebb78a5786c9bc21d3fb3253d497b8a62e5d99" => 24712,
        "0000078bc1a0778aa6d8dbefde4f110eee570c1b945c553b9ed98368871d6b5b" => 24664,
        "0000062c4e24e761dc2d1b9b6d1588198b3f4ad8a767151ef6414bfe87ec94da" => 24816,
        "000000e3756fb54df8e63db7c69158a88247ae6fb951602fececa314820a644c" => 24817,
        "000000461e6f7b4d5656173bf188e83bee5f10d971616d946db3027cbecfa13f" => 24625,
        "0000037bd6d0b218eb906ab6206fde5a96f972ede0308bf76cac4190428a924d" => 24624,
        "000007756da9e48ef99829c467379e3a873ec9bf90652a9d10483329ef4e222c" => 24672,
        "0000018ed9cd191989b356447ad4d0649d5c471e1661a79aab2613251cf991a7" => 24673,
        "000004413ed62592b7a47b06a296ecb79dfcf5e65227a3e93713b36c48846f6a" => 24721,
        "0000050c3da50cc911506eb4bfb50e7b1f1ac0d165eb62f239a48545cc5834c1" => 24720,
        "0000010f2aabcceddff97f4f87987902f99629518dfdcdf5522e7e159b78a825" => 24769,
        "000000426666e40ff602a896a1493b765daa6fd6667f7c5edd0cfd7c23857750" => 24768,
        "000000b693fbd6d2a02da459158394220f83a4cdb947117246c83a65c9296bf1" => 24860,
        "00000683d6d9d7df1b530eecd973fd71aac2434687cdf94b02a959237b41b123" => 24859,
        "0000052e1a801d4313cd6a90beed7f478f34b778c2a8cb0e8b98dbcf83c5efbc" => 24858,
        "0000076bffe00bf97fa0baa71c1444f5e3270cd0ada2423f1c77858b95d19338" => 24857,
        "00000763996b273b8f8fd4b184e0175679bae5bd176d021efd71d4d598e0a184" => 24855,
        "00000088430da03d58df458cb0118dfb7a5583241718c8cbb1b02f7eee673463" => 24854,
        "000006fe4eb832348f5abbd7d7fe5653068dd20ca58e05bec321f486a9fe5eb1" => 24853,
        "0000018db2f6d303edcdd6ded823acc31bfa50f08389111b4f90ca4cc753006d" => 24852,
        "000002f575aa919a482ec157c9c3614057078bdf99da797305c9c7f549a9c677" => 24851,
        "000002dd2d2977ff6fa6765954919dd816b96671d4c8d5d0752f5e4efc48f8bf" => 24850,
        "000004574333f4fcbbf989299a298c53b89718aa10c169c32db07c18c14a33ad" => 24849,
        "0000053f364c7ff258ff87185de4c789420372d9bd76f37dbdc57d5b3d1992c0" => 24848,
        "000001e2680c8f337c3a6d81f7b72e67836dd64a6cbbaf13875a0a72976282e3" => 24847,
        "0000030d4554e01211b54c9041f7a257ec83f9079a1913eda923bdad11f08670" => 24846,
        "0000045348e3cef51ae93a68a09e56eab3a08f339ab62b196709f83bc97c6936" => 24845,
        "0000040777831dcff5ffb3e3c0bd38a008a6d4456c06e320386004fa68a49e7e" => 24844,
        "0000027e119d72c5dda56cf39223b35f232092495996345be248eeb48c9f9130" => 24843,
        "000000baa4a33d65e11107122a92601053ab4e716a6d661dfd6bb9f874188413" => 24842,
        "00000645381ff1106e135a9d5df3cec8edfafce71c182325d4b75099d4002c4d" => 24841,
        "00000056c854b1996beec75c464693bed7a958e9b67d71d701340b8ff6d9e054" => 24840,
        "0000027aa1d0102ddacd2f9dde2681ceaf0b5c0caa806cb7f816c119051464e6" => 24839,
        "000006437989e53e813df1a8aa388096387ee5de9cd0b045779a703582e7949e" => 24838,
        "00004cbd92d058ad559d6cbb2d973e3e199015826eeab9691cb718ba349b0f70" => 24837,
        "000000885cf1eaa6b419152fc651e0119d57281f18a6ae3806dbd4fcfb5cad25" => 24836,
        "0000049f89a5dac8c9593c601f9a385440ee050abe8cf5f039a3d59c58f06249" => 24835,
        "00000104885a2403d15e65e25daf9f78c755c2953945914f58e6328c9dd50c4d" => 24834,
        "00000795888b51da4cfe848adfcdff19cfc815c1865c58e2200ea9ad596df9e9" => 24833,
        "000004169e45af9fd4ddaa64c400758df7cf36b0c1a22f2b9f29a11fdea70eba" => 24832,
        "00000743fc8bde3da9911bbd5275e2e2a68afed8ce1d56b73ed8557528a4fb45" => 24831,
        "000003e8049e2baee08e173a04106df0c4b0f858f8ae1918caf970eb10d2046a" => 24830,
        "00000169672fae75e70f45a7031bffbdc92fb81d183d0324f39eac9268d11ceb" => 24829,
        "0000082b9da7946ea66dfa9c6ca9765a41219a049129248e4c5656f9bdd8aab6" => 24828,
        "000001249c80a3899d72096a03195ebfb96b27118559094e149fcdee18017458" => 24827,
        "0000002d7fc5efaf7d8799e6418f2cf0eda1e9dabc9771366ea66c5e09dae7cf" => 24826,
        "0000051ee159b9aa029497f509e3c92bfa03737974444d04bbc940cc8a901bab" => 24825,
        "000008076d72be61be309f5a8e531eb5a85405f4ee75125a1b3556774b5848d2" => 24824,
        "000006cba36bb9086be9ac34b03d0bb2379ccbf9af7cd17da4f2450dc9f7006c" => 24823,
        "000004c5b1b7e1bdf532e1318154cf2ebe387b5b9ded7c748b75856003a43371" => 24822,
        "0000066acb91115887007ecbd8d82eb4f4c32c0270552fe8b2f32e48213575f9" => 24821,
        "0000006dc4fc61cdef4df41ee9b7f49fe17eff54a66f3a3a88bed959388a5434" => 24820,
        "00000178548ba2f34bd307781683b0f97ac31ccdd9b1b22e148c59211567c407" => 24819,
        "000005e3612dd20b3bce4ea865eb39045fff1f8937301f9a1db3654d1a98ff80" => 24818,
        "000008ab4c5a522743760f8ef14089c352b0f7b1013cd7ba7392c96e497afe85" => 24815,
        "0000049c9e3d99ae33ce5d71e7e3974d8be8d87e65f66f5d36f200f482eb89a3" => 24814,
        "000006799bc165a2d527f3f980c919863cd3a78dcf3fd201f57bdd6c27ff7af2" => 24813,
        "000008822d1b4327327d2477ed48157ab739f5631451f59e669f65da6b4d99d0" => 24812,
        "000005cbfa02e404973f41eccae792d8394f35089a39ae4f4242e61fc3110dc4" => 24811,
        "000000c25bf5d29876879fd5664efecf5dbe6b889756b0413a7e9a7e14b63894" => 24810,
        "000001c6e2988ef3d42b7fe817481eded236109b1ecab23bde43710cd0b82890" => 24809,
        "00000043a23d0bbcd576eb8c39b194e9ee5364ed47645dd4917bb45d06768777" => 24807,
        "0000002c7dabf7c52be36c77728fc2a1111f0caa9d167e4930c954540c3aa142" => 24806,
        "00000213354b423e60b5e4e3da96a57658fae6451078b4301eb7fc96d286c346" => 24805,
        "000006acdb4419d876c00a195f16edf1b5732ef6c26e651ae9f2f3c2b8964a3a" => 24804,
        "000002a9757ea470f4d6341cec13cc12ae8e7940bc401b36b22a1a44d39cd7ec" => 24803,
        "0000042837f5359d0b4d72b88a90e782afe782afbec03fbe54c7f06ef91e0fd4" => 24802,
        "000003de21b33156e7d94b1408fc9ef753f7cc9bfce52fe6ef2def3c2184a00a" => 24801,
        "000000f63595d3c80a44ff104b19828bd7f337027dc064cd179e2eb5e5e8db56" => 24800,
        "000006ac1f34ea47d2db12fa1c8dd765190ab4c5c8329baf12c6a8ba5fc42dbb" => 24799,
        "00000401fc8d3cfd75086cc67b73a32d7978877e1726ad9949bbe67dc4d46840" => 24798,
        "0000027b6ceb8c45d444005a6b948ced264c8fc39fb412dc69389ac63fee668e" => 24797,
        "000003f23b070d2734940dcc2682517883b6f5d3189c34b5dff4742153ec64ab" => 24796,
        "000002640f8273af03475f6ee496f80700b12980bc4fb059f734e6183377b078" => 24795,
        "00000005cb3272a005d40b45caa1cdeaba2651d619cbd6196f2c6c6317c8917d" => 24794,
        "00002b2c169c551e051fe563eec39734ef89eedee4d65f8a2c0074280418c8fb" => 24793,
        "000001f0718c066dd865f72836254d6e9b217f49ed4e08b89bb8e7ff3b3aa6af" => 24792,
        "00000156030a46774d864bc346009cdafb47b9ea30a1ddfb5ad5ef5bcc8e0cf9" => 24791,
        "000003799100d36e4f1eae601ee81df2f02ffeba59d3f29a7db43c2b7568d481" => 24790,
        "0000047213ccf1910fe90f8b8ced3e8a2e3a9c1bc93ebf4103f250dde548e470" => 24789,
        "000003e3515c438775628e49dd61701a8327005566c2b88407d9afb59244092c" => 24788,
        "000002563aa47ebb2189f33735f698231161c5df29627e36c01dcbf0a6a2315e" => 24787,
        "000003c7f4624196872c229be62947dd0004e09571fb1b8c0625fda088873e5c" => 24786,
        "000002c6cc3fc915987a1c89f295e969db682abad30b7143c66df3d020c81aa6" => 24785,
        "000001a0000cf3979494e1451bbb2e5b8e5364c003d69b07e33fc9a4a42afe49" => 24784,
        "0000051f34bb857fb33424cb780e066cfdd565c64714048d42b84d2ce64e0b87" => 24783,
        "0000046900a9d9478e84fa09702fa98e734ade9416f43229969c03d499e2b827" => 24782,
        "0000020466a6a3f34b7edf6e7a56950992eb43e4f46babce3c8e7a1f027932cd" => 24781,
        "0000008ea4a9398eeaa5ee996129f6d58f3ee6eb36ad48b39dcfafd491b72698" => 24780,
        "0000023b916ba5f3cc323065f8673c6648cae692a97cb84ae052d372e2ad1fdc" => 24779,
        "00000128469a850ff260fab8d71cec4ae08953f53adc769b0e4510c64cd5b1ae" => 24778,
        "00000329f84bfad3889f8e37a6e5f532547a73840428b36a875423aa3f781922" => 24777,
        "00000039433505249e61dcea66c0f3075ac384cf216970149d2e7f149f031ac7" => 24776,
        "000003c944684f4df6d0e9df65bd0ff45db24279400c977ad1ddd095d3d26a20" => 24775,
        "0000050ef9d0c1e6e6d1506dfc9a93c87b49adc40d3f266f168726a987cbb16a" => 24774,
        "000000457cb432ced5fb2ea78d36ccb2744d22c1a4aace36ecd8e603325282fc" => 24773,
        "000004c958970f8db6169284b8fc0a48624964e38a1da4260a528855c549ffd0" => 24772,
        "0000028a421d2f116e56ac5da67217e942fdc5d783d05f2b0ec08aaa98a4af1c" => 24771,
        "000001b392b1b5e29cb334a6c7d9baf85e8468fc9d5ac2860c2ed4dfa8affb3d" => 24770,
        "000003a2b3a849971822edc9505114e61704a58db8ebae14c78bd0fa9184f519" => 24767,
        "00000081a650f7953b6f6e704b14b142e06972a3fdd86ebf5327820adce8a6f4" => 24766,
        "0000004b32f5cc9512ca0bd2f96f84a29b975c879ea050c7309c9321cd76d67e" => 24765,
        "000003367801e71b6bfe4ca3e277603f2f9091caf80be7bd697b2e7d8b2fa091" => 24764,
        "00000555ea65e4120624c4c8699af9ea3c91a3f8e42095192397fbdf6774f02a" => 24763,
        "0000016c4f0f1cb3ebd8f2b2adb53347a3c69fae7c0854b4ea41a1d4c8af7d6c" => 24762,
        "0000028655333153a9e4262170b8acfda4a2dec64dd387b2033dc0a075eed242" => 24761,
        "000001c7704486626aec884fe851b5b54326825dd56d56d09f03beb2a576ae09" => 24759,
        "000004db5d642d4a869ce8100ac5154658c06b9ef0b9e6b9957139529884b79c" => 24758,
        "00000495b214d71655cc24f56bf9aab7d2ab286857e6f9890f1ae639c357b69e" => 24757,
        "0000029d66a078af9d234a430f0cf8580ee0aac264fc91e25b6e379836f64aa2" => 24756,
        "00000319ef4c483b432a52e103a5fbaf72cb2cb03238793982455386f223b5f4" => 24755,
        "000000a5cd94be2be2764cb5e3eebb1c4aa24b0bab289ba4bdf032cb5c4511a4" => 24754,
        "000003a4acaf2de030463272ce372a6d38973bc4d3ec9abaecd88fb242095c6b" => 24753,
        "000000b13d3a5e352c02e26fbbc0c5a19c630ffaba54177c3074b08311c334a5" => 24752,
        "00000306d57c64e4ea982061c855b4ec7830836e9779d45460d96b7bbeb58ee9" => 24751,
        "0000046a30db2347c925c399ab3d08e95f134545a1b4f559c970db585b7a2a2b" => 24750,
        "00000002a9013220364222bb38dbfb6fa1a53177524d9e0de1d243b34ebfc136" => 24749,
        "0000004c90e593866678aa4cba5211019e02e922ff7180fa3f152b20aeee589e" => 24748,
        "0000046d5e84be89842cd2f74a3d72f3299e63179409c7c14757b7d66ded1132" => 24747,
        "0000024082c1607e22f1300d5df14ae43a7b41ed747c0a8ee4b6bbcccab493d9" => 24746,
        "000006aa0077df5557c493d7eb81396e7ab10302a09d9fb2021d1eecc0077b08" => 24745,
        "00000149b4e0da935b7aa97f720ef7e6d49e828880e20d7fcc368923176a8968" => 24744,
        "000000b401b4b4a5174cec59765cbb23fe491ca4b7ae1521ddb32f3425860de2" => 24743,
        "0000056f246e61d0ce87af62a18bde093a00f782b339bb5708fb75abc3c1d3db" => 24742,
        "0000057c5bd82a630463ee1a6cc591c7b52b352c603d3d8909f6b4fc699fdd2b" => 24741,
        "000006f2286327b6a2a1270f770609ff027d02180d69d2ad18b63566dfc0515a" => 24740,
        "000003611f378cd85a09df3bed09f29bb69923a73584ec70830ffdf4d9b25285" => 24739,
        "00000018c1d055316dd10aaba200a40e4b22bfaa2aa4fe946aad9c48dd265a7f" => 24738,
        "000005b7c527392bcfbdd75b06f0906fe85ec2ee8fe8c1fc08a4f7820e091f90" => 24737,
        "00000483b14397ee07d6a6e0ce4caca153c46e24ff9793a472b80be780fde4af" => 24736,
        "000001531ce19a3042757887098225c468a56fe159729d40d9a20e764875a0e2" => 24735,
        "0000002f47f9cce781cee4169fe7e1a3966fafbe0375f6844d32b1337a91b4a6" => 24734,
        "000004e5ea1774456c0c0225dd1bdd8ad0007f4552df456d5148243d6a38359f" => 24733,
        "0000022ee8c41b97ed57f95887d5942bdef91728abc8e5aba6ee8fd3b74ab4e3" => 24732,
        "0000034aceca60ff1c4136b3d3aba4a875d47b0cb481797b0dbeef9f1125485b" => 24731,
        "00000798b049b80fc751a9d02ce80872dbe54b3798123e47a7726be03a24013f" => 24730,
        "000000f92ebd1f266b370a27cc8f3426f78153ab3fff492b2088eb92ea6a6243" => 24729,
        "000000d4bfaef03c58066d660a30950705376f55fc9430fda897f1c340c149b9" => 24728,
        "00000123d0cab60bc8bc0f7748d040090429a3570a55b77e11b032d90578fa95" => 24727,
        "000003258e7acb7c9f8e2d5f8f802a6418dd819388bbc1463360ad9f9505034f" => 24726,
        "00000541b0635b6b9bbd8921d4a8fb12cb6633323cda8cac7a84c32dcc79cbdf" => 24725,
        "00000605bee6c4084f4bc1b8e0a0b4565513085df4888fbabb1ff59402cdfe60" => 24724,
        "000000a42f0395faa6036ec8dbb6f40618d74fbc35b2c343bf03f0d5ab1f108b" => 24723,
        "00000535556772e86ca1210083a4cb4db74454f4cef914fcc719cdd7d17d8c54" => 24722,
        "000005e33e11c29db88a6a59520395c6d74ab06b168434171d51695d32d447e1" => 24719,
        "000005f2268271f37aad0fe7cfb9ce5069d4ea9178d999b3488942c0152dffcb" => 24718,
        "00000470f83644a5c08132629c03807cc6682846e2aa5b39959d5f3cd12cf48f" => 24717,
        "000000e70d9b8d5d9b8a6af9b7daa957df3d36f30edf1ef5a59da62eb10e17ae" => 24716,
        "0000065d0abb85677e6ab4af484d80f7e90a3cc01fc46c555e3ac8659644d844" => 24715,
        "000001f6cd40684b33b496fc574facfd81790681ee36088387a364dccb48def3" => 24714,
        "0000049522e2a80117de535914d67c7e8904bfc0bbfdca4702d56939c06a8cd1" => 24713,
        "0000012e65c0ac8c90b83d42a532b4a7c3774c0cdedfdb74ba64d1637ca4a7fe" => 24711,
        "0000007f70b2ea36d666379e731f7ecff79db6ec9838d39d24af693756608242" => 24710,
        "00000098ef907dedce9eaf2132cf67545a0f746e6316c698f35bcefbc5ead111" => 24709,
        "00000672106963c28e4ae06d0e0491bdaf1aabd4ea3a0759a65261e231d6aa2c" => 24708,
        "000000fa70d8d7cba9ebbf96c297e22cebd58ab8d7e1da8fe801606701237610" => 24707,
        "000005b33952dfbb9e9c936503c1f474df369bb7807adf95909b37d316e9052d" => 24706,
        "0000028acaa3b4fcc3eb0e9295a3a9a0b395ef464507a9af5bc51cd2b04b38e5" => 24705,
        "00000553a90aab22049b035f06cd4740d4c283c3b7daef5bfe8246b776e5d0a6" => 24704,
        "00000015443346b5aaaf59fb71694428399183a0da3d23a36ba62ef6f868d65b" => 24703,
        "00000524346a800cb8b841e79a429803f06daa25dda27c4e15529711dd026485" => 24702,
        "000001ade388c5280e9c0abc025d860f4b134d78f5f894910966ef58d79eb6c3" => 24701,
        "000004970192dd41d3afe3c809b8cb150c8210a6cd41360ecc504f8e765a14b5" => 24700,
        "000002be5a953f7fed9e4be3361df036a07676fff669f3fae721c1cfdeaa12ba" => 24699,
        "000005ec3a5ed58b575c0dcea597b0bae984f9642cdbb745c2a4acf92b1228e8" => 24698,
        "000004416dc20703d3aa5f017d7cfb96f1f030c06dafad3b0c45414e7b84847c" => 24697,
        "000005cf0f4caba6b236341afb3e7f8beacb8018364136c198e6db485f131e14" => 24696,
        "00000487243de854c402ef3026822e4b33cb23f3526275e931d00589697c7b11" => 24695,
        "0000040fd0c77eb035c04daa054de7df0a330bdd767a2b9f40929fff4b631ca9" => 24694,
        "00000212c4234717534cf9b4351dcde432eebeeac2231cd04d1a1f470710e72d" => 24693,
        "0000037372e681ae0013faa6f509043940f39345f6fc2c9d69bfb26a92d4d3a7" => 24692,
        "0000000c03ffa98e5ba1c1fe1441c4812a91746aefda9b6d150ed08242d4821b" => 24691,
        "000000602033c5a0c32421f3adcb3de7189931a633e98047b38ba6717f242740" => 24690,
        "00000362c6b66c7054dbdbab81c79e263472b1cea6b0a3e455716fcf328f8e58" => 24689,
        "0000002fbb3e6eb53ec3174f2071db442dac5ffe82794cc5b0498fbac080f8da" => 24688,
        "000002177c627bdcaf2e95791166c9641bf02fdd5cc1d4a85ca20aa25acdd107" => 24687,
        "000001019891cb0eb860698cca78b8a17fae795c0798c437a0989896bb02c353" => 24686,
        "0000013f4ea9f6aebc88a59d1fa8631f46eff2e18e54b4ae9d284d1e640d7fea" => 24685,
        "000006c99c58dbec27e6435cd4a1552007c55afec8af797c1435196a3c622aee" => 24684,
        "000007e05572b439fc0a21ce64981b82669d34d73144344b80e2fccbaae85687" => 24683,
        "000001be22b6a741824b01cca475101002e0f1457f024d439ea5ee0a1aada27f" => 24682,
        "0000001072bcf507e848486e5ff320c1522f887b158ba6e4303b3ceddd4762a8" => 24681,
        "0000035731fd748b408aed499c9c4b45628e9338270a96fabcfe763f7b6e54ed" => 24680,
        "000006501d187ea8b178cd4b6d0aff146dd478609487bd889a693ebe76d27b09" => 24679,
        "000000b48edc5982b7986c89ddb7a3046fce5418a3bff0c40b9c208b768dcc90" => 24678,
        "0000040798a6b4632f2a9dd0f2a07523affae1bb4ab547598bb972d76fa19c8c" => 24677,
        "000003a5ef3d13b8f452e41fe4a45156d512c295d094e6ea352543b2280929b4" => 24676,
        "0000068ad662037d30811d86dac5b50ac0ee104afa0775b7b5be6e89ace77929" => 24675,
        "000001fcac0d415ee0fec9b164a5c0d3baf7fcc01cefd8f55ae1cbaf4edc557a" => 24674,
        "00000439528d740502cfb49d869ea9e24d2a13ec9b4dbec666923aed9a74d52f" => 24671,
        "000004404897d33d867d3804df92ceffaccddf2fbde381800461ea3b88d89d30" => 24670,
        "000009b14dec4bc7ba636de6ae25db8bde066f132b2fa96d9c4fa29ed1b6b82f" => 24669,
        "000005c6ec5a786816157f38b3e721529f6bac1d4a9e20bbc6119d1f44b38ddf" => 24668,
        "0000057e45e1a7176be2b0489956d83a1044e5e842097e353cd5ebae7f1a51d3" => 24667,
        "0000022e51809198d0ecf3fc49eb4706f9cd09b6ade82c9b87923cda2e5301fa" => 24666,
        "00000854e9d76d2507226e86af114dc3572206011d850843a40c1d3fea0860fc" => 24665,
        // "000008ca1832a4baf228eb1553c03d3a2c8e02399550dd6ea8d65cec3ef23d2e" => 0,
        "000009ab6ebbf37176dd95a82f3f66d9ef24e8108dfe7874fdf2adee3dcab7c9" => 0,
        _ => u32::MAX,
    }
}

unsafe extern "C" fn get_merkle_root_by_hash_default_333(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> *mut u8 {
    boxed(UInt256::from_hex("0df2b5537f108386f42acbd9f7b5aa5dfab907b83c0212c7074e1209f2d78ddf")
        .unwrap().reverse()
        .0) as * mut _
}

#[test]
fn test_devnet_333() {
    let bytes = message_from_file("QRINFO_1_21976.dat");
    let chain = ChainType::DevNet(DevnetType::Devnet333);
    let cache = unsafe { &mut *processor_create_cache() };
    let context = &mut (FFIContext {
        chain,
        is_dip_0024: true,
        cache,
        blocks: vec![]
    }) as *mut _ as *mut std::ffi::c_void;
    let processor = unsafe {
        register_processor(
            get_merkle_root_by_hash_default_333,
            block_height_lookup_333,
            get_block_hash_by_height_default,
            get_llmq_snapshot_by_block_hash_default,
            save_llmq_snapshot_default,
            get_masternode_list_by_block_hash_default,
            masternode_list_save_default,
            masternode_list_destroy_default,
            add_insight_lookup_default,
            hash_destroy_default,
            snapshot_destroy_default,
            should_process_diff_with_range_default,
        )
    };
    let result = unsafe { process_qrinfo_from_message(
        bytes.as_ptr(),
        bytes.len(),
        chain,
        false,
        false,
        true,
        70221,
        processor,
        cache,
        context,
    )};
}

#[test]
fn test_processor_devnet_333() {
    let chain = ChainType::DevNet(DevnetType::Devnet333);
    let processor = unsafe {
        register_processor(
            get_merkle_root_by_hash_default_333,
            block_height_lookup_333,
            get_block_hash_by_height_default,
            get_llmq_snapshot_by_block_hash_default,
            save_llmq_snapshot_default,
            get_masternode_list_by_block_hash_default,
            masternode_list_save_default,
            masternode_list_destroy_default,
            add_insight_lookup_default,
            hash_destroy_default,
            snapshot_destroy_default,
            should_process_diff_with_range_default,
        )
    };
    let cache = unsafe { &mut *processor_create_cache() };
    let bytes = message_from_file("QRINFO_1_21976.dat");
    let context = &mut (FFIContext {
        chain,
        is_dip_0024: true,
        cache,
        blocks: vec![]
    }) as *mut _ as *mut std::ffi::c_void;

    let result = unsafe { process_qrinfo_from_message(
        bytes.as_ptr(),
        bytes.len(),
        chain,
        false,
        false,
        true,
        70221,
        processor,
        cache,
        context,
    )};
}
// #[test]
// fn test_processor_devnet_manual() {
//     let chain = ChainType::DevNet;
//     let processor = unsafe {
//         register_processor(
//             get_merkle_root_by_hash_default_333,
//             block_height_lookup_333,
//             get_block_hash_by_height_default,
//             get_llmq_snapshot_by_block_hash_default,
//             save_llmq_snapshot_default,
//             get_masternode_list_by_block_hash_default,
//             masternode_list_save_default,
//             masternode_list_destroy_default,
//             add_insight_lookup_default,
//             should_process_llmq_of_type,
//             hash_destroy_default,
//             snapshot_destroy_default,
//             should_process_diff_with_range_default,
//             send_error_default,
//         )
//     };
//     let cache = unsafe { processor_create_cache() };
//     let context = &mut (FFIContext { chain, cache: MasternodeProcessorCache::default() }) as *mut _ as *mut std::ffi::c_void;
//     let creation_height = 1008;
//
//     let tip_pro_reg_tx_hash_1 = UInt256::from_hex("663b2fb8bb620db387f7268ccdf261d0739b3b734080487736560536f9da67c0").unwrap();
//     let tip_pro_reg_tx_hash_2 = UInt256::from_hex("beba26ecf8a70d13898cdf463dc2b29b3111999d9ad7bb3b1bd270c38ec64a46").unwrap();
//     let tip_pro_reg_tx_hash_3 = UInt256::from_hex("67eb2b533192cd354e9571cb361c5fb2f3a40847aed1b0983a18164f650542d4").unwrap();
//     let tip_pro_reg_tx_hash_4 = UInt256::from_hex("f4bb17991ade54751bae0d192102ffa1fe9428085f459acf22a34f061e5b2a18").unwrap();
//
//     let tip_llmq_hash_1 = UInt256::from_hex("0e66f9273a2de1905244de60faf55bb6ec795cc6c66607f2c0cf9264e224d02d").unwrap();
//     let tip_llmq_hash_2 = UInt256::from_hex("6a8908985dac8faaf2edeeb5fc581ca8cfd13ab8a0d054283b10241d9cd74ee7").unwrap();
//     let tip_llmq_hash_3 = UInt256::from_hex("0e66f9273a2de1905244de60faf55bb6ec795cc6c66607f2c0cf9264e224d02d").unwrap();
//     let tip_llmq_hash_4 = UInt256::from_hex("0e66f9273a2de1905244de60faf55bb6ec795cc6c66607f2c0cf9264e224d02d").unwrap();
//
//     let info = types::QRInfo {
//         snapshot_at_h_c: boxed(types::LLMQSnapshot::from_data(vec![1, 1, 1, 1, 1], vec![1, 1], LLMQSnapshotSkipMode::SkipFirst)),
//         snapshot_at_h_2c: boxed(types::LLMQSnapshot::from_data(vec![0, 1, 1, 1, 1], vec![], LLMQSnapshotSkipMode::NoSkipping)),
//         snapshot_at_h_3c: boxed(types::LLMQSnapshot::from_data(vec![0, 0, 0, 1, 1], vec![], LLMQSnapshotSkipMode::NoSkipping)),
//         snapshot_at_h_4c: null_mut(),
//         mn_list_diff_tip: boxed(llmq::MNListDiff {
//             base_block_hash: UInt256::from_hex("663b2fb8bb620db387f7268ccdf261d0739b3b734080487736560536f9da67c0").unwrap(),
//             block_hash: UInt256::from_hex("5fcef4606b48e92b611df852b12974e549fc385a7097d2370a7bd0bad8cbb055").unwrap(),
//             total_transactions: 1,
//             merkle_hashes: VarArray::<UInt256>::from_bytes(b"010e45d82414995ed1c23f546c35afa316499c09b870dd0a3c15796c4ccfdc00c4", &mut 0).unwrap(),
//             merkle_flags: b"01",
//             merkle_flags_count: 1,
//             coinbase_transaction: tx::CoinbaseTransaction::from_bytes("03000500010000000000000000000000000000000000000000000000000000000000000000ffffffff050205040101ffffffff038d0cb75b0300000023210285c760cb2fd04fc7ff217cfd1c66594ba47247c29eed85949f43e80f57b53727acd94e854a030000001976a91471d69c816b5ad8718c800607fef3a47221078d6088acb0bd3111000000001976a914a73955c08d561a22a399513e1c5d3983d110701d88ac000000004602000504000042696f1f2db709cde94efa6f8de3e5c5ffda082fcb3f9d81b5929849385667bd3394b4b77e40afd081094fb55b49a94f39bc6910146b5010b9d6af082f15545a".as_bytes(), &mut 0).unwrap(),
//             deleted_masternode_hashes: Vec::new(),
//             added_or_modified_masternodes: BTreeMap::from([
//                 (tip_pro_reg_tx_hash_1, MasternodeEntry::new(
//                     tip_pro_reg_tx_hash_1.clone(),
//                     UInt256::from_hex("5fcef4606b48e92b611df852b12974e549fc385a7097d2370a7bd0bad8cbb055").unwrap(),
//                     SocketAddress { ip_address: UInt128::from_hex("127.0.0.1").unwrap(), port: 13998 },
//                     UInt160::from_bytes(base58::from("yhedxEwiZ162jKCd3WpvWgWWocDiciJuKk").unwrap().borrow(), &mut 0).unwrap(),
//                     UInt384::from_hex("8c3a4249f6e1597ac13fce64b91361ebf6d0837d5a95736549b88826868c34c7c8ede2da665e2702708fc431c9eb231b").unwrap(),
//                     1u8
//                 )), (
//                     tip_pro_reg_tx_hash_2, MasternodeEntry::new(
//                         tip_pro_reg_tx_hash_2,
//                         UInt256::from_hex("63f2bb5920d1a0c27c9689d62c6834a314584fcd7b2bf52388c1dcc2c987f2ec").unwrap(),
//                         SocketAddress { ip_address: UInt128::from_hex("127.0.0.1").unwrap(), port: 13999 },
//                         UInt160::from_bytes(base58::from("yZLegVnDt5t4KZAiXiH2M88LbvkHxRnXL5").unwrap().borrow(), &mut 0).unwrap(),
//                         UInt384::from_hex("1185482390215003acac18979f4090b5cb4f2a7abd54a0e25b676d681bb6b53853488a74a014863403eab95b47afa017").unwrap(),
//                         1u8
//                     )), (
//                     tip_pro_reg_tx_hash_3, MasternodeEntry::new(
//                         tip_pro_reg_tx_hash_3,
//                         UInt256::from_hex("17ac84060b137d5e2c19bc4f2fa030fd4c49aedc77c64380251d7c22bf78e560").unwrap(),
//                         SocketAddress { ip_address: UInt128::from_hex("127.0.0.1").unwrap(), port: 13995 },
//                         UInt160::from_bytes(base58::from("ygo4ZEACuXGWygecqexNvLR2ryPV6LJBXh").unwrap().borrow(), &mut 0).unwrap(),
//                         UInt384::from_hex("1904bd1b479ff9fb5d99996575e4bdad5fefad11adf58de8e75d1b6e4964adf8b55d9e4f9432d56df6bef67f2ad68cda").unwrap(),
//                         1u8
//                     )), (
//                     tip_pro_reg_tx_hash_4, MasternodeEntry::new(
//                         tip_pro_reg_tx_hash_4,
//                         UInt256::from_hex("50cc1bc3de661f923afd38e608a5b4fbd1b608cba85d050e7380cbbbe4fc41ff").unwrap(),
//                         SocketAddress { ip_address: UInt128::from_hex("127.0.0.1").unwrap(), port: 13997 },
//                         UInt160::from_bytes(base58::from("yiyRqpgyVXTw3SGyWMVNeWS5YMNC56MMnW").unwrap().borrow(), &mut 0).unwrap(),
//                         UInt384::from_hex("0579dccf1de4e4bbf5f69bd1ccf5df2dd327d32b49e3789ec823858bb4fc3fef32c214461ca16151e7c5176573291429").unwrap(),
//                         1u8
//                     ))]),
//             deleted_quorums: BTreeMap::new(),
//             added_quorums: BTreeMap::from([
//                 (LLMQType::LlmqtypeTest, BTreeMap::from([
//                     (tip_llmq_hash_1,
//                      LLMQEntry::new(
//                          LLMQ_INDEXED_VERSION,
//                          LLMQType::LlmqtypeTest,
//                          tip_llmq_hash_1,
//                          Some(0),
//                          VarInt(4),
//                          VarInt(4),
//                          b"0f".to_vec(),
//                          b"0f".to_vec(),
//                          UInt384::from_hex("0f10444bd28d6a0993224baef8ad9b5c6fcdb246b268ef36928e9b9594e1e0e8679b867a16914c55d129007b07169767").unwrap(),
//                          UInt256::from_hex("a689bbe717b3e6ad6ea82d56dcd39cac0fc59b78849277519916be7422e2656c").unwrap(),
//                          UInt768::from_hex("0ab9c10a55e71608c4adb77e7fcf3aa060d26ee243c8bee3fa0a5af04e79a628974444a68c3bdc289d5a4141c7e5ba5c15a3f0f40c4f16dd4cdb2cb42223ecd842a238c9937ce6c5b065dfebc35fd4a0c64aac0358a1095cfae7c3fedf6063eb").unwrap(),
//                          UInt768::from_hex("10a116918d6d5e44732178c68076cc640b884ea91a83072b4205056af2682c6ee65b32d7f49bd38e349cdbd5b2292f3904d1e1c4a17e45bd94d4d1d5c89a9eaade467dd208d8878cb4fed3afb8b2eada4b5cd10ba65c9c8356c5bc54fdd22834").unwrap(),
//                      )),
//                     (tip_llmq_hash_2,
//                      LLMQEntry::new(
//                          LLMQ_INDEXED_VERSION,
//                          LLMQType::LlmqtypeTest,
//                          tip_llmq_hash_2,
//                          Some(1),
//                          VarInt(4),
//                          VarInt(4),
//                          b"0f".to_vec(),
//                          b"0f".to_vec(),
//                          UInt384::from_hex("01debc67e536f44b62b0558b6c3a15e7b0ea96b31b284875a9ca8d29ed7b214a75586095bdaf587daac6db781341512c").unwrap(),
//                          UInt256::from_hex("1283ed47e65a4a6b8696baee4e58982db52f2cc8a2b9f49ae45fdad31f6d5e8f").unwrap(),
//                          UInt768::from_hex("0b0f242f2c157df7a6b86d6de735088c230f654787cb45a23d299e9b9adb17b2382e14160f4181108198a3efe50ca3351298186b2be82c622ffda6c2cfde2007c080db44ef97e088b1127f3286f7220c1ed87993d6f186a5085764cc911a4c48").unwrap(),
//                          UInt768::from_hex("82035bfe76e6ddf3a88d141512e5624ab3cd04596d7afaf928352e76d55bb819a5183a9aeb9a604bb23a40f371dc34f500c2738f4e3147c909d8ba384ce26c13b5695fca04e16e59064b2e53632fd8314a08bc81b49a7555eeda5cdc79f71a7c").unwrap(),
//                      ))
//                 ])),
//                 (LLMQType::LlmqtypeTestV17, BTreeMap::from([
//                     (tip_llmq_hash_3,
//                      LLMQEntry::new(
//                          LLMQ_DEFAULT_VERSION,
//                          LLMQType::LlmqtypeTestV17,
//                          tip_llmq_hash_3,
//                          None,
//                          VarInt(3),
//                          VarInt(3),
//                          b"07".to_vec(),
//                          b"07".to_vec(),
//                          UInt384::from_hex("05a55e3fea0341d7f1985a1d5cefa20536bbe56df49c3bddff19ba8ec23ae20956cd1bf4718c6c33b8901583de3b5455").unwrap(),
//                          UInt256::from_hex("bb78ee19643d2bc92334e4d630edab7fde7be90549525121b6db97ecace5524a").unwrap(),
//                          UInt768::from_hex("94a41ea4095cd7d4ca5db33018eb2df3abb9f51ea74653db1dc2832a45f683d96cd0cd209458ece30c6f6b908c53687a0e428b6763db9c65bcc753509fb919e4643d52c759ff8285724840677be35e7db3f9981bb93c36ba41f51afeb386a511").unwrap(),
//                          UInt768::from_hex("89befef3e3c509192d843c426df9c7316c6cb63ec4bc2c50744d0ac33a769283df7b645a965c3dba9e008b258bbe8bb7016774fc81d31e85ac34cb844a8c572dc6b2d6576b65df0fb09114c964c315fc66ea22b1911ed26fb82d39b9635b18b9").unwrap(),
//                      )),
//                     (tip_llmq_hash_4,
//                      LLMQEntry::new(
//                          LLMQ_DEFAULT_VERSION,
//                          LLMQType::LlmqtypeTestV17,
//                          tip_llmq_hash_4,
//                          None,
//                          VarInt(3),
//                          VarInt(3),
//                          b"07".to_vec(),
//                          b"07".to_vec(),
//                          UInt384::from_hex("92470c6e0268b1cd8f326e673c952fcbb5b0dd8f7f28dd0bb4a7d14023f69bab475f2aff3930b00bc040c211e0363272").unwrap(),
//                          UInt256::from_hex("c82cbf83d68d2a9598ebd14798d7544485659884dad31d0305c9e01969031769").unwrap(),
//                          UInt768::from_hex("94c3181a766833b06ce5b4d4c83534da6665974f81e2d9fe6e4eb644ea9c033fcaf62cdc244a82a8453261043cb55f4006d62f310dfe918c292663a2abb19df253ddf67db2c4df126edaa2b21a61dbc9af8fc0a36fce55b8bf14a57f7edb60ca").unwrap(),
//                          UInt768::from_hex("94be970450945d578263d08ab524d995e253db865d7195671f2c48a0baff7d2b6c9f17f79abf8247244459c60104d8da0efe78a709db8e85e1047fa620365bcf31f9f488aaf27a46cf1ba33d768cf6ea66ee02777ef0b42b2d92e16011517438").unwrap(),
//                      ))
//                 ]))
//             ]),
//             block_height: creation_height
//         }.encode()),
//         mn_list_diff_at_h: null_mut(),
//         mn_list_diff_at_h_c: null_mut(),
//         mn_list_diff_at_h_2c: null_mut(),
//         mn_list_diff_at_h_3c: null_mut(),
//         mn_list_diff_at_h_4c: null_mut(),
//         extra_share: false,
//         last_quorum_per_index: null_mut(),
//         last_quorum_per_index_count: 0,
//         quorum_snapshot_list: null_mut(),
//         quorum_snapshot_list_count: 0,
//         mn_list_diff_list: null_mut(),
//         mn_list_diff_list_count: 0
//     };
//
//     let result = process_qrinfo(
//         boxed(info),
//         false,
//         chain.genesis_hash().0.as_ptr(),
//         processor,
//         cache,
//         context);
//     //let masternode_list_decoded = unsafe { masternode_list.decode() };
//
//     let result_unboxed = unsafe { *result };
//     let list_diff = unsafe { (*result_unboxed.snapshot_at_h_c).decode() };
// }

#[test]
fn test_processor_devnet_333_2() {
    let chain = ChainType::DevNet(DevnetType::Devnet333);
    let processor = unsafe {
        register_processor(
            get_merkle_root_by_hash_default_333,
            block_height_lookup_333_2,
            get_block_hash_by_height_default,
            get_llmq_snapshot_by_block_hash_default,
            save_llmq_snapshot_in_cache,
            get_masternode_list_by_block_hash_from_cache,
            masternode_list_save_in_cache,
            masternode_list_destroy_default,
            add_insight_lookup_default,
            hash_destroy_default,
            snapshot_destroy_default,
            should_process_diff_with_range_default,
        )
    };
    let context = &mut (FFIContext {
        chain,
        is_dip_0024: false,
        cache: unsafe { &mut *processor_create_cache() },
        blocks: vec![]
    });

    let mnldiff_bytes = message_from_file("mnlistdiff--1-25480.dat");

    let result = process_mnlistdiff_from_message_internal(
        mnldiff_bytes.as_ptr(),
        mnldiff_bytes.len(),
        chain,
        false,
        70221,
        processor,
        context.cache,
        context as *mut _ as *mut std::ffi::c_void,
    );

    let bytes = message_from_file("qrinfo--1-24868.dat");
    context.is_dip_0024 = true;
    let result = process_qrinfo_from_message_internal(
        bytes.as_ptr(),
        bytes.len(),
        chain,
        false,
        true,
        70221,
        processor,
        context.cache,
        context as *mut _ as *mut std::ffi::c_void,
    );
    println!("Result: {:#?}", &result);
    assert!(result.result_at_h.has_valid_mn_list_root, "Invalid masternodes root");
    // assert!(result.result_at_h.has_valid_llmq_list_root, "Invalid quorums root");
}

unsafe extern "C" fn get_merkle_root_by_hash_333_2(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> *const u8 {
    let h = UInt256(*(block_hash));
    let orig_s = h.reversed().to_string();
    let root = match orig_s.as_str() {
        "0000086ccc192b07450acf188eba3d0360058dd43793fc622e0b7744cad5cbed" => {
            "52b4cb95e82f827e916b9a5a95198203cb7d60d892932bfc64aa9496c79631e7"
        }
        "000003227cf2f83a1faa683ece5b875abeb555ebf1252f62cb28a96d459bcc11" => {
            "5c30a026f7cb72fffa390b1200955354eafb6ab5264b8043ff2a2e582f230342"
        }
        "000003bd988f78d4663780b467dc6155f5c1641c16a2d0a09cf62eeaaf9ffb39" => {
            "b8d3a314e523596b23a57537ba1a7a8328c96ae895eab8e80760eb841a88df06"
        }
        "0000091ffc39c9891a425b7eacb051c026d8e6ab4a86f074b6961f7c4a0ea87c" => {
            "5703a09bdc76f7e37107d359766a74dca3ed0c71b7a0132260e153544d4ee208"
        }
        "000008804a2cc065da4752c7f6f4b889c682749997bf24c704ab44e932ae0f6b" => {
            "5551649a12b1f22b4e014b560aab1572f0470956bbe1c6ada1dae84310277eee"
        }
        "00000aa4e53ca2059b645787d4516dc75f1289708d2a2d3f6b103d171821647b" => {
            "09594af7333c774ec2407eccb9b24d2a4be9c49d8e59b34b10fa1f5d2c7739af"
        }
        "00003130c6ba235a12bf1057c459268ad10924a47d9b34eddc57858d77f72ddb" => {
            "ff4e47838f277a170ac230745b023dbe2fa9390fe9e9b3242aff129240944c94"
        }
        "000000b5f4669ac2ebda3607fa627df62656650ee689a8c82705f340ec3d78ea" => {
            "6de4c9854cd3ff66b4fc0f46d51c0a3da00919b08a1cde84404d8fa2ca8e8ba4"
        }
        "00000692fc267184a6ab60ed6af6a55cdf527773872c601f098569c254f2d189" => {
            "e74463659764773681b780aa204c8914db2aa831fb3da82dce529f9d67434495"
        }
        "000007991952ca7c32420cd92654ecc41d5716c191cc96cb8978ea3257a69992" => {
            "062fd8d751abcbcadf2027e1536e2710a04ed21f0d0ddc7f2697b7282f0bd04b"
        }
        "0000037bbcd8e2af574fff36fd248d2058b3a9582c9a9c3fc6ab1dcee69c25e5" => {
            "52430300040dd138bca38fe9441d94345e5c34644e1c211308bc420dde47ddbb"
        }
        "000004f78ebfe17c2e78c58ca1e754e02f3e52043bea00f63be7f2cd58d23c96" => {
            "ff3c3a2740c38cb87dbae24421f4473a35af8e8619ecbec677265bb74a0d1f51"
        }
        "00000597b3da0bdb229b571bfcebb78a5786c9bc21d3fb3253d497b8a62e5d99" => {
            "f1861fc7b0b00fa5cb76b4ec918f19a898ac50aeeaf69e9586cbb0cb2ccb4103"
        }
        "0000078bc1a0778aa6d8dbefde4f110eee570c1b945c553b9ed98368871d6b5b" => {
            "b600667103e998ab12bc8e8e45e1b6efd3ceb5ec8a15e6d7f765202d2ba70766"
        }
        "0000062c4e24e761dc2d1b9b6d1588198b3f4ad8a767151ef6414bfe87ec94da" => {
            "ab1331d5de92b75c32a8de2b75db01ea9676a7927779b3f38b9ebfb039d33ecc"
        }
        "000000e3756fb54df8e63db7c69158a88247ae6fb951602fececa314820a644c" => {
            "fc930831682f27c6a837e65adf545ed686a57eceb62f05a1e11c601183869920"
        }
        "000000461e6f7b4d5656173bf188e83bee5f10d971616d946db3027cbecfa13f" => {
            "77a02226b66fdd4b3494626bb75691ffc575b0de3236e519a7f88a90d2a8601f"
        }
        "0000037bd6d0b218eb906ab6206fde5a96f972ede0308bf76cac4190428a924d" => {
            "0d15faa6f607d4b17c5d24c2247c9327f1b36140bd6c10071a3360c1d57e6678"
        }
        "000007756da9e48ef99829c467379e3a873ec9bf90652a9d10483329ef4e222c" => {
            "544a57a166570b1569a58b736e7526ecdb79c972be5f4a60e3142b959f730d80"
        }
        "0000018ed9cd191989b356447ad4d0649d5c471e1661a79aab2613251cf991a7" => {
            "a89ef31ee6ac489f8f3607eb8f45de084d51cd87ab92fac7868e2a817440cb2b"
        }
        "000004413ed62592b7a47b06a296ecb79dfcf5e65227a3e93713b36c48846f6a" => {
            "48d83f6760e7c3eb3b19da95651cfc385d22cfe864a3b53c9c00c4654d1961f4"
        }
        "0000050c3da50cc911506eb4bfb50e7b1f1ac0d165eb62f239a48545cc5834c1" => {
            "892997c525cb923b96951814d4575e75ce16850e3cf12a9d673970682ef7eea5"
        }
        "0000010f2aabcceddff97f4f87987902f99629518dfdcdf5522e7e159b78a825" => {
            "6baa43c539d6605cfe29e2f49b0f5913874518206803c1031b9753d56f55ee7f"
        }
        "000000426666e40ff602a896a1493b765daa6fd6667f7c5edd0cfd7c23857750" => {
            "f0e942c71fc30f5997162a3392a7142557e801cdafe292ad9641aeb213aa6e4f"
        }
        "000000b693fbd6d2a02da459158394220f83a4cdb947117246c83a65c9296bf1" => {
            "48f81ed8fa2f69b1c6144e435ca78809f805db00109b708480d38a2b8197f241"
        }
        "00000683d6d9d7df1b530eecd973fd71aac2434687cdf94b02a959237b41b123" => {
            "9a27c39d7c0e47d331dcc07816b1794593e1c3b6d854b3e0d5f46a916766f7fb"
        }
        "0000052e1a801d4313cd6a90beed7f478f34b778c2a8cb0e8b98dbcf83c5efbc" => {
            "1e906bb71f88905f5f457eba573d4bea9fb37689dd59875533785b6c2cc506aa"
        }
        "0000076bffe00bf97fa0baa71c1444f5e3270cd0ada2423f1c77858b95d19338" => {
            "1955d09fb7650cf2eb2a8b300c89bb8b4522f0b720ea2c9c235c29625b72019d"
        }
        "00000763996b273b8f8fd4b184e0175679bae5bd176d021efd71d4d598e0a184" => {
            "ee69bab287a0561e23545c3092b5598889b2165045600b65091f5d064a38260c"
        }
        "00000088430da03d58df458cb0118dfb7a5583241718c8cbb1b02f7eee673463" => {
            "15cc835964dab2bb3886b5e5cee5511fba6e7c77cc802835dda39dd424265c17"
        }
        "000006fe4eb832348f5abbd7d7fe5653068dd20ca58e05bec321f486a9fe5eb1" => {
            "63515c8d179eda42dab1f4a975bbaadd9898788e061ec7357c6714353a332ed7"
        }
        "0000018db2f6d303edcdd6ded823acc31bfa50f08389111b4f90ca4cc753006d" => {
            "00022d442bc384793a356aeeacfe6d43effce4afa60d0ec5da4650c2e545755d"
        }
        "000002f575aa919a482ec157c9c3614057078bdf99da797305c9c7f549a9c677" => {
            "292ba3e5efad3c49097c0db3b818bf920933248a5706f845b313cb0598bdc776"
        }
        "000002dd2d2977ff6fa6765954919dd816b96671d4c8d5d0752f5e4efc48f8bf" => {
            "0fd38cd24e0b2213c2391152c35889fc18f84e075a1e214e4ba9fc82195a11d2"
        }
        "000004574333f4fcbbf989299a298c53b89718aa10c169c32db07c18c14a33ad" => {
            "eb722e4cee02f65a4839ed67631b80d1d0a7b323ee2c4b30ede92388eaa48a73"
        }
        "0000053f364c7ff258ff87185de4c789420372d9bd76f37dbdc57d5b3d1992c0" => {
            "ea387ae59b896c7e7351509632bb3943930370b884e13fbe703ca871d1725e55"
        }
        "000001e2680c8f337c3a6d81f7b72e67836dd64a6cbbaf13875a0a72976282e3" => {
            "b722474955912746173fb7f17f7a342b494c98d1599c50b7c9a35070c3e78924"
        }
        "0000030d4554e01211b54c9041f7a257ec83f9079a1913eda923bdad11f08670" => {
            "18439878443990f9fd7a2a598e0d74be3eda31dd198444aa35ebf9de3dea0dfe"
        }
        "0000045348e3cef51ae93a68a09e56eab3a08f339ab62b196709f83bc97c6936" => {
            "c1bc302bc6e7318178eeabfbcdfcc3de5b89b507a771aa5a06645ee05a8f03e6"
        }
        "0000040777831dcff5ffb3e3c0bd38a008a6d4456c06e320386004fa68a49e7e" => {
            "e81c9ab159a488f6a86c592f770ae44fbb31ccbaf77c2db31f8b7105137c30aa"
        }
        "0000027e119d72c5dda56cf39223b35f232092495996345be248eeb48c9f9130" => {
            "4ba1856be4bdd7faceafd79d8548c87917487e994b61c65454bcd46ff549f2d7"
        }
        "000000baa4a33d65e11107122a92601053ab4e716a6d661dfd6bb9f874188413" => {
            "73ef8566ea6a716033d4dea8a311043016efa0942981ebbab7289715b984b3cd"
        }
        "00000645381ff1106e135a9d5df3cec8edfafce71c182325d4b75099d4002c4d" => {
            "8e834a0a5482eca0cef23ce4a9c6ddca8412a972f938ede42d3900919db9f85c"
        }
        "00000056c854b1996beec75c464693bed7a958e9b67d71d701340b8ff6d9e054" => {
            "58811d85c144b03f273fc4579c5336e90fd58da543cba74110bde7471ff82b4f"
        }
        "0000027aa1d0102ddacd2f9dde2681ceaf0b5c0caa806cb7f816c119051464e6" => {
            "7eb1bd17340f7cff6deae1d6cb12de756338c145b51a58668773bc8a2275e47a"
        }
        "000006437989e53e813df1a8aa388096387ee5de9cd0b045779a703582e7949e" => {
            "6fa7e4e02ec3cf482fb179f463e00e34948115303a3e1ca0ce4a82fcf341e0c1"
        }
        "00004cbd92d058ad559d6cbb2d973e3e199015826eeab9691cb718ba349b0f70" => {
            "1271616d765ed0010c1213c190fe6d1d6e84859f6d6dfab62a95ba96c90eadc6"
        }
        "000000885cf1eaa6b419152fc651e0119d57281f18a6ae3806dbd4fcfb5cad25" => {
            "a28c627f58c40748536f3b6f6aeb73f55a6b1acc54ed81510972acba0a6acb33"
        }
        "0000049f89a5dac8c9593c601f9a385440ee050abe8cf5f039a3d59c58f06249" => {
            "21d1c3e4985f27a6b361891c63fe045a9fc3199234cc3e2089e9a3793de24424"
        }
        "00000104885a2403d15e65e25daf9f78c755c2953945914f58e6328c9dd50c4d" => {
            "08d3447875c127f2e37d06a1ed02cf973a61d35c7127860c3f3bb4393cc40dd7"
        }
        "00000795888b51da4cfe848adfcdff19cfc815c1865c58e2200ea9ad596df9e9" => {
            "7551b29e47bf5ecb9a39fe0722273c9b2622ff87ce088aa9c8f167c25674efab"
        }
        "000004169e45af9fd4ddaa64c400758df7cf36b0c1a22f2b9f29a11fdea70eba" => {
            "afe8d8af3ea9d2227d465613ec5005f8e663512dc129ef3169f013d6518cb493"
        }
        "00000743fc8bde3da9911bbd5275e2e2a68afed8ce1d56b73ed8557528a4fb45" => {
            "296d41098918166d02d61a99975ecaf7780f286d0b3b6e2a6619023af2c2fe68"
        }
        "000003e8049e2baee08e173a04106df0c4b0f858f8ae1918caf970eb10d2046a" => {
            "f258dae6201a647985d093f9e1ecc602ada427736888585556910b9ee324d475"
        }
        "00000169672fae75e70f45a7031bffbdc92fb81d183d0324f39eac9268d11ceb" => {
            "aff7439850e5ebe758714832bd56c19d67e005d7a27aa6b977e3a158fd86fbd4"
        }
        "0000082b9da7946ea66dfa9c6ca9765a41219a049129248e4c5656f9bdd8aab6" => {
            "58ea2e9310568de837fcd3278cc9083c0e31b79896e25c487c237eb65f38dbcc"
        }
        "000001249c80a3899d72096a03195ebfb96b27118559094e149fcdee18017458" => {
            "f68d24638b3b947689b1a95c1aa8fbe51cdc0bbf2546bb1b2145c09d97901598"
        }
        "0000002d7fc5efaf7d8799e6418f2cf0eda1e9dabc9771366ea66c5e09dae7cf" => {
            "95a9cc169d6600fc1ae5188010b0ab277ad816b59f78a39b4876f6641596bcbc"
        }
        "0000051ee159b9aa029497f509e3c92bfa03737974444d04bbc940cc8a901bab" => {
            "7ef98bb9b3566288bd7c28a2e42436bf8f60f52b15c8da32ae7e624988f1bf63"
        }
        "000008076d72be61be309f5a8e531eb5a85405f4ee75125a1b3556774b5848d2" => {
            "7f9512c2488e48e0ac3967ea476344ccb9b2572865e294c4a1bf7c082c0c1771"
        }
        "000006cba36bb9086be9ac34b03d0bb2379ccbf9af7cd17da4f2450dc9f7006c" => {
            "767d18f30dae94f145f81711f9867a903bc8fa58a4cc0d93791bc31cf683cc64"
        }
        "000004c5b1b7e1bdf532e1318154cf2ebe387b5b9ded7c748b75856003a43371" => {
            "85fff19567ca04c51cd5f345c563b755480227a512e63c2b94f007d82089f0ca"
        }
        "0000066acb91115887007ecbd8d82eb4f4c32c0270552fe8b2f32e48213575f9" => {
            "3280258fdc954d6470088b8b998ef3159f746721999b1e1ed8b48eca03a89897"
        }
        "0000006dc4fc61cdef4df41ee9b7f49fe17eff54a66f3a3a88bed959388a5434" => {
            "b3142f93901412470ff5a60bb97f154bd073fc4b5680de9f6e9ed8105f952f57"
        }
        "00000178548ba2f34bd307781683b0f97ac31ccdd9b1b22e148c59211567c407" => {
            "b2c7c4467f297d5b5555a78a04a5f388191ba0e1b66c46910ab50f23492ca314"
        }
        "000005e3612dd20b3bce4ea865eb39045fff1f8937301f9a1db3654d1a98ff80" => {
            "438c23bd2a25faa8a0e0938b4bf472a978e94f3ac28cf32c2cb4de1b093dc804"
        }
        "000008ab4c5a522743760f8ef14089c352b0f7b1013cd7ba7392c96e497afe85" => {
            "ef7eeb4cdababb13885d3914e1cd7b8f93d0372e9cf15c6cb9a9733e189d4105"
        }
        "0000049c9e3d99ae33ce5d71e7e3974d8be8d87e65f66f5d36f200f482eb89a3" => {
            "5a951f79deee87276b7a99c8d919b5625b11bb2b5e8130d101c0060c6ba22daf"
        }
        "000006799bc165a2d527f3f980c919863cd3a78dcf3fd201f57bdd6c27ff7af2" => {
            "edd0aa20824e1b9db9b04a5ee85fb16a7a7a0553999537caa2717c6266975c1c"
        }
        "000008822d1b4327327d2477ed48157ab739f5631451f59e669f65da6b4d99d0" => {
            "4a653975f3b01949bedf979fed7fece05ffc064e4d0ae5c69f57999445a00c70"
        }
        "000005cbfa02e404973f41eccae792d8394f35089a39ae4f4242e61fc3110dc4" => {
            "638c996bb12ad3581f31f2994ebb0983c975e541e6bd75fadc4aabab2b1e2949"
        }
        "000000c25bf5d29876879fd5664efecf5dbe6b889756b0413a7e9a7e14b63894" => {
            "33b7035620f8ab27533844a2069cc73e8879d721a79a3f4aa302d3516103dc29"
        }
        "000001c6e2988ef3d42b7fe817481eded236109b1ecab23bde43710cd0b82890" => {
            "5ec0593cf7bf479877e0b5aae2affe195fa860693129b14906f6205829e79db9"
        }
        "00000043a23d0bbcd576eb8c39b194e9ee5364ed47645dd4917bb45d06768777" => {
            "c47aea598c0bcb0fcaa5e3d2a1cae54de3d701ac999b3d75c478d8c3b4062e88"
        }
        "0000002c7dabf7c52be36c77728fc2a1111f0caa9d167e4930c954540c3aa142" => {
            "291739ec18d97bdbc2165b78276241a09071c780f703f44dbb99070fe297f260"
        }
        "00000213354b423e60b5e4e3da96a57658fae6451078b4301eb7fc96d286c346" => {
            "8d65d6f8d35ea86b40e22e7a281aecdf316ba3bdbcec0d135d732fa4e0adea5a"
        }
        "000006acdb4419d876c00a195f16edf1b5732ef6c26e651ae9f2f3c2b8964a3a" => {
            "ac0e90305591b67c48afdd22854961582b6270ada16b22b12e51535275ff6dff"
        }
        "000002a9757ea470f4d6341cec13cc12ae8e7940bc401b36b22a1a44d39cd7ec" => {
            "403baa600723264025e8f4121bf8a6d35e0f86a6791fe3c05ae9cc9a81371c99"
        }
        "0000042837f5359d0b4d72b88a90e782afe782afbec03fbe54c7f06ef91e0fd4" => {
            "1d1e1958e0cecda3756dccdc0daa9bd75d83534f99f1573e7f0f04a79d8ef999"
        }
        "000003de21b33156e7d94b1408fc9ef753f7cc9bfce52fe6ef2def3c2184a00a" => {
            "981f77de0a04a241c6ba5a9532c90f95399548c143ff6759a83af1c17f1d9c92"
        }
        "000000f63595d3c80a44ff104b19828bd7f337027dc064cd179e2eb5e5e8db56" => {
            "60d7c94e27c451d0b0519abf2e7297d6d03edc953839850d67f8fee315a333aa"
        }
        "000006ac1f34ea47d2db12fa1c8dd765190ab4c5c8329baf12c6a8ba5fc42dbb" => {
            "96bdced70b2ea5938ebab0786921422488e0c11f1542460b0a1a0abbc9ded2ff"
        }
        "00000401fc8d3cfd75086cc67b73a32d7978877e1726ad9949bbe67dc4d46840" => {
            "1680e4e050838ff64711a4e0b513a2a3c12e769bcee2ee98ced3130be22a9dc3"
        }
        "0000027b6ceb8c45d444005a6b948ced264c8fc39fb412dc69389ac63fee668e" => {
            "493c8ceb8c8bbe311d3a4ab438756f753393c59eeed20f49f450e0fffb01513b"
        }
        "000003f23b070d2734940dcc2682517883b6f5d3189c34b5dff4742153ec64ab" => {
            "42b4f4d81356e206a38e8902bdd3816c53bc37b99c26ad5c2656d93fdd997631"
        }
        "000002640f8273af03475f6ee496f80700b12980bc4fb059f734e6183377b078" => {
            "cff2b67def2c0330f3b94408263f0d16bc9dc6c5d7cdf1c27b5d69e7bc85ec2c"
        }
        "00000005cb3272a005d40b45caa1cdeaba2651d619cbd6196f2c6c6317c8917d" => {
            "e869767cdd4f87b2e5ce17af3771e3348e86fb464ab52415f5d6ae7f0810c666"
        }
        "00002b2c169c551e051fe563eec39734ef89eedee4d65f8a2c0074280418c8fb" => {
            "457ce3aa9c57e761c70006c5ee2b6cdc44268dffebb1496df027fc94f4434002"
        }
        "000001f0718c066dd865f72836254d6e9b217f49ed4e08b89bb8e7ff3b3aa6af" => {
            "5ce64f838c23a9684ffebf7417398a6978de294d0417d67721f2f3ea1fe59988"
        }
        "00000156030a46774d864bc346009cdafb47b9ea30a1ddfb5ad5ef5bcc8e0cf9" => {
            "25b41d4886e9c6522bad0f930bb8f6321b989abf22cc308446aca2332bb443ae"
        }
        "000003799100d36e4f1eae601ee81df2f02ffeba59d3f29a7db43c2b7568d481" => {
            "8614dc6a4b3df476d29df930d2a85085b747a9da4444f2ffd8ede95ecc3c3618"
        }
        "0000047213ccf1910fe90f8b8ced3e8a2e3a9c1bc93ebf4103f250dde548e470" => {
            "433604c27acdd014af061155f9f0427d6750eeda899be051ebc79988ee5ec741"
        }
        "000003e3515c438775628e49dd61701a8327005566c2b88407d9afb59244092c" => {
            "62b4fc1cec68e8700b428464eaa123d9206afcfe5515e43b6350cf960ddff3dc"
        }
        "000002563aa47ebb2189f33735f698231161c5df29627e36c01dcbf0a6a2315e" => {
            "2b94cbcfc62f96e38a2d35e89df86e3c73b6127bcd5ec54a1a6b65e425f62a9c"
        }
        "000003c7f4624196872c229be62947dd0004e09571fb1b8c0625fda088873e5c" => {
            "267d3e878168483b7a7326a5dce30de10719ebf612c6ec1609f97bc0c25a559f"
        }
        "000002c6cc3fc915987a1c89f295e969db682abad30b7143c66df3d020c81aa6" => {
            "69aadd8c54d743e283f7efe28c6c3747db9cc56dad527660f93675882c1fccc1"
        }
        "000001a0000cf3979494e1451bbb2e5b8e5364c003d69b07e33fc9a4a42afe49" => {
            "c87bcb325c5385d3a3d59d9825297e5afa6ac90c39c3bf8ce564f0b95310424f"
        }
        "0000051f34bb857fb33424cb780e066cfdd565c64714048d42b84d2ce64e0b87" => {
            "8b6c786b0e89691b9f525eccfbdcfe3c9d2e40486a541ca9e7fb584d88ccf5bb"
        }
        "0000046900a9d9478e84fa09702fa98e734ade9416f43229969c03d499e2b827" => {
            "1e1f36e337c0493bd1b178051c8b42be434b0ce567b9ee1ef844fc950cb7d612"
        }
        "0000020466a6a3f34b7edf6e7a56950992eb43e4f46babce3c8e7a1f027932cd" => {
            "ffbf5508f9f933ea14b91b00d804d63bf6f816a09db21ef2333e794326d9e4ec"
        }
        "0000008ea4a9398eeaa5ee996129f6d58f3ee6eb36ad48b39dcfafd491b72698" => {
            "1fe2f273d42263374697890bbb858d5c2479837599eff5e37ba9d80181a71ca2"
        }
        "0000023b916ba5f3cc323065f8673c6648cae692a97cb84ae052d372e2ad1fdc" => {
            "2d062245bee6e9c053bd5e26bce664c0a6c4271ffbcf4b502dea1ddb960d905f"
        }
        "00000128469a850ff260fab8d71cec4ae08953f53adc769b0e4510c64cd5b1ae" => {
            "1e366e20dafbd93954deb58361a6c92b0846ab57015033b4d46d153fd07376bf"
        }
        "00000329f84bfad3889f8e37a6e5f532547a73840428b36a875423aa3f781922" => {
            "ec8aaf177d99942033d4a5bf04f68436a37556a86ef86bae68b992150d8e0fe7"
        }
        "00000039433505249e61dcea66c0f3075ac384cf216970149d2e7f149f031ac7" => {
            "68a58f7cb6462b5509c8a52617fc14f4113fce4773dcf679345f12647fbc683d"
        }
        "000003c944684f4df6d0e9df65bd0ff45db24279400c977ad1ddd095d3d26a20" => {
            "1658ee60f05710256858a23281494aa144333ed06c2be3c167979cafaea18d7a"
        }
        "0000050ef9d0c1e6e6d1506dfc9a93c87b49adc40d3f266f168726a987cbb16a" => {
            "1ea15968bf1200beae5fd67242c134536c683388b88965c1332f41d147960daf"
        }
        "000000457cb432ced5fb2ea78d36ccb2744d22c1a4aace36ecd8e603325282fc" => {
            "15cd134152a289645ad5bf17b205fb7651f32954b4b0302538d81d67782eae37"
        }
        "000004c958970f8db6169284b8fc0a48624964e38a1da4260a528855c549ffd0" => {
            "69bcd4949035c8fc2b0f2d11806e031d7b1e728bf3a39c406454c4767eca3747"
        }
        "0000028a421d2f116e56ac5da67217e942fdc5d783d05f2b0ec08aaa98a4af1c" => {
            "e1b6c73a9f3540acbf87106ca8306edd9b25e40b576462f00a5b0268873f85d3"
        }
        "000001b392b1b5e29cb334a6c7d9baf85e8468fc9d5ac2860c2ed4dfa8affb3d" => {
            "323b403506c54cc4119029dcf2dea0e2ad27641135abac006aee894df0fd9e57"
        }
        "000003a2b3a849971822edc9505114e61704a58db8ebae14c78bd0fa9184f519" => {
            "52795e25b88332aab36e18a3871d8de9f0ffbd7a26394e633b85c52915d6b6f3"
        }
        "00000081a650f7953b6f6e704b14b142e06972a3fdd86ebf5327820adce8a6f4" => {
            "0599719b08ce78a6b79c9722a8d50e0f124a9158df2198c78db8cb59c2b13d0c"
        }
        "0000004b32f5cc9512ca0bd2f96f84a29b975c879ea050c7309c9321cd76d67e" => {
            "03556bc5c6eb0458410a48319fbae6d8b68e97264dd0185cf7996e3aa2432fff"
        }
        "000003367801e71b6bfe4ca3e277603f2f9091caf80be7bd697b2e7d8b2fa091" => {
            "49e298fd9fcadad735a72fa84279e66d3569ef3f417033e34ef0ec0b764d0d2e"
        }
        "00000555ea65e4120624c4c8699af9ea3c91a3f8e42095192397fbdf6774f02a" => {
            "3f9e47004fa5ae565199aa25f23a0d3e828f7062fa54f3d3922f8a8d42d87dcb"
        }
        "0000016c4f0f1cb3ebd8f2b2adb53347a3c69fae7c0854b4ea41a1d4c8af7d6c" => {
            "49ae7f66ecb7ab04675d13351983a4cb0d9c1270f6126a44f480ce8f7b0da9bf"
        }
        "0000028655333153a9e4262170b8acfda4a2dec64dd387b2033dc0a075eed242" => {
            "25b814db212d80f6ec64c896bc5703ed6c83d25a21208904dbece269b2c1e8f1"
        }
        "000001c7704486626aec884fe851b5b54326825dd56d56d09f03beb2a576ae09" => {
            "82c0e989264ee35a73c793ff4787dd9cccd360b9824310350f96a84e68a65486"
        }
        "000004db5d642d4a869ce8100ac5154658c06b9ef0b9e6b9957139529884b79c" => {
            "7837bd27d828f25e2553c20bd3c1bc791e74153d714d8698ea73e2274a285e9d"
        }
        "00000495b214d71655cc24f56bf9aab7d2ab286857e6f9890f1ae639c357b69e" => {
            "9eebc821850b559661dc12d2ad7956bcf84c48855157d69320cebed2a7af7081"
        }
        "0000029d66a078af9d234a430f0cf8580ee0aac264fc91e25b6e379836f64aa2" => {
            "095dc5603b17a20dcf9098bf78bc823ed978889a48e5419421ebf96bd1a72e14"
        }
        "00000319ef4c483b432a52e103a5fbaf72cb2cb03238793982455386f223b5f4" => {
            "e127647a7116620ac6b5c9860c3db07878c907a5455465eb1e7459ae0d44f535"
        }
        "000000a5cd94be2be2764cb5e3eebb1c4aa24b0bab289ba4bdf032cb5c4511a4" => {
            "91d3e18a32ff9cd35e03f9fde5c23acae3a49dca5ba53315621b29d6518fa586"
        }
        "000003a4acaf2de030463272ce372a6d38973bc4d3ec9abaecd88fb242095c6b" => {
            "dd26bba770ca425e65f1d246afcc66b13acb67f255a0595d4345d1b567fd82b6"
        }
        "000000b13d3a5e352c02e26fbbc0c5a19c630ffaba54177c3074b08311c334a5" => {
            "e1b0ff1824d3492cf1b57ebb37122bb0fa0291f2da58884326cf4556c594d43d"
        }
        "00000306d57c64e4ea982061c855b4ec7830836e9779d45460d96b7bbeb58ee9" => {
            "aea894630ab1cc0e67341cae8254d4c94ab0673542b6db168a88117e6b80eabf"
        }
        "0000046a30db2347c925c399ab3d08e95f134545a1b4f559c970db585b7a2a2b" => {
            "62e912763dfe8494d354d1422997eda857e8a87d737af9258926a5300d2e2cb8"
        }
        "00000002a9013220364222bb38dbfb6fa1a53177524d9e0de1d243b34ebfc136" => {
            "36f3743971a12d0a47343a8e420ff0e7f8773c0427f15b59ddfcdb63edb41f9f"
        }
        "0000004c90e593866678aa4cba5211019e02e922ff7180fa3f152b20aeee589e" => {
            "d0a6830114029bb5ee4fd98aac7f31dcc10b29a269a5207d433a4f9ef75acfd2"
        }
        "0000046d5e84be89842cd2f74a3d72f3299e63179409c7c14757b7d66ded1132" => {
            "8d0b3e7526c05a488a316457869911ddbcbe076ea0725d0731acf221e942b03f"
        }
        "0000024082c1607e22f1300d5df14ae43a7b41ed747c0a8ee4b6bbcccab493d9" => {
            "9e33d98eb38e719118539ca93a2b01bf960bab9912230ec389187115dff67a33"
        }
        "000006aa0077df5557c493d7eb81396e7ab10302a09d9fb2021d1eecc0077b08" => {
            "a435355c1e467e499d84a961c476252ddf0b866b832ff4ce9076fb891e0c92a1"
        }
        "00000149b4e0da935b7aa97f720ef7e6d49e828880e20d7fcc368923176a8968" => {
            "cf0a805bdeba1303ad9bf42102f8e1f6c115a808a30d17468fe491c07d35d598"
        }
        "000000b401b4b4a5174cec59765cbb23fe491ca4b7ae1521ddb32f3425860de2" => {
            "fb0e7c8757438e1d743fdf9a9aecdd34e3b8b8b4926e0013b1e619f3281cf955"
        }
        "0000056f246e61d0ce87af62a18bde093a00f782b339bb5708fb75abc3c1d3db" => {
            "b05cc74248717f3ca2b86af1c0adbaa136fc1531b6fa4556756457bfe65f25be"
        }
        "0000057c5bd82a630463ee1a6cc591c7b52b352c603d3d8909f6b4fc699fdd2b" => {
            "fdb3c89356432dfec8c3f338c04beafb34a10a18ce9bae6bad242e869572083a"
        }
        "000006f2286327b6a2a1270f770609ff027d02180d69d2ad18b63566dfc0515a" => {
            "b466f799d1bb83d68a84604e326b94ecd2e7c81f2c5e2484b81fe1c3af242811"
        }
        "000003611f378cd85a09df3bed09f29bb69923a73584ec70830ffdf4d9b25285" => {
            "b82c41fdd20e60fd8d4d1d2a20f619c164abe9c79f6aa412183be6305bdb4c09"
        }
        "00000018c1d055316dd10aaba200a40e4b22bfaa2aa4fe946aad9c48dd265a7f" => {
            "c4f40a3fb78b45c92e0c277088e11effaa5c82f3e090050c9b62fa118ec21f59"
        }
        "000005b7c527392bcfbdd75b06f0906fe85ec2ee8fe8c1fc08a4f7820e091f90" => {
            "19074ed4bb1d8c8c48457a6fd35944e3ed82fe9fb41058ea4d933843a982ab66"
        }
        "00000483b14397ee07d6a6e0ce4caca153c46e24ff9793a472b80be780fde4af" => {
            "edca803c70b7448a4a9bbf9a06ecec4f5a9355986bc4690980e3e38edc160a61"
        }
        "000001531ce19a3042757887098225c468a56fe159729d40d9a20e764875a0e2" => {
            "77e1a542d4896155ac4eb4046ebcf8c7f5b4c8992b94cf539fba100c4702f730"
        }
        "0000002f47f9cce781cee4169fe7e1a3966fafbe0375f6844d32b1337a91b4a6" => {
            "2946f59c0bc5b135a4dee08a182359edfe34ac7058e3b1200706aa3b40d91189"
        }
        "000004e5ea1774456c0c0225dd1bdd8ad0007f4552df456d5148243d6a38359f" => {
            "665e1e3efc15062c731180cba3c9346590c22b1e025ba69e63d5738627d69d50"
        }
        "0000022ee8c41b97ed57f95887d5942bdef91728abc8e5aba6ee8fd3b74ab4e3" => {
            "b6c3746b47cf2f623a33d9f9f45b9994cdf8a7bb56b90dfe4f8dc72ffaefc88a"
        }
        "0000034aceca60ff1c4136b3d3aba4a875d47b0cb481797b0dbeef9f1125485b" => {
            "e009dc1821ef7c066424db510a4fa92ff09287008a791eafa2dd30eaae0b0fb0"
        }
        "00000798b049b80fc751a9d02ce80872dbe54b3798123e47a7726be03a24013f" => {
            "23e2bee91da462aff5631ef2d38e61812db9f47e39d3dc9d41638bb9e26f82cc"
        }
        "000000f92ebd1f266b370a27cc8f3426f78153ab3fff492b2088eb92ea6a6243" => {
            "4207aec3e7fef47b03c948b56a9aacebf50942a144a60dd1dbdd099fe0b6f7b5"
        }
        "000000d4bfaef03c58066d660a30950705376f55fc9430fda897f1c340c149b9" => {
            "88c437101e0c5b74d3da9e602336b5ab5f718ad25e260f4929bfbebf63eb1de1"
        }
        "00000123d0cab60bc8bc0f7748d040090429a3570a55b77e11b032d90578fa95" => {
            "386ad0c2a54da863b60ffc5a5aa39f15865d8d63c97d6f332486ea86d5070997"
        }
        "000003258e7acb7c9f8e2d5f8f802a6418dd819388bbc1463360ad9f9505034f" => {
            "06205bb387002ecadd7008c111967944dd4c670c4660341089bc39113bbe2839"
        }
        "00000541b0635b6b9bbd8921d4a8fb12cb6633323cda8cac7a84c32dcc79cbdf" => {
            "d5b027bc4d5b5e51a058fc1f9790e4305aaf34a90b02bf52f9ad6b1fc48684f4"
        }
        "00000605bee6c4084f4bc1b8e0a0b4565513085df4888fbabb1ff59402cdfe60" => {
            "a837a841fd23b1239cc1192f918baa27f17b2c3590861fe1cbf1ed0e7df25e72"
        }
        "000000a42f0395faa6036ec8dbb6f40618d74fbc35b2c343bf03f0d5ab1f108b" => {
            "f3e0fd4f5839ac5d7006f151ecb69e3f809f956aa17401dfdaf23b3e562edcfb"
        }
        "00000535556772e86ca1210083a4cb4db74454f4cef914fcc719cdd7d17d8c54" => {
            "328d107fab30086071b55204e2c70a6578feaf25c58a9ce908a55c016d5085ac"
        }
        "000005e33e11c29db88a6a59520395c6d74ab06b168434171d51695d32d447e1" => {
            "13040668ebf7d3002ca7a8d92c7e900ee30005cf5676fa4bbcbf615f3b9c5468"
        }
        "000005f2268271f37aad0fe7cfb9ce5069d4ea9178d999b3488942c0152dffcb" => {
            "4abc6a4044d5fe0b8d78ce110be8b74d0e93ddaf13ac59327b783558943e85b3"
        }
        "00000470f83644a5c08132629c03807cc6682846e2aa5b39959d5f3cd12cf48f" => {
            "36e2a661bfd3f2c714f014a17536faa773cda0b69e4c6184d8e3fc5bc4f4447c"
        }
        "000000e70d9b8d5d9b8a6af9b7daa957df3d36f30edf1ef5a59da62eb10e17ae" => {
            "c26379382825a2b0e049e4879f8b58418903a3727996efb9cfa06753159ac7b0"
        }
        "0000065d0abb85677e6ab4af484d80f7e90a3cc01fc46c555e3ac8659644d844" => {
            "271ef00edcb62830944bc4f8b12f5468d1cd03c6f0fd91c23a5c84fb6a54a84c"
        }
        "000001f6cd40684b33b496fc574facfd81790681ee36088387a364dccb48def3" => {
            "48eb977cf38aa34fba9c7b3ef9eccf981f3dd2be79ded257713a069867400c55"
        }
        "0000049522e2a80117de535914d67c7e8904bfc0bbfdca4702d56939c06a8cd1" => {
            "dc6aeb6eb686dbb6fe006ad0511f62f19ac6fdf1bfaf2d378ab05227db16c112"
        }
        "0000012e65c0ac8c90b83d42a532b4a7c3774c0cdedfdb74ba64d1637ca4a7fe" => {
            "f4ca0168659be3483f4d76b7c6fc1c9b2fa0980e647b62edb4ea90bad0574401"
        }
        "0000007f70b2ea36d666379e731f7ecff79db6ec9838d39d24af693756608242" => {
            "10c48ab606cf8d9df59a189d2f8854b11c0eb0d31133a72bc6a48b7ee9c387e8"
        }
        "00000098ef907dedce9eaf2132cf67545a0f746e6316c698f35bcefbc5ead111" => {
            "214fdc442396bb4237f7f685321be9019c5af999e322bb2c65ce407c0541946a"
        }
        "00000672106963c28e4ae06d0e0491bdaf1aabd4ea3a0759a65261e231d6aa2c" => {
            "1edc0d86c37dcdf052a906ea97e845e6fe5b44b60e10e56297186b53f89773fa"
        }
        "000000fa70d8d7cba9ebbf96c297e22cebd58ab8d7e1da8fe801606701237610" => {
            "805000906a3d51ac46798c7f705e05677a9a9d1f4a7ffb8aaab534287427400b"
        }
        "000005b33952dfbb9e9c936503c1f474df369bb7807adf95909b37d316e9052d" => {
            "915cf5e4625e7f7af51f736906f11516167dd181ef9043bd79790eac7f54a183"
        }
        "0000028acaa3b4fcc3eb0e9295a3a9a0b395ef464507a9af5bc51cd2b04b38e5" => {
            "f14f86f7659787743da8ab9640e0032349a49cca51a55a2cb7f056db71ae2474"
        }
        "00000553a90aab22049b035f06cd4740d4c283c3b7daef5bfe8246b776e5d0a6" => {
            "87c7b4250747991b0a426d29df0a0a5c24ad315fcd0dad30e92d89aa78658a7f"
        }
        "00000015443346b5aaaf59fb71694428399183a0da3d23a36ba62ef6f868d65b" => {
            "a3fd7bfbd3fee3d24599b79286949ca533a5b8b8444c52b52ef8db84d412dbdf"
        }
        "00000524346a800cb8b841e79a429803f06daa25dda27c4e15529711dd026485" => {
            "0b93ecff8e8cb7b9f6ee8b04f9847ff58dd5942c9febbb471f6b5cf8d90621eb"
        }
        "000001ade388c5280e9c0abc025d860f4b134d78f5f894910966ef58d79eb6c3" => {
            "a72758cf7ccb75d0798bf0284bee2e2c09c9703c67a828425c70510275242be2"
        }
        "000004970192dd41d3afe3c809b8cb150c8210a6cd41360ecc504f8e765a14b5" => {
            "ec3603f979e4567d5de6e6720ae213f5c61fd89b2eb3b35ab58dbde3d2975461"
        }
        "000002be5a953f7fed9e4be3361df036a07676fff669f3fae721c1cfdeaa12ba" => {
            "854902335e3f9925a489da28b4e03ee6b13889323b86ac29199d27510470420a"
        }
        "000005ec3a5ed58b575c0dcea597b0bae984f9642cdbb745c2a4acf92b1228e8" => {
            "0a6de1ebaf8d6f5939cff888a90c1c9e9719c505f5a065937f45db49d228de26"
        }
        "000004416dc20703d3aa5f017d7cfb96f1f030c06dafad3b0c45414e7b84847c" => {
            "02893469abc44cf760eb93652a4d14bc9068aea36e6a257d2c22586f4b19cb43"
        }
        "000005cf0f4caba6b236341afb3e7f8beacb8018364136c198e6db485f131e14" => {
            "f00ef2bac46bd56bb5fdc3a92401c5aeabd2ca5768fc160d5546a09ba6938b17"
        }
        "00000487243de854c402ef3026822e4b33cb23f3526275e931d00589697c7b11" => {
            "11b5d83a01175ef00e457e2c88b5676cb771378edc196e769f8a3d908dc51db0"
        }
        "0000040fd0c77eb035c04daa054de7df0a330bdd767a2b9f40929fff4b631ca9" => {
            "fd3c2105bf11754a8b61951ad9c1fe8620d9bc806fa88d7ef108af2faf57f415"
        }
        "00000212c4234717534cf9b4351dcde432eebeeac2231cd04d1a1f470710e72d" => {
            "6ce77c3fcf9604c893f1308a90a2fb6cea90be8c74e595dc71cee56edf27f914"
        }
        "0000037372e681ae0013faa6f509043940f39345f6fc2c9d69bfb26a92d4d3a7" => {
            "29cbb16ac8bbd849a49c36ea89cfb6e301d3ecea23b6ce4eed02dd0c8f44c6fb"
        }
        "0000000c03ffa98e5ba1c1fe1441c4812a91746aefda9b6d150ed08242d4821b" => {
            "77e5d6f84c44c054830621c67d5401f9d678d88a783b6106d8eb4578aa58020a"
        }
        "000000602033c5a0c32421f3adcb3de7189931a633e98047b38ba6717f242740" => {
            "37d6724e05f33571e78f96a97efa65f3b5006e9e0302e19f6cf6da87291c5b83"
        }
        "00000362c6b66c7054dbdbab81c79e263472b1cea6b0a3e455716fcf328f8e58" => {
            "fea27119c993c72526432dcabd8994fe41b3b0f2f0132621310ae3cb2ca0631d"
        }
        "0000002fbb3e6eb53ec3174f2071db442dac5ffe82794cc5b0498fbac080f8da" => {
            "69cacb471ae2813cf45987cabf36a59d65e845b7b041dd488a9b5ba6ef5ff03b"
        }
        "000002177c627bdcaf2e95791166c9641bf02fdd5cc1d4a85ca20aa25acdd107" => {
            "9e126d96a5fe6d73ade44c5d3d78184462bee4c88e9818c8dd8e924d4bc16195"
        }
        "000001019891cb0eb860698cca78b8a17fae795c0798c437a0989896bb02c353" => {
            "bf593a2283bf6da71cb8226341dca164cf0487433403821bcc7ae730b916eb94"
        }
        "0000013f4ea9f6aebc88a59d1fa8631f46eff2e18e54b4ae9d284d1e640d7fea" => {
            "2edb60ad3b3cd55c337c5581cbe7b2beb9fb32f3bde5ba01f5f318ee835a4089"
        }
        "000006c99c58dbec27e6435cd4a1552007c55afec8af797c1435196a3c622aee" => {
            "92c4d264369a41a886a185108f3ef02463fccc088b2d0a993dd0ef13366c7d5a"
        }
        "000007e05572b439fc0a21ce64981b82669d34d73144344b80e2fccbaae85687" => {
            "10bd9b03ea231098959ad01bcf7e39bfb30e62a8edc8311f87be49d3f589b78d"
        }
        "000001be22b6a741824b01cca475101002e0f1457f024d439ea5ee0a1aada27f" => {
            "4c86015f0859abc0ee367c66c4de0e0a6b3a5fef87af890770ebd95f0aebeba0"
        }
        "0000001072bcf507e848486e5ff320c1522f887b158ba6e4303b3ceddd4762a8" => {
            "c367cd0b8812bac0076cc6d09751ffba418f69dd6e77c18dfda883ec8faed411"
        }
        "0000035731fd748b408aed499c9c4b45628e9338270a96fabcfe763f7b6e54ed" => {
            "e0f1cfb22cb14e932dce207416f5ecdf3ec99d0933cde16c90af24939606ceed"
        }
        "000006501d187ea8b178cd4b6d0aff146dd478609487bd889a693ebe76d27b09" => {
            "9650825a23523c0d5db69c15193d2c22514a447024a1010a8d26e8fc8fcb1216"
        }
        "000000b48edc5982b7986c89ddb7a3046fce5418a3bff0c40b9c208b768dcc90" => {
            "e3abd322716b7211f7c730d14dc9be77aa76b56f9ddb1a5b18fc57b202040ca7"
        }
        "0000040798a6b4632f2a9dd0f2a07523affae1bb4ab547598bb972d76fa19c8c" => {
            "3f23f56d6beeae1c6647f0c1dc7ac0522611e247c2ea3afdb31954b07d69deee"
        }
        "000003a5ef3d13b8f452e41fe4a45156d512c295d094e6ea352543b2280929b4" => {
            "da11d7d1db998c2685d44092943499cede70f4d5c1bdd546326385181f36a5d0"
        }
        "0000068ad662037d30811d86dac5b50ac0ee104afa0775b7b5be6e89ace77929" => {
            "8b4c7644283cb143585f9ddfca36598bd2b0a7dbd538c879b0091d0ecad9335b"
        }
        "000001fcac0d415ee0fec9b164a5c0d3baf7fcc01cefd8f55ae1cbaf4edc557a" => {
            "d801948eaf709e97eaf071ecc723c675483e674abfe68963d61350bb893dfdd3"
        }
        "00000439528d740502cfb49d869ea9e24d2a13ec9b4dbec666923aed9a74d52f" => {
            "d26a3242d276db7633aaa7d9b90927773364cde8b9d066003d42cea6211767f7"
        }
        "000004404897d33d867d3804df92ceffaccddf2fbde381800461ea3b88d89d30" => {
            "f0aea4033ad17837fd709be4ff01b5714b5d5faff1d684aa7ba57fad062aacd2"
        }
        "000009b14dec4bc7ba636de6ae25db8bde066f132b2fa96d9c4fa29ed1b6b82f" => {
            "89f31946adca397478aac8c8fca16325036a1b58697c8235611ce7a3d7ea543b"
        }
        "000005c6ec5a786816157f38b3e721529f6bac1d4a9e20bbc6119d1f44b38ddf" => {
            "ea2c7197551e3fcc6379fb0f828b251827005cdf09421c9b533eb6633c13c447"
        }
        "0000057e45e1a7176be2b0489956d83a1044e5e842097e353cd5ebae7f1a51d3" => {
            "9ee5aace19c67bc9149860b96166f62b056b99cd8ad4649ff68d749122be62fe"
        }
        "0000022e51809198d0ecf3fc49eb4706f9cd09b6ade82c9b87923cda2e5301fa" => {
            "d066614dd7466b66dc840baf48d3512f3b5c3d90a762d07cc83d16640553be1c"
        }
        "00000854e9d76d2507226e86af114dc3572206011d850843a40c1d3fea0860fc" => {
            "0838d17bf44b280dae9beb09c8a21c0a7f362d11cd5781f890d24077fc47054c"
        }

        "000008ca1832a4baf228eb1553c03d3a2c8e02399550dd6ea8d65cec3ef23d2e" => {
            "e0028eb9648db56b1ac77cf090b99048a8007e2bb64b68f092c03c7f56a662c7"
        }
        _ => "0000000000000000000000000000000000000000000000000000000000000000",
    };
    UInt256::from_hex(root).unwrap().reverse().0.as_ptr()
}

#[test]
fn test_jack_daniels() {
    let chain = ChainType::DevNet(DevnetType::JackDaniels);
    let processor = unsafe {
        register_processor(
            get_merkle_root_by_hash_jack_daniels,
            block_height_lookup_jack_daniels,
            get_block_hash_by_height_default,
            get_llmq_snapshot_by_block_hash_default,
            save_llmq_snapshot_in_cache,
            get_masternode_list_by_block_hash_from_cache,
            masternode_list_save_in_cache,
            masternode_list_destroy_default,
            add_insight_lookup_default,
            hash_destroy_default,
            snapshot_destroy_default,
            should_process_diff_with_range_default,
        )
    };
    let cache = unsafe { &mut *processor_create_cache() };
    let context = &mut (FFIContext {
        chain,
        is_dip_0024: true,
        cache,
        blocks: vec![]
    }) as *mut _ as *mut std::ffi::c_void;

    let qrinfo_bytes = message_from_file("QRINFO_1_107966.dat");

    let result = process_qrinfo_from_message_internal(
        qrinfo_bytes.as_ptr(),
        qrinfo_bytes.len(),
        chain,
        false,
        true,
        70221,
        processor,
        cache,
        context,
    );

    println!("Result: {:#?}", &result);
}

unsafe extern "C" fn block_height_lookup_jack_daniels(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> u32 {
    let h = UInt256(*(block_hash));
    let orig_s = h.clone().to_string();
    match orig_s.as_str() {
        "55cea87c22849891b4e8819a8504605cc54314d03dacb97e6fa2aeb3d8000000" => 74221, // getBlockHeightByHash
        "79ee40288949fd61132c025761d4f065e161d60a88aab4c03e613ca8718d1d26" => 1, // getBlockHeightByHash
        "132bfe2ab16f6fab18e1c4d64deb8a748b12c4bd92fdc660e222ab2a21050000" => 74160, // getBlockHeightByHash
        "3ad221f88a172adb8aca647f3a059b008d55fb0dd07f918412620178d2030000" => 74136, // getBlockHeightByHash
        "8d80307b7fd5728799e62e1d461948331dd4b543a330deaf2d98576eb6050000" => 74184, // getBlockHeightByHash
        "97bceb2e855f80bf4f8617ce9cb9206682171a3e1b92d2160ad799d2cd040000" => 74208, // getBlockHeightByHash
        "34336e375eb5f6eefb147479d4c1a4823ca47282bab8a5eaf85aa3b3e4010000" => 74209, // getBlockHeightByHash
        "5c1d03d5dfecf9d5b467eb79d08af9eb4529d69f5590bc7b3c66779271030000" => 107512,
        "45c100e3aa8c91998d7ba0fd9655f95ea93f892a6050679516b9f03f83010000" => 107560,
        "2906e8b02c3aad1aa9176c87b06d5c19d94ab6ad15777a88017d70a4e8010000" => 107608,
        "d7b56611622ebac1f98d11f5b567076459733da645b973581dc2342c95060000" => 107656,
        "e8ae95b476453ef1514c590941616582f527bbde2464974239b32184c2010000" => 107704,
        "8782b2192054460b20585848fc53f9a875e232bd4a4d7f7bfda4b9563a010000" => 107966,

        _ => u32::MAX,
    }
}

unsafe extern "C" fn get_merkle_root_by_hash_jack_daniels(
    block_hash: *mut [u8; 32],
    _context: *const std::ffi::c_void,
) -> *mut u8 {
    // 74221: 55cea87c22849891b4e8819a8504605cc54314d03dacb97e6fa2aeb3d8000000 -> 601bb47971ab483aec1ee77074a785036edbb7ce543d868881aa4e04a39490c0
    let h = UInt256(*(block_hash));
    let merkle_root =
        UInt256::from_hex("601bb47971ab483aec1ee77074a785036edbb7ce543d868881aa4e04a39490c0")
            .unwrap().reverse();
    println!(
        "get_merkle_root_by_hash_jack_daniels: {}: {}",
        h, merkle_root
    );
    boxed(merkle_root.0) as *mut _
}
