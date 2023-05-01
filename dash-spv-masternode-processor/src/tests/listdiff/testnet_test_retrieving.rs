use hashes::hex::FromHex;
use crate::bindings::common::register_processor;
use crate::bindings::masternode::process_mnlistdiff_from_message;
use crate::chain::common::ChainType;
use crate::crypto::{byte_util::Reversable, UInt256};
use crate::ffi::from::FromFFI;
use crate::lib_tests::tests::{add_insight_lookup_default, FFIContext, get_block_hash_by_height_from_context, get_block_height_by_hash_from_context, get_llmq_snapshot_by_block_hash_from_context, get_masternode_list_by_block_hash_from_cache, get_merkle_root_by_hash_default, hash_destroy_default, masternode_list_destroy_default, masternode_list_save_in_cache, message_from_file, save_llmq_snapshot_in_cache, should_process_diff_with_range_default, snapshot_destroy_default};
use crate::tests::block_store::init_testnet_store;
use crate::util::data_ops::merkle_root_from_hashes;

fn init_hashes() -> Vec<UInt256> {
    vec![
        UInt256::from_hex("e7e36d6c735c76603aa26c90c6bbeb74a32e0ccb1f04782380ae93e57fb519b6").unwrap(),
        UInt256::from_hex("f7321c4d9e32ed5978a9ae40028c4ee86c62a0e56537250a86430977a22b3fac").unwrap(),
        UInt256::from_hex("a4f3963f8ba2ede4af3da963e34dc19dc89f6b4aed55e2d7d9478d7a1399b086").unwrap(),
        UInt256::from_hex("e7f674b043d508bca1c901381f376a31c3e51fc22e678061f89a4bc20b043d34").unwrap(),
        UInt256::from_hex("dbded654cd8b1e9671906b0ecd6d36e649795f48b1ba206df9c933984f2e8110").unwrap(),
        UInt256::from_hex("a7b8b0fcf94b0f05c7eba5949420b1f29b5e3906bfd40b80e2df8e5f4ae784de").unwrap(),
        UInt256::from_hex("090d453a429c12bae4a41f32d659aaa39ff0850f4e9b6d2d3218a500766192c3").unwrap(),
        UInt256::from_hex("bddf02456877de4ebcab5276cbfc08a56ef0f2eb41e1290c59899784030ff6b9").unwrap(),
        UInt256::from_hex("4a0209271d167e74ace633fd24a9be00c9ba64ba6155ff3f48e7b33ead172a3a").unwrap(),
        UInt256::from_hex("81bfc1b1d2f15da075f7a65f67f3909aae558e8383d3fe92c0367404563cfef2").unwrap(),
        UInt256::from_hex("f40ea2e64108a42d15185703229c941eb6435ed5c96ae62701a87b23ba036abb").unwrap(),
        UInt256::from_hex("d7a17b2f320d2159d2e72ac77f7b4c70e7a6bc2a06ce9b903b83e34f3a71880a").unwrap(),
        UInt256::from_hex("b12420a04b0dcc67315c8c35401c2bc8bf38f1e40ef510de57db08a596af9cb5").unwrap(),
        UInt256::from_hex("76525f7759aac75f057f7095a7d116a5dc1290d78499efccca28fa5e211e4978").unwrap(),
        UInt256::from_hex("f400dc6bbdc988cc7c4c7076136c0128fb8eaaf0c3990f4cdcf12b9a2864695e").unwrap(),
        UInt256::from_hex("7bf543060b8e88f0006fddd72bc50eb3329cb79836db12d1c63602895762ed78").unwrap(),
        UInt256::from_hex("2e860b3056808f52c2d1fb2b3c6dd4fcdbf28c705fbc60c5c190eede561f27e9").unwrap(),
        UInt256::from_hex("df88b8c121af50e64b998be0226301d6e6108e0afb1573e4fb2cf7e00c48e9a0").unwrap(),
        UInt256::from_hex("26d9d4190e8f3bede12ef76ca4f0a86b60e7c82a10a1b5b0c0b682e6f50c903f").unwrap(),
        UInt256::from_hex("20e27d2e035002641212e3db6be3750b4d0cab6c2faa9a9561778ece0cd6f1af").unwrap(),
        UInt256::from_hex("d1e12436d6002c54c5fc7619ee21ffa75104f8e182b034a0a8cab6727603c15b").unwrap(),
        UInt256::from_hex("9e17bee70eec41c957b0650d95c9349beacab2f6a09d90433656b2833152fec4").unwrap(),
        UInt256::from_hex("b6f381a3c90ad6065e7c2491cdb6cb9ed1965f1970e788e40a2a8feca05dfcea").unwrap(),
        UInt256::from_hex("9063ec4f4df38a217c83b3b067603b09a20a337a4118e6cfd20dd9e1b4a03405").unwrap(),
        UInt256::from_hex("a0c3f1945e82d2f3086d97dd9319629b354a1f89596f79e0d7472ee9e508b920").unwrap(),
        UInt256::from_hex("14d8f2de996a2515815abeb8f111a3ffe8582443ce7a43a8399c1a1c86c65543").unwrap(),
        UInt256::from_hex("2b73a9870f0fd4c7c06193d1847bf0341f44e7235e91439da82106b058fcf089").unwrap(),
        UInt256::from_hex("f5737c290dbcfb90ef3cb8d4076ffa198ccfe0168201c431d5fd90759e44974f").unwrap(),
        UInt256::from_hex("d43da16b218d1b0bb9f1b9b6a04fd88d4fe1d8343de8e35fe02740be891ba4f9").unwrap(),
        UInt256::from_hex("2365b6afdef878ee575df3d390f855679c651362c5491b6601c23933a33f3193").unwrap(),
        UInt256::from_hex("085eca46adf569129effe75c418993dc1e6afe270d8dd15d3bd86a61f0daf7d6").unwrap(),
        UInt256::from_hex("ac69568dd3f21beb1b25210e718d191ed4d4a7f5b4d0846ed7aa724f49be775d").unwrap(),
        UInt256::from_hex("9c5da1bacfc6350e3329a2cfac76ec78d2351546fdc224971efd79e412949375").unwrap(),
        UInt256::from_hex("a2ad729613951727cd68889c7153476fe8ae5983d2a651036248116feaf53ca9").unwrap(),
        UInt256::from_hex("da8b76b0cb9c750062fb221bbf1ae49cf157e4447baae458ee7daa9d9cb101d5").unwrap(),
        UInt256::from_hex("9c1208b08f6ab66f7a554d644320ae22d22d89a50945baa746329da91651e6a0").unwrap(),
        UInt256::from_hex("43903412ee410b5459c7c7080dada4ef96ad166cd6f86f6c270aa5b6537c6a6b").unwrap(),
        UInt256::from_hex("ba67feeb708ba70f6e92c9fdcd44035c365955eb4e05f0173394bea0b6844ecc").unwrap(),
        UInt256::from_hex("04e5d047541f62b75489d679956c2d4ea053aa0d94c4d373a17144655fd97326").unwrap(),
        UInt256::from_hex("4b6f86cf35203abae15d10fb0b4cbe238e0d5e387f305f8e9cba560442221b7f").unwrap(),
        UInt256::from_hex("22e7f112eb0e06f7894293a4b06f36f0d6f98e66818b1c5d09d671c9338a08aa").unwrap(),
        UInt256::from_hex("43471bd379d1cd1933d88ef2842a203de9334b0c02976025256bf1fd7df99415").unwrap(),
        UInt256::from_hex("8cde8673f8228131d9ed4f14e920639a226c51a1884724878b433147dd3c8031").unwrap(),
        UInt256::from_hex("51eaf8f46b4ef368a0349a36a041bcee976aa0ae9cf3cf482ebb0fdd9cda1d66").unwrap(),
        UInt256::from_hex("b920a9e12f09eb633f4084b1c7f4fe9cf4bacd581c1a193fb3db08267d615363").unwrap(),
        UInt256::from_hex("5a2952f5b21b6c251aaf51d1784c355eae94bac540fc5fcda2b41966cde84056").unwrap(),
        UInt256::from_hex("ff43ac83b9c14817b72aea8bb15c69f1ed74f0a9251eb6ae2c7150193e976f1f").unwrap(),
        UInt256::from_hex("fb6b938139b4c2a48ad5bebe9caf5f2083325f6c7413c194d2c0870d0f5a1d9f").unwrap(),
        UInt256::from_hex("b93512d2419f1b38ca6685200bd1f2d6d1865b97ff4d9586ee0be826c39d1a48").unwrap(),
        UInt256::from_hex("d9e7936959cb47f52d36fa5b81bb16ac8bcdf99f26e492184c59678d2a62e7f1").unwrap(),
        UInt256::from_hex("4336afc4a009e6db9702e9fe5859160ab51a53d6666a04caf1b5e487f5935318").unwrap(),
        UInt256::from_hex("cb802d5b8d2e437ab4607188945fca23bc1221788e419445765a51991215f51c").unwrap(),
        UInt256::from_hex("9429e3700f3061bc8c8c747acd9b64145efb6060ab8d3f63b165d3216c7c3773").unwrap(),
        UInt256::from_hex("c773cadb8e78f2b51e3a04dca516e354b026e64aef8f826af957608849e3828e").unwrap(),
        UInt256::from_hex("6128e1d87f61029506196c6813885b500e250c99d9e4c470dd26e6e173c9e73f").unwrap(),
        UInt256::from_hex("ac9cb4555952ef5d7e43e7335ad9379e37e0830700c9f2d9b7fad35674bb6b22").unwrap(),
        UInt256::from_hex("c57c2d19e8e17c0e192340b30bb7f81018bded16dfbcc41ad4473aa37cf192dc").unwrap(),
        UInt256::from_hex("c26dee2679812dd47f2e44be1010afdb18e102389d656e62201d6ae591dd242e").unwrap(),
        UInt256::from_hex("309118674e4e7e58c1671febc42350f46499f322f0d5c486ade5e351919100cc").unwrap(),
        UInt256::from_hex("3d138eeaa5f63b564835d4f0348ea57d97486cb835a084be919dbaf6ffa6ecb6").unwrap(),
        UInt256::from_hex("7e060ee3526d1f6babcf087f80d58861124d7f4697c5574b3c455fb65a06fca7").unwrap(),
        UInt256::from_hex("273a74505e1f361f4b4fb96ed11bc1f98c5397494b6090fe04fa67c82fb94cc4").unwrap(),
        UInt256::from_hex("73b309fc6c9799bae0358ef888b068a7aa20eb1de2383bd7d3d9c030c1e8b8fa").unwrap(),
        UInt256::from_hex("6c90b5675af7d0c073471a6ce1b6403478bad46f1794cd4e4cae1c78312476f0").unwrap(),
        UInt256::from_hex("4468006f469e588b71db460f7ab151255e231bbb850c6a43d419c8eea4acba94").unwrap(),
        UInt256::from_hex("49b52dece0b1657323b16ddf09dcc3bef804635a32f2de2750afbc0574a20266").unwrap(),
        UInt256::from_hex("c6122878b9d65507f4c3f1060143ade134e09263f55a791c641a76057d3aea67").unwrap(),
        UInt256::from_hex("5276ef5a097454b326aab2d6d09bbfd53989fd5d54d7bb98f314480449e53988").unwrap(),
        UInt256::from_hex("a82a2449f8af0bcf93976415fb4abedd562f0c581ab61c541023e20b4e2d7fbc").unwrap(),
        UInt256::from_hex("c0c8ddd0a59f54b75d3f3af57c721e396ab6ee22a3579ed91a96fe25c87a04ad").unwrap(),
        UInt256::from_hex("8ada6c15af35ad39cdd731e38c7db0a32753fe7a586a2084b7fb59b559df78a1").unwrap(),
        UInt256::from_hex("d09bf256459c39b774def612f9a6354d3d4f5d10ca4300b8b841eac3ef542382").unwrap(),
        UInt256::from_hex("688d227649bbc5155824dc8013f0482eb7697667c1487f9dd459e6c3a6240509").unwrap(),
        UInt256::from_hex("170201082d6d01c930921f22aa8215a2bd8731ab1926051beaf60a99b37676db").unwrap(),
        UInt256::from_hex("914d162c56ea339b19901c4a44cd88bcffc683caabe54a353789c09cea9e9f56").unwrap(),
        UInt256::from_hex("d606b85cb03ed5b11548a6cca5d85b31d1a2b3543b4678f7c2b23fb915affcca").unwrap(),
        UInt256::from_hex("284ad64627455d4ca8f5f9075018b84c14b548022b19c63245bb773d5bc684e2").unwrap(),
        UInt256::from_hex("1196a11122b202219c9c6d06e0ea6bac8045f188081c49001bd3cea59887c52d").unwrap(),
        UInt256::from_hex("cfb5f3cc7c8c5a1f3ddcb5c7055a8fbeac17a3cb394a9698f291b9ab7e145454").unwrap(),
        UInt256::from_hex("d48a6a84a588120653fa31532c9e18ac90823a119a8f25a32a6a798d1d6888cc").unwrap(),
        UInt256::from_hex("6f43509c3329c761cd36d2045e9ac246f04983c929e4a7e7ea0046b1b5dd2dae").unwrap(),
        UInt256::from_hex("4e4096582e781e63e86dc6e374884972ec33907add0d4a71588c6bc220bec7d7").unwrap(),
        UInt256::from_hex("01e4554ca14ae334274026dc6f7f586e456516a54dce16805bd3bb7df4602be2").unwrap(),
        UInt256::from_hex("ba54a9bd1ea26094db46ebfbd215adf9a4f03f681ef186c81f3d7f1be6ab7a5e").unwrap(),
        UInt256::from_hex("529b80bf5566acda6af74911b8ee602c044aac1935c66db48cdcd2553586668a").unwrap(),
        UInt256::from_hex("e1169c915e2d33b6197f09c80c2b4940662744dd2cc8bd7f564a480a786e307b").unwrap(),
        UInt256::from_hex("b73a305b69cca3ef1eed0574093a8f5c93daf675aa38a5a0fc66a1e024ca140e").unwrap(),
        UInt256::from_hex("f368d7768bd5ac608ea4e0cc99ffaa7bca2260be54c4b96e80bdb9e936c0e14e").unwrap(),
        UInt256::from_hex("00335d1c7c7bec7ee9538c254e57e4cc5dfbd2f512f7dbbcd52d67b48a96f6a3").unwrap(),
        UInt256::from_hex("68685659e87e5994e8d7be1d9e1405e818c713650a73ae940e1ad110f989bd05").unwrap(),
        UInt256::from_hex("1e97b8da660008564174cf5e75ec47fad28e07eadbe2ad60f49376378387926a").unwrap(),
        UInt256::from_hex("6db1b8db0170dde4f64403de2e9cf679d634290047201d620ea8ef4fe64bb3e4").unwrap(),
        UInt256::from_hex("88135a11dda699fe47c5d382a4813437906b7903518b0b0a24ae713468584538").unwrap(),
        UInt256::from_hex("f955fca6afc84689f311527462922e525bf5b598061545fd9649b61f83118d5f").unwrap(),
        UInt256::from_hex("51deb1d22c56acdd5452ccc440e8948aeb4d94acd95a8ab54e49c209016d9fd2").unwrap(),
        UInt256::from_hex("ad86aa32d2fdc821ed9ab248392e3e49ed0d28cb96014d2d35eeb9a7e3013ab0").unwrap(),
        UInt256::from_hex("53d910b00dc4c096b49fc1165896bc7341e7798299e8f2fa5518494c4f57d699").unwrap(),
        UInt256::from_hex("fc2bae55ae8d99e97de1ae41170e0e0a39c067fab6e263da2d9eeca58695c153").unwrap(),
        UInt256::from_hex("b18f901af9ed2cd700338f3f46c8bfb08f15fb557cb6c49f097c52922748b344").unwrap(),
        UInt256::from_hex("0bb411a461d7693a8de18f0a1924e0628db7650e2de5cea744e284373af07575").unwrap(),
        UInt256::from_hex("9cc114d5630efe13347281298b746d7bee25befadba89f6839afcef6486e52ec").unwrap(),
        UInt256::from_hex("25ff43a78cb15d0cc379fa053f7c7607cce471c93a9f3be4939f94102e38b408").unwrap(),
        UInt256::from_hex("07d5bd2519b091171e0de1c685bcbaf63cf34866c068ed600e9192ad248aa72e").unwrap(),
        UInt256::from_hex("9673c4295754f87ea1ffcb250c7c7fede0b39b08116d013582c9cf543e125e8a").unwrap(),
        UInt256::from_hex("854675add071bd9435b11a8d1fcdc36be8d8ddca7c59ae992e8b0279cfe5fb18").unwrap(),
        UInt256::from_hex("33fbe8fe4481da36b2ed4e8966966cd36dc0033c34d9c299dd67fe0d10fc85c7").unwrap(),
        UInt256::from_hex("0d8b1c5d932aa8e58b273212af3c89bd159310efd65d902aeccda04dd6e4967f").unwrap(),
        UInt256::from_hex("f6f95a78727f17468c3817c45e171d8291cab0288fffd34ff05074c246634bd8").unwrap(),
        UInt256::from_hex("7f5c0f3296a6e22e533a90fd59b1c4b9418c397045954b31fca1fdc763fc8a86").unwrap(),
        UInt256::from_hex("65aac8656da4feccf2ca3dccfbc4354846ed97f51c1278022d539dc0977bb614").unwrap(),
        UInt256::from_hex("ab082616b5da66b5614d1b97c3759276ad216944c8ac519d390679aaa0b2056a").unwrap(),
        UInt256::from_hex("c9fbba4dd0e6eaf43d8d266111bd2a0701eb7cb797a8ce9858217ec00fbf1e14").unwrap(),
        UInt256::from_hex("55b18a7352a8996a9a035afefb2d1589c479a309b0a1fc52875e406898126b16").unwrap(),
        UInt256::from_hex("2c1b54ec24b451f13bdca55abe6f542f8b3d6553509240f9710f998f04aa9590").unwrap(),
        UInt256::from_hex("71ad8ece820b04ef12661b4a1ed463ff71fb37a445bd250b66b65cadda743a5e").unwrap(),
        UInt256::from_hex("91d1e47dd69fe5ef5787086af9a269a52e71a5a8963f49f8d4c384ae99ac98a1").unwrap(),
        UInt256::from_hex("f9e7c374d1bf171e214ea8f96f9f0ebf52c960846792546df11fb5a2b4da6321").unwrap(),
        UInt256::from_hex("69a38bb2c09e55dd6e876046992e7e389327a13fb0cb479749ee09ed3cdb57fb").unwrap(),
        UInt256::from_hex("99964fe5d1e7a5ceb19313db8894758fcc4e48a6baea314e40cd24b1898ea233").unwrap(),
        UInt256::from_hex("3a5ce3c8c32c209cdf12a234b42aa8e38bf2bd041650a5043e88a3c27ddbd403").unwrap(),
        UInt256::from_hex("c1e2203f2f39163e6363dcdeaf3192c52f2e39addc73b3d34783cf17fc949f45").unwrap(),
        UInt256::from_hex("b178ceed4a30285679797155aa6fdbda31663a4d9e8bcba981c332bdeac022e6").unwrap(),
        UInt256::from_hex("4ff3c2e76b43b8b76c069c2c43ef374410ff4c6ae3bfeead909fcd2ae4930021").unwrap(),
        UInt256::from_hex("57e04ba7dd47408e941af0f637646f543181c4f6fb198901f0fe1e3e5d39661c").unwrap(),
        UInt256::from_hex("f2ef52f9f83bcb0895b57df9fd512d03a3e10ac8791d532b9fce65aaba9834f0").unwrap(),
        UInt256::from_hex("b845fe54f1f9754c1fe8cc6d075d2f5bd05af11cef7353b26a3acf21fe8226f1").unwrap(),
        UInt256::from_hex("bd2d69a7eeca5bbd3070f834eeb5612fc529ad22539622e519744b038a984387").unwrap(),
        UInt256::from_hex("3f680ae7616d471729ef0cfc84bca6e0c3c9c19cf15e07a5a451074ba0bdcd01").unwrap(),
        UInt256::from_hex("0c1afa037067adfbb4874a101663eaf42137266d605b2873b60983beba87cadd").unwrap(),
        UInt256::from_hex("bd51b858dcc711ff89fa1499981196be4207640214a50cfbdfccf6054523f987").unwrap(),
        UInt256::from_hex("921b214d0c901a7e890aa3aa5ff16b782e0c25fe2f3dd8fc525e348e1e8b63ce").unwrap(),
        UInt256::from_hex("4ad6471354da794e9db769e9465fec5db9aa3f575b4ccf8c4ff889ecbdcdb8d0").unwrap(),
        UInt256::from_hex("f922851fcafc86876642e29043dfdf6684b410830d203b3bc02d10837751c44b").unwrap(),
        UInt256::from_hex("f58855661d972309ff106a6346670b362172e0d7bc13ca82fe3515a48a68302b").unwrap(),
        UInt256::from_hex("370fde22aaed5e2b0da27043ac2ff0608b52b29780e41ff021061030443ae20d").unwrap(),
        UInt256::from_hex("bc1fad8ad7ae013eb72f2b7cdd421b667b02397ff3e3316fe153f032915d2601").unwrap(),
        UInt256::from_hex("6668a82cfa8a5619b2719190c1de41934674ba37f1160b8e92c45e97addb3a8d").unwrap(),
        UInt256::from_hex("13ab18043e25d18a0fbab47e74d6f6a338b63d7bd4167145511b2cde768ee01c").unwrap(),
        UInt256::from_hex("d8a7ab90f1a0acb0f16c33a615fe603ecaaf7cfa9ef8f83578cf9bff14c14509").unwrap(),
        UInt256::from_hex("c6ccc10908f2cb10f524c1fe2bd38b9513ff7713e5377304ecfab9afb22c8a2a").unwrap(),
        UInt256::from_hex("6c726aea7c34bbaebfae9bdaec36d20af9df58b2722d6221df5acbccadd68ca8").unwrap(),
        UInt256::from_hex("d89e020e377f5d017140a8ba6e26d4ce9e419877bd756eb90e4ea048d7d538b8").unwrap(),
        UInt256::from_hex("c057c8e641ed84d90e8338a79eeaff0997fd1bbb01c5b3dcfea05b5b82ab3502").unwrap(),
        UInt256::from_hex("8884a714756a168032c6372fd0c58cac040759ae2f9742874cc70a9d6ab38fde").unwrap(),
        UInt256::from_hex("8d66127b38f89c264922e288cacd9bf2278876f96a0d5c4b063be45e60b98c29").unwrap(),
        UInt256::from_hex("6aa97c683f72464adf808486ae06780abdcab6606be1a1a8c5c5cfaa70eb6d65").unwrap(),
        UInt256::from_hex("4991813af0a747f5775646a48f9d4115f7e9f8bb239c58148e24c1da971e116c").unwrap(),
        UInt256::from_hex("6e793293a676277c97c8d8ffe25a958e5627caeafa270da4dc9b96e2e88379ff").unwrap(),
        UInt256::from_hex("572708276e69de93ea18e7ce4e66bb621ff9f3267ab7cccc536795689fb1fe8c").unwrap(),
        UInt256::from_hex("f0ec5589209991c869034f0e577f7413eb73427d0f7b07a957c1385e4ffb8014").unwrap(),
        UInt256::from_hex("efe21840c8bb668d9828c9a99712f560f032c1f471e982a27bb9f3194647cd26").unwrap(),
        UInt256::from_hex("065e829bb4c5e3c3890cc7890f7161e896866f740dd5ab9f7800eb87e7927cb9").unwrap(),
        UInt256::from_hex("642fd11e9eb7727fee1aa908706d30c09be7a8bddc5fafb49a2431e9625d8d7e").unwrap(),
        UInt256::from_hex("4f39065ecda5dd3e5293001ad0b81d9a0a0fd589aceb5ac02891e47edf73b869").unwrap(),
        UInt256::from_hex("6a66582ad87329ba7c06aea20873e2880c5ac48b4fe36d7357de0849ae201436").unwrap(),
        UInt256::from_hex("6f36e52b6e5d9b45271096765b2a44ae84c783f457458a2d1700561ae6cf543c").unwrap(),
        UInt256::from_hex("9662bf990453a04251b26bf66bf9b5ed90ecf83f3ba3f8f66c317502f4d866af").unwrap(),
        UInt256::from_hex("1c3580556e5642cc25cc674e8cfae5c2b65848b34cac6809ee683ed247323195").unwrap(),
        UInt256::from_hex("a8098656e72dfff7dc0a3aeff7d3d9f97de9b1eea3e41786818c7f3f9a367780").unwrap(),
        UInt256::from_hex("397085f4cc1e92a17515fc326c65234f5286b555098754d38693f027b9953257").unwrap(),
        UInt256::from_hex("3fe0d1ed6efd7900b75b369effc8aaf1cfcf47324f4c848a8aeb5c615bb11c2d").unwrap(),
        UInt256::from_hex("a08514a23e8339afd576ce9f09b38bb9af6369dcf00939b0f4b3d3913140e6da").unwrap(),
        UInt256::from_hex("76d14be3721e4534db2f00a0a2cd486603a043eb0bf155566987146418e0a637").unwrap(),
        UInt256::from_hex("8cad5bff566ae3a38e88680430825ae6cf0f0812df0c6ecf4b94c15a52c2f2b8").unwrap(),
        UInt256::from_hex("d0ddbe1971be00474d8d1a63da8b84c5bf6c2f0a3c185796502344092921b493").unwrap(),
        UInt256::from_hex("6b4f66e57cf5bdf6a0f3accdfefe51e8e493ff254cfdb093b76968a660fe5878").unwrap(),
        UInt256::from_hex("de325fab1171332d31ca8142ff4ab49af88f6b6bd651dbbd263fc3aabb7ef84a").unwrap(),
        UInt256::from_hex("ed4e01e08bafa9d8efa778c96da03f7341901933dba8b6557c67ee2df501e48f").unwrap(),
        UInt256::from_hex("53289d2ebfd4f37a685d2b0774817e7dfb943587540e85524265bb5916c42986").unwrap(),
        UInt256::from_hex("c8f2cb8e9eb7d1e4448454f23a03166c55642ac27ba85adb1f738e0353710fc3").unwrap(),
        UInt256::from_hex("daee1fc2f3509568c0f87c230314282bc5f13a9a1aa1c661f849762da8cfc1da").unwrap(),
        UInt256::from_hex("1d7d7a71d82ad285613c46be70c7a7d1cd64a5832c8a9d0f8e1c3f19bc2803e7").unwrap(),
        UInt256::from_hex("867620cf3a1fec5f455fbdb3577782eddde4218e4cdf3dbd4b1ae63867e16b29").unwrap(),
        UInt256::from_hex("80a10a29da93fefd8a3bd9fb508a63ec65e3a561137223cf4af43050fe495d05").unwrap(),
        UInt256::from_hex("790ccf70d7a4faa77dc199fecbbd2bac725434d9612ffd12f7da09d751714cc9").unwrap(),
        UInt256::from_hex("d58cb91c184233d1f07f22b8eeeaf19b88df9d0f4e7508bbefdff6c940383c0e").unwrap(),
        UInt256::from_hex("712927b1815317b2adca5c953d69b4480296715a1a036f1ec0e6caace1c19f03").unwrap(),
        UInt256::from_hex("c2ec1153a273d5adfff01bfd08f595baa0e8126481f2617e4ea37011c55216e1").unwrap(),
        UInt256::from_hex("c10c93ce089ffacf0b12350bc3c835bf027e42179d4d4c66286aac40e9629d63").unwrap(),
        UInt256::from_hex("9f3fb5170bd6742bca967e68abc4f326215a8bd3f51a6c678c320195f64b85ba").unwrap(),
        UInt256::from_hex("f3385257c512a405fe3e2686934bb82e94843f9013122604474da45e264f7f16").unwrap(),
        UInt256::from_hex("3c01accf8fa4c60c3ff14d2c5874f6d2672cec4efb726b80e80415f7bf03175a").unwrap(),
        UInt256::from_hex("3fc847893e9e6250aea160abaa64a7396615fcb9ca40c55f220718d55ffa54f1").unwrap(),
        UInt256::from_hex("c0fb0bf61a43b1823eb9d4446f0c8dc31a0e8f9f11727b7762b48b378e0b702f").unwrap(),
        UInt256::from_hex("813f501c2f5071172be0bdf0e6d9916856582f6df4111654f6a33ceb3cf86b93").unwrap(),
        UInt256::from_hex("cd1f36bf170c78ad6fc395c0259b8b821d325a6d5ba18031fc571c313affbe38").unwrap(),
        UInt256::from_hex("cece35461902d3d93d6b92d1ba6e987451c89efce3546eb0622843540c2781f8").unwrap(),
        UInt256::from_hex("860763c4d65bebc2f6b3da76da0e69c62eae470f2d7ff17be192dd266d90c777").unwrap(),
        UInt256::from_hex("73cd1a4672f37ada0994d8a96a1451af4a812fe2283fa751bb20b6fef261b57c").unwrap(),
        UInt256::from_hex("4b14b7d9e51effb57f5f12dc03ed39c0f10d60774d220482b05576c855fdf137").unwrap(),
        UInt256::from_hex("1209496fdc7a1934161651143833172a56fc518a7cfb64686bf1e9cff98f52be").unwrap(),
        UInt256::from_hex("37665da935dd92d2164a51b9209576b4ca2c700217789139cfc2551b534a8c97").unwrap(),
        UInt256::from_hex("03b452abf55fac171eb2d0a28d9dd8f3c3f0d48f25ba6f6005a9f7fff6399174").unwrap(),
        UInt256::from_hex("1f53eac62ae6e3e512d497fd2cb1386d9cb5126a4fc7bfac481fd57762418e04").unwrap(),
        UInt256::from_hex("f0c9001af8e472413811d580adcf5b204c345e060771c94735c48147d179b896").unwrap(),
        UInt256::from_hex("0b1256f5b6af45e66a4095fd23ef53357f0469aae79fed58838ef5170658b6be").unwrap(),
        UInt256::from_hex("f75b2fade451c4ea61d0938e1cf1c06f5d2bc3a97cf38f383d7fb5c2564f0072").unwrap(),
        UInt256::from_hex("5d2d58458a26336dc2ddac77b3bbaa9f21670ec74af01956ded4aa4aa589f9c0").unwrap(),
        UInt256::from_hex("4a5987ab0577b13831d3e18fd048fda9ecc4d2bf3e1f7f869c879c9e67d2a73c").unwrap(),
        UInt256::from_hex("c6393a369aa9f99e8b8ce5e60e10b60c4535d6e47d16076526626c218eb69498").unwrap(),
        UInt256::from_hex("f23c8b054a28f4f54a43e14269b117966d51c8066b4ee69bb5368902a24950c5").unwrap(),
        UInt256::from_hex("f5da85fffab1bb2ca472adb14d29f9859ea7f10a73c529530aea4eaabcfb4354").unwrap(),
        UInt256::from_hex("6d71c1ffa65bfba8007291049009187c80c9a74a9804d764c0df67652ed9c173").unwrap(),
        UInt256::from_hex("551e5c5366cb467072b0de54b4e9f9c14bcb2426deb3b1ceb8ea621df67f0177").unwrap(),
        UInt256::from_hex("9ba2904cb04f448c71761ff4ebb3dd208a9e2d9dbd6189304cdf6b9b0592214d").unwrap(),
        UInt256::from_hex("9af29689982ecafbe1e2c89358cff2d42376115d6aae162c56b8ef2e6f19fdbc").unwrap(),
        UInt256::from_hex("0d5058453a5378f5ed4a37ebcb7720a283136854655e19076814b58206d752bd").unwrap(),
        UInt256::from_hex("700d7c952c473f5f430a3ddabb84b0ef67a79e31440f1b01e849038c04c777fe").unwrap(),
        UInt256::from_hex("fd2b4d505c1eb7dfede267a5cbca136818873a91ec1c57844af46f7dfda8ae0c").unwrap(),
        UInt256::from_hex("c12d21e616ccdcb84bb2ff49024d102376770df3c6b68793e2ae54176a058de2").unwrap(),
        UInt256::from_hex("cc3e43df79ee9c276aa2d3d0437d6be501c114b8f7162f42fdd00f0e4610a42b").unwrap(),
        UInt256::from_hex("ad22b2e3b83b535e1b0b9838d568ce24cb2d76fe147b214cd46ca9ea7917b4b7").unwrap(),
        UInt256::from_hex("655fa1c0ed68a37a63b1f810f51706b5e16bad137d85c77102c102652eadf4ae").unwrap(),
        UInt256::from_hex("e4dcd1e16712584c7c17d17ae9329f607261c447544660cd8fa760bcee86facf").unwrap(),
        UInt256::from_hex("f1313d63c8b91f7eb27aab604089e9d75f5fd9eca7001df2c1158283adf7b99a").unwrap(),
        UInt256::from_hex("56333720e05a851e37d51e9427e66a7c9380e2f5ccccc84e359e64bcf17f3857").unwrap(),
        UInt256::from_hex("be860723b5bada008cf880010c468b8e4d218a0c2187f6cf2e3e28df4cc36d97").unwrap(),
        UInt256::from_hex("ddb5ce9fbee2726ee9fd8f733acddb786519290f1b88b8057c97e5d43ff96e79").unwrap(),
        UInt256::from_hex("f28cd8cf1afd206209d7a4a2caa67ee014640ac28f05940e695a0b316caa071c").unwrap(),
        UInt256::from_hex("08da2ce3a77039fe2560127b8fa0b83497d21c153b17a16dc068780f145f44f5").unwrap(),
        UInt256::from_hex("6d7b22af5c9acd25d80cd3496988f97fdda962cb632be29a9f675b8f129d8ddb").unwrap(),
        UInt256::from_hex("f9d2d5fe512f846282a05c18a9ff2b65b2964efd1c942101a1b78430162cb0f4").unwrap(),
        UInt256::from_hex("26b6a446118c70f83ba808d69ee492c335761038e1fd9e9d81caa2666dd51cb2").unwrap(),
        UInt256::from_hex("595a0175cd08f6eb0c0c7660a7a3f6ed2a8e761b62ddd71440bd0c744e951d41").unwrap(),
        UInt256::from_hex("89180c0c550bb1b9786c4cb32a7e079f2de94739ff4d4c0be9c803f99abeba26").unwrap(),
        UInt256::from_hex("95e86a0cf83c640bb805f089b664242af8a44dabf0d62e94a9ca10887d876d15").unwrap(),
        UInt256::from_hex("64b86b8b48f4d9f884368d79520a0adcc239687126bf245400ff81a4485e4a7f").unwrap(),
        UInt256::from_hex("ea4c93192dc54a4a862a992a2a7a47e77ed8fd4c454ea92de663b77ac5183e6f").unwrap(),
        UInt256::from_hex("7737062c9a475de568198a10b0b619d6c8f0afb64a95ba9562c56f2d2f271406").unwrap(),
        UInt256::from_hex("b98bb14865b11f9b2f3e0d54cbca6dcec2a1dcb4d256e5e0f1951b434b9196e1").unwrap(),
        UInt256::from_hex("15b4ac699090a4d4f942b52382e8263163c9d0f2ae84d52f3592d82024c3aea7").unwrap(),
        UInt256::from_hex("12a5b142ee53e7382d5a5bfa9caf798b8c75e499c6fc4c9cbcab753e1cc252b4").unwrap(),
        UInt256::from_hex("254eee874d58ef50c417c32bf6e5da4191626a6affd0f381ece5a67518a2d11f").unwrap(),
        UInt256::from_hex("8d400a4e6b445001caad5ee070aeb2aac8410c069c9cd6b976753c4e50e02b0b").unwrap(),
        UInt256::from_hex("ac68e2d6a1da9172e2000d6174d6bd1a1dbd9ad27b36a30ea174d0fa3ad6134e").unwrap(),
        UInt256::from_hex("baaa1978d1b56bcda53a03485869216182d51a5a685c00e9e5bd7a2602823c58").unwrap(),
        UInt256::from_hex("49453bbd6d06594455ffcb76b1ea91d31cb6d1f63fa5c42dae24ae7498c1a667").unwrap(),
        UInt256::from_hex("c0d25334b89c545bd27cdf91bb2219ac3b336f12f8630354193c8d790cc73865").unwrap(),
        UInt256::from_hex("1f4ab767f64d321f61d0a0995faa3096bf54742d62efe594aeabba6dbfc7e830").unwrap(),
        UInt256::from_hex("dfb3586607b1251ada77cce997f8a9aa407ea4e44bd0d93687f83710d95efc39").unwrap(),
        UInt256::from_hex("a3a56d57d83efa4fc437291f5137f7c18c57b21bfb760e3ad548f8c594f9b34b").unwrap(),
        UInt256::from_hex("5bd19476a478c74785daba30e91d6371c3bddb7b2b4758291938a4dc310e3ea7").unwrap(),
        UInt256::from_hex("22eb46a7554535adf84d3e0b0fb8321027c1028af40ca264867167f93ebf73a0").unwrap(),
        UInt256::from_hex("2735e353b9d1074a87ed8c0f97f1d025c0305f4572a38662809826652ce6f351").unwrap(),
        UInt256::from_hex("f61ee221a1aefecbf038feaf98dd719036b05039702d2c7b5d4fc62e4005de30").unwrap(),
        UInt256::from_hex("1ee33e687bfcca12fbbbbc7463b7fa5855cf3611609f3947b8ce517a65f2aef3").unwrap(),
        UInt256::from_hex("329a0dd5a23e0a524d8a5f24b4ad659aeed8ec10d6679e15ac3aea3d96d9d7cf").unwrap(),
        UInt256::from_hex("c7492b5e02229690c72c9f0eaf77abc10577e01c6bc67a3882533a81dec856fd").unwrap(),
        UInt256::from_hex("8596f685756e9481497eefa7a15c7e32468bf434ae3a11651b19dc5b52b0524a").unwrap(),
        UInt256::from_hex("4fee70dc9e8d95277d6fc186d9ace9506f3bc791a74a4ddd776ff631329cc791").unwrap(),
        UInt256::from_hex("8ae4b3d80009bdc2a8dbcd2c306fd7af87811e0bf281fa43335d1795b0c8bf1e").unwrap(),
        UInt256::from_hex("3bf98b8bab2bd3ba697abe4aa8ba68f3ecfa9a94ccedb684ae550f8085142d85").unwrap(),
        UInt256::from_hex("e0dcfeabe40d2bd7a6b0242ca6930e3ecb5d7e4995d63e57048e9c550bad2b00").unwrap(),
        UInt256::from_hex("e184bb5593f302b5f7463520d3ab13db94788d9000e774b1560299672914d511").unwrap(),
        UInt256::from_hex("65b25e426b55fb2ca44c95b2738d775b97304c0af18745454f92d59f96512e80").unwrap(),
        UInt256::from_hex("6147b362effe787ffc465c0d39e9737e068e1b5df94273037d21a39a4eab6d3c").unwrap(),
        UInt256::from_hex("d43625a4e687020f86417bf7bc2412711b49fa27d0e8a99a15543a6a5e5c8531").unwrap(),
        UInt256::from_hex("58473e110a5ecce2aa49cb992ad5865f24929f7e5f081f3f8aadb20b979deca0").unwrap(),
        UInt256::from_hex("c5f1336a330ee42641f340ae20d3c435ad54b3366e656b612250f7e70e2745d8").unwrap(),
        UInt256::from_hex("262d20a6c7b27c31f5760296efeced87b44e94e9c4d34ddad28e659a84e16241").unwrap(),
        UInt256::from_hex("65cfab1408b4c8cc9a875960246358378805338bc1f434636dc94e99acccfab3").unwrap(),
        UInt256::from_hex("b7a90269fbe5f0fb5ef28ad82f140ff3c121adb22bc236dc741ffab517bb69d0").unwrap(),
        UInt256::from_hex("7d9a05de3fabe0e82073c8eb87086c758dbf1d6755a526a3e1b82d7c008b29d1").unwrap(),
        UInt256::from_hex("78e8e29ad2c21ca4fd9065603651e20564a00b902e503ec39d8b198e1eb9135f").unwrap(),
        UInt256::from_hex("e88b32419da1313a72949644a6df7b0ee0629999915203fae9663567d80f106d").unwrap(),
        UInt256::from_hex("918ccdea0dc7ffc311d0c6751d7cbf29ad6cde40dfce88d7e011c85490ecf41b").unwrap(),
        UInt256::from_hex("f4917d877c34e550f07f63afdf6393319863b1f04704c312a3dbae693bac6383").unwrap(),
        UInt256::from_hex("35dcae6d88a079188b8da96789104dced067c57a82a22c7e10a606d7b4bda1dd").unwrap(),
        UInt256::from_hex("b9cdc6e58f675a5ae083d7f6bda44b88961dd8946e0378324ee4c54c99479b27").unwrap(),
        UInt256::from_hex("6d36008e4492d6b490c0019ac7fde37978bdda52e47b53f1f679f026addbdbe8").unwrap(),
        UInt256::from_hex("696b9c0eac6c9e500d0464ee2a9c0d45ceb9b2a895493a9e8596217cd687b64a").unwrap(),
        UInt256::from_hex("b15d52da7e1b73631b26b50fc322d909cbc1cde174581277c53ae2be66efb308").unwrap(),
        UInt256::from_hex("e969ef9630cba22800158836c3ef72c02b4580d02fb7c4a8afa2f5b85340fd8c").unwrap(),
        UInt256::from_hex("914910abd631419348765699ecc51c01aa194e5f7c4aa786b16ba270976b559c").unwrap(),
        UInt256::from_hex("2bb92be2f2dfd92c431e9950ed100992a7246253ac9beb85364b12912298139f").unwrap(),
        UInt256::from_hex("d686e0a13cfea6c41de3f3e02a7e08f5ac64389f736b0414cc96fb6e63c0b0bc").unwrap(),
        UInt256::from_hex("af7f84a7d9f0fac4bf5aeef1b6e89ab45cc91a62b33b51c6a9f87e0b59cbc3e7").unwrap(),
        UInt256::from_hex("4cf8764ce3bcca1b752162c9b4924d401bb735d64bc297258cda4e0bcbce6843").unwrap(),
        UInt256::from_hex("473a8b34eaeee13573ba87646b26fa6cd673d462ce0a939e4d568f04a07156cf").unwrap(),
        UInt256::from_hex("f8bdfe07ee2578af3a6f5a78e53f47eb8017769c30a1dcd01459aca454a8cf8f").unwrap(),
        UInt256::from_hex("d76ca1d0b272afc2a995bdca67f7926a98eef012e15cdc7578b033d3caec6909").unwrap(),
        UInt256::from_hex("2643455f725d2f41ceaf9682ce87f1ce91cf51be570224baa3b81321d43f83d2").unwrap(),
        UInt256::from_hex("a2352d1d1f18afcb23b015e17a7ecbd696b90033cae2edb259588f31e39d4f3f").unwrap(),
        UInt256::from_hex("7f40fbdf2c94c086cfdbf8063c2cdfcdce649e5dd0ba22d446b0fa9af01eabf0").unwrap(),
        UInt256::from_hex("f56eed8edcad82751ba7bf11e91d220e6214deb12ca3f84a19a78aef60469e61").unwrap(),
        UInt256::from_hex("6ce15a914b2019b4d2f64b8c258ce05ee6bfd510fc8b7cb9e6d8185d8caff1ad").unwrap(),
        UInt256::from_hex("c056b7c79fa67b06632127dde65663d5b16cae84e434fa15b4d52afb756a20d6").unwrap(),
        UInt256::from_hex("a36ddddb232c549ba376b4e9909a7a07e48c8c10e9fc3c09c966e488c6286915").unwrap(),
        UInt256::from_hex("661adf362740f4a3818acb70978126fb103b6360ab1f51d0662fca81e3b751a3").unwrap(),
        UInt256::from_hex("bc0648cfc85b6ca1f79896208bff33b61aa8948e30932f27bc7ce55ed789ed41").unwrap(),
        UInt256::from_hex("23abc5fd590e8804f0ca4da3cabd3e4a6586862f0012476dec456b18be67ee31").unwrap(),
        UInt256::from_hex("74afb7fff229c6f149872521c071bb80323218825aa948d1ecef862e2f9c51e7").unwrap(),
        UInt256::from_hex("b6b1417c193daa7b97ac20a6353831be4feff04bd0325efab21a9959e422d482").unwrap(),
        UInt256::from_hex("f5f79d34d9c10df30cfc024ed35607bf76dc76f24e4f1e57427b8bed10a5d836").unwrap(),
        UInt256::from_hex("e70eafd0a3b0b4a482dade2afa562199ae86290fdc9d304c35e26d5c1562fd83").unwrap(),
        UInt256::from_hex("2cb7ad706b857f3c17f655ed84d4df69885fb8e701454ef2ba163613692689bd").unwrap(),
        UInt256::from_hex("a732dc34921014c9f4be9ec8e91791894ef324d7519a5e0038d2f8af848adaf5").unwrap(),
        UInt256::from_hex("e5ff0e8252166e03d1e650f24d685e929b12871d6790a9f664bedd4d1da0fa79").unwrap(),
        UInt256::from_hex("69b687ada4aab43d5798f72402e233b9c0e414056f2daad894411bece75db22f").unwrap(),
        UInt256::from_hex("095d86c2ad5998c0a5dc722eed54b2f18724f91121123045aefd64d78c7606bb").unwrap(),
        UInt256::from_hex("ba53501b35ff90f93d6cf99462ef96211be215362eb9f9b6c65464531e35d0fa").unwrap(),
        UInt256::from_hex("cf42dfaa9522c7af2c690328be1d9f70d9b14c6207d1614ed96242585bd00dd0").unwrap(),
        UInt256::from_hex("f1511a8dc45d82bdc7ee6cef25b574bf58e1ee60be4e01a63d8f12afea437bab").unwrap(),
        UInt256::from_hex("1b8b81cd633611b4ae29ae2ed2c0db9307b5bd64c13ad36ea012529cafec73d6").unwrap(),
        UInt256::from_hex("56ff0e9a2150076b03be9a216225bf2e5a2387e1e94c41e38f337e25ebe2bec8").unwrap(),
        UInt256::from_hex("0d0ae02f877d0a1f9e8cb33b3ae876b763dc1b062d1583ef4ff75d2300048038").unwrap(),
        UInt256::from_hex("f1790b1db1b06f4bdc069337ff851f231151b503ca51876e351e4286f1a87b50").unwrap(),
        UInt256::from_hex("d92b9b5cd6912347bf67c86be5e1c878fb71cc5e4d756576cda0e3970756c83b").unwrap(),
        UInt256::from_hex("6f176bc3d18c3cd4f663249af2b45facd4c944736f6babd2ba7aeec5b88da129").unwrap(),
        UInt256::from_hex("1572d4c89600f6aa69a15d374ddc1bdbc37e260511a48f1713cb295a4d8fd5f7").unwrap(),
        UInt256::from_hex("7096d138cfaad0855a052c603f5666cd1bae259552c2e97237fe3def4dfd05b0").unwrap(),
        UInt256::from_hex("f4ed2b9b9da593cc2a78a4f07f3efdcbb49de1b803cab8b9988fe8b8ea6a9aae").unwrap(),
        UInt256::from_hex("ddd6658330591c6a91761b98b37471c8f7d2fd9363ce74aeced123791e9fcd66").unwrap(),
        UInt256::from_hex("f67833dbce400c96d633cf0e8b8c60ca7ea70fbcd0fd748d1408fd2e7703f84e").unwrap(),
        UInt256::from_hex("86b4f9f63d48c145c282012a149d1d716d845a9c1781436d4585b6366eb1133b").unwrap(),
        UInt256::from_hex("68090accc7d3b1fae409ad096d6c8813a3ff5900ac32b57b6d33a4ff5f8c2478").unwrap(),
        UInt256::from_hex("aa1d1cc68ee8fbe8e3d51330a28eda8cf5d9749e31ac1a5518d4cbbe627ef241").unwrap(),
        UInt256::from_hex("6d27a16e6a137af602e47bbd64992c46623e00ffdfc1e66018e626716511a152").unwrap(),
        UInt256::from_hex("0ac44988afcdb876cee272a59fe30546d800fd3751e780a3c1f9bd2fe0370835").unwrap(),
        UInt256::from_hex("bbab186b508afa0a449bc7ddca82fc80e429d324dba3cdbe1f7198d57c0f0ad7").unwrap(),
        UInt256::from_hex("e998a63b396efe599ffbccea9a50e2eda2ae83f6016a308d382accec23428aef").unwrap(),
        UInt256::from_hex("f3536f95b51cc781dbc55bc65a57a08bb77cc4edb425ddf9ec313aabf352c7e1").unwrap(),
        UInt256::from_hex("d37930382576d168ab9f0d1662c9ba048c114d6100487f25c25cf266249c644f").unwrap(),
        UInt256::from_hex("215aaced5c3527a23ab919a5974f0ae355450a3e99d38762bb73e2e6070bb239").unwrap(),
        UInt256::from_hex("71de51a9dbfa174c8cab524c75b002f86715fee6433839511dfb4786e12d3953").unwrap(),
        UInt256::from_hex("f5791cecae11ce00c8148710118168e69f6e574ed2e2ab8325558e7366fd15b7").unwrap(),
        UInt256::from_hex("e779e9d71fc2d14ba418c1588809f8e672a2f0e85f104a1a0793fbe04bdc9305").unwrap(),
        UInt256::from_hex("b99bdd3fcfb73de03aef90b0b7df8679002e8fc487af14fac46d767c6d542b48").unwrap(),
    ]
}


#[test]
fn testnet_test_retrieve_saved_hashes() {
    let chain = ChainType::TestNet;
    let context = &mut (FFIContext { chain, is_dip_0024: false, cache: &mut Default::default(), blocks: init_testnet_store() });
    let bytes_122064 = message_from_file("MNL_0_122064.dat");
    let processor = unsafe {
        &mut *register_processor(
            get_merkle_root_by_hash_default,
            get_block_height_by_hash_from_context,
            get_block_hash_by_height_from_context,
            get_llmq_snapshot_by_block_hash_from_context,
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
    let result_122064 = unsafe {
        *process_mnlistdiff_from_message(
            bytes_122064.as_ptr(),
            bytes_122064.len(),
            chain,
            false,
            true,
            70221,
            processor,
            context.cache,
            context as *mut _ as *mut std::ffi::c_void,
    )};
    assert!(result_122064.is_valid(), "Result must be valid");
    let bytes_122088 = message_from_file("MNL_122064_122088.dat");
    let result_122088 = unsafe {
        *process_mnlistdiff_from_message(
            bytes_122088.as_ptr(),
            bytes_122088.len(),
            chain,
            false,
            true,
            70221,
            processor,
            context.cache,
            context as *mut _ as *mut std::ffi::c_void,
        )};
    assert!(result_122088.is_valid(), "Result must be valid");

    let block_hash_122064 = unsafe { UInt256(*result_122064.block_hash) };
    let block_hash_122088 = unsafe { UInt256(*result_122088.block_hash) };
    let list_122064 = unsafe { (*result_122064.masternode_list).decode() };
    let list_122088 = unsafe { (*result_122088.masternode_list).decode() };

    let reloaded_list_122064 = context.cache.mn_lists.get(&block_hash_122064).unwrap();
    let reloaded_list_122088 = context.cache.mn_lists.get(&block_hash_122088).unwrap();

    let entry_hash = UInt256::from_hex("1bde434d4f68064d3108a09443ea45b4a6c6ac1f537a533efc36878cef2eb10f").unwrap().reverse();
    let entry_122064 = list_122064.masternodes.get(&entry_hash).unwrap();
    let entry_122088 = list_122088.masternodes.get(&entry_hash).unwrap();
    assert_ne!(entry_122064, entry_122088, "These should NOT be the same object (unless we changed how this worked)");

    assert_eq!(entry_122088.clone().previous_entry_hashes.into_values().collect::<Vec<UInt256>>(),
               vec![UInt256::from_hex("14d8f2de996a2515815abeb8f111a3ffe8582443ce7a43a8399c1a1c86c65543").unwrap()],
               "This is what it used to be");

    assert_eq!(entry_122064.entry_hash, UInt256::from_hex("14d8f2de996a2515815abeb8f111a3ffe8582443ce7a43a8399c1a1c86c65543").unwrap(), "The hash of the sme should be this");
    assert_eq!(entry_122088.entry_hash, UInt256::from_hex("e001033590361b172da9cb352f9736dbe9453c6a389068f7b76d71f9f3044d3b").unwrap(), "The hash changed to this");

    let local_hashes_122088 = list_122088.sorted_reversed_pro_reg_tx_hashes();
    let hashes_122088 = reloaded_list_122088.sorted_reversed_pro_reg_tx_hashes();
    assert_eq!(local_hashes_122088, hashes_122088, "Hashes for 122088 must be equal");

    let local_hashes_122064 = list_122064.sorted_reversed_pro_reg_tx_hashes();
    let hashes_122064 = reloaded_list_122064.sorted_reversed_pro_reg_tx_hashes();
    assert_eq!(local_hashes_122064, hashes_122064, "Hashes for 122064 must be equal");

    let mut entry_hashes = local_hashes_122064.iter().map(|hash| reloaded_list_122064.masternodes.get(hash).unwrap().entry_hash).collect::<Vec<UInt256>>();
    entry_hashes.sort_by(|&s1, &s2| s2.reversed().cmp(&s1.reversed()));
    let mut reloaded_entry_hashes = hashes_122064.iter().map(|hash| reloaded_list_122064.masternodes.get(hash).unwrap().entry_hash).collect::<Vec<UInt256>>();
    reloaded_entry_hashes.sort_by(|&s1, &s2| s2.reversed().cmp(&s1.reversed()));

    assert_eq!(reloaded_entry_hashes, entry_hashes, "Entry hashes must be equal");
    assert_eq!(reloaded_list_122064.reversed_pro_reg_tx_hashes(), list_122064.reversed_pro_reg_tx_hashes(), "pro reg tx hashes must be equal");

    let reloaded_hashes_from_122064 = reloaded_list_122064.hashes_for_merkle_root(reloaded_list_122064.known_height).unwrap();
    let hashes_from_122064 = list_122064.hashes_for_merkle_root(reloaded_list_122088.known_height).unwrap();
    println!("------ INITIALIZED ------");
    hashes_from_122064.iter().for_each(|h| {
        println!("{}", h);
    });
    println!("------ RELOADED ------");
    reloaded_hashes_from_122064.iter().for_each(|h| {
        println!("{}", h);
    });
    println!("------ ------- ------");


    let reloaded_merkle_root_122088 = merkle_root_from_hashes(reloaded_list_122088.hashes_for_merkle_root(reloaded_list_122088.known_height).unwrap());
    let merkle_root_122088 = merkle_root_from_hashes(list_122088.hashes_for_merkle_root(list_122088.known_height).unwrap());

    let reloaded_merkle_root_122064 = merkle_root_from_hashes(reloaded_list_122064.hashes_for_merkle_root(reloaded_list_122064.known_height).unwrap());
    let merkle_root_122064 = merkle_root_from_hashes(list_122064.hashes_for_merkle_root(list_122064.known_height).unwrap());

    let merkle_root_x = merkle_root_from_hashes(list_122064.hashes_for_merkle_root(list_122088.known_height).unwrap());

    assert_eq!(reloaded_merkle_root_122088, merkle_root_122088, "Merkle root for 122088 must be equal");
    assert_eq!(reloaded_merkle_root_122064, merkle_root_122064, "Merkle root for 122064 must be equal");
    assert_eq!(merkle_root_x.unwrap(), UInt256::from_hex("86cfe9b759dfd012f8d00e980c560c5c1d9c487bfa8b59305e14c7fc60ef1150").unwrap(), "")

}
