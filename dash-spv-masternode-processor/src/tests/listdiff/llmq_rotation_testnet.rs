use crate::bindings::common::{processor_create_cache, register_processor};
use crate::chain::common::ChainType;
use crate::crypto::{byte_util::Reversable, UInt256};
use crate::hashes::hex::FromHex;
use crate::lib_tests::tests::{add_insight_lookup_default, FFIContext, get_block_hash_by_height_from_context, get_block_height_by_hash_from_context, get_llmq_snapshot_by_block_hash_from_context, get_masternode_list_by_block_hash_from_cache, get_merkle_root_by_hash_default, hash_destroy_default, masternode_list_destroy_default, masternode_list_save_in_cache, message_from_file, process_qrinfo_from_message_internal, save_llmq_snapshot_in_cache, should_process_diff_with_range_default, snapshot_destroy_default};
use crate::models;
use crate::tests::block_store::init_mainnet_store;
use crate::tests::json_from_core_snapshot::{block_hash_to_block_hash, ListDiff, masternode_list_from_genesis_diff, QRInfo, snapshot_to_snapshot};

#[test]
fn testnet_quorum_quarters() {
    let block_height_8792 = 1738792;
    let block_height_8840 = 1738840;
    let block_height_8888 = 1738888;

    let qrinfo_8792: QRInfo = serde_json::from_slice(&message_from_file("1738792.txt")).unwrap();
    let qrinfo_8840: QRInfo = serde_json::from_slice(&message_from_file("1738840.txt")).unwrap();
    let qrinfo_8888: QRInfo = serde_json::from_slice(&message_from_file("1738888.txt")).unwrap();
    let qrinfo_8936: QRInfo = serde_json::from_slice(&message_from_file("1738936.txt")).unwrap();

    let diff_8792: ListDiff = serde_json::from_slice(&message_from_file("1738792_diff.txt")).unwrap();
    let diff_8840: ListDiff = serde_json::from_slice(&message_from_file("1738840_diff.txt")).unwrap();
    let diff_8888: ListDiff = serde_json::from_slice(&message_from_file("1738888_diff.txt")).unwrap();

    let list_diff_8792 = masternode_list_from_genesis_diff(diff_8792, |_| block_height_8792, false);
    let list_diff_8840 = masternode_list_from_genesis_diff(diff_8840, |_| block_height_8840, false);
    let list_diff_8888 = masternode_list_from_genesis_diff(diff_8888, |_| block_height_8888, false);

    let block_hash_8792 = list_diff_8792.block_hash;
    let block_hash_8840 = list_diff_8840.block_hash;
    let block_hash_8888 = list_diff_8888.block_hash;
    let masternode_list_8792 = models::MasternodeList::new(list_diff_8792.added_or_modified_masternodes, list_diff_8792.added_quorums, block_hash_8792, block_height_8792, true);
    let masternode_list_8840 = models::MasternodeList::new(list_diff_8840.added_or_modified_masternodes, list_diff_8840.added_quorums, block_hash_8840, block_height_8840, true);
    let masternode_list_8888 = models::MasternodeList::new(list_diff_8888.added_or_modified_masternodes, list_diff_8888.added_quorums, block_hash_8888, block_height_8888, true);

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
    let chain = ChainType::MainNet;
    let cache = unsafe { &mut *processor_create_cache() };

    let context = &mut (FFIContext { chain, cache, is_dip_0024: true, blocks: init_mainnet_store() });

    let bytes = message_from_file("QRINFO_0_1739226.dat");
    let old_bytes = message_from_file("QRINFO_0_1740902.dat");
    let old_bytes2 = message_from_file("QRINFO_0_1740910.dat");
    processor.opaque_context = context as *mut _ as *mut std::ffi::c_void;
    processor.use_insight_as_backup = true;
    processor.chain_type = chain;

    processor.save_masternode_list(block_hash_8792, &masternode_list_8792);
    processor.save_masternode_list(block_hash_8840, &masternode_list_8840);
    processor.save_masternode_list(block_hash_8888, &masternode_list_8888);


    let snapshot_8792_h_c = snapshot_to_snapshot(qrinfo_8792.quorum_snapshot_at_hminus_c);
    let snapshot_8792_h_2c = snapshot_to_snapshot(qrinfo_8792.quorum_snapshot_at_hminus2c);
    let snapshot_8792_h_3c = snapshot_to_snapshot(qrinfo_8792.quorum_snapshot_at_hminus3c);
    let snapshot_8840_h_c = snapshot_to_snapshot(qrinfo_8840.quorum_snapshot_at_hminus_c);
    let snapshot_8840_h_2c = snapshot_to_snapshot(qrinfo_8840.quorum_snapshot_at_hminus2c);
    let snapshot_8840_h_3c = snapshot_to_snapshot(qrinfo_8840.quorum_snapshot_at_hminus3c);
    let snapshot_8888_h_c = snapshot_to_snapshot(qrinfo_8888.quorum_snapshot_at_hminus_c);
    let snapshot_8888_h_2c = snapshot_to_snapshot(qrinfo_8888.quorum_snapshot_at_hminus2c);
    let snapshot_8888_h_3c = snapshot_to_snapshot(qrinfo_8888.quorum_snapshot_at_hminus3c);
    let snapshot_8936_h_c = snapshot_to_snapshot(qrinfo_8936.quorum_snapshot_at_hminus_c);
    let snapshot_8936_h_2c = snapshot_to_snapshot(qrinfo_8936.quorum_snapshot_at_hminus2c);
    let snapshot_8936_h_3c = snapshot_to_snapshot(qrinfo_8936.quorum_snapshot_at_hminus3c);

    let block_hash_8792_h_c = block_hash_to_block_hash(qrinfo_8792.mn_list_diff_at_hminus_c.block_hash).reversed();
    let block_hash_8792_h_2c = block_hash_to_block_hash(qrinfo_8792.mn_list_diff_at_hminus2c.block_hash).reversed();
    let block_hash_8792_h_3c = block_hash_to_block_hash(qrinfo_8792.mn_list_diff_at_hminus3c.block_hash).reversed();
    let block_hash_8840_h_c = block_hash_to_block_hash(qrinfo_8840.mn_list_diff_at_hminus_c.block_hash).reversed();
    let block_hash_8840_h_2c = block_hash_to_block_hash(qrinfo_8840.mn_list_diff_at_hminus2c.block_hash).reversed();
    let block_hash_8840_h_3c = block_hash_to_block_hash(qrinfo_8840.mn_list_diff_at_hminus3c.block_hash).reversed();
    let block_hash_8888_h_c = block_hash_to_block_hash(qrinfo_8888.mn_list_diff_at_hminus_c.block_hash).reversed();
    let block_hash_8888_h_2c = block_hash_to_block_hash(qrinfo_8888.mn_list_diff_at_hminus2c.block_hash).reversed();
    let block_hash_8888_h_3c = block_hash_to_block_hash(qrinfo_8888.mn_list_diff_at_hminus3c.block_hash).reversed();
    let block_hash_8936_h_c = block_hash_to_block_hash(qrinfo_8936.mn_list_diff_at_hminus_c.block_hash).reversed();
    let block_hash_8936_h_2c = block_hash_to_block_hash(qrinfo_8936.mn_list_diff_at_hminus2c.block_hash).reversed();
    let block_hash_8936_h_3c = block_hash_to_block_hash(qrinfo_8936.mn_list_diff_at_hminus3c.block_hash).reversed();


    processor.save_snapshot(block_hash_8792_h_c, snapshot_8792_h_c);
    processor.save_snapshot(block_hash_8792_h_2c, snapshot_8792_h_2c);
    processor.save_snapshot(block_hash_8792_h_3c, snapshot_8792_h_3c);
    processor.save_snapshot(block_hash_8840_h_c, snapshot_8840_h_c);
    processor.save_snapshot(block_hash_8840_h_2c, snapshot_8840_h_2c);
    processor.save_snapshot(block_hash_8840_h_3c, snapshot_8840_h_3c);
    processor.save_snapshot(block_hash_8888_h_c, snapshot_8888_h_c);
    processor.save_snapshot(block_hash_8888_h_2c, snapshot_8888_h_2c);
    processor.save_snapshot(block_hash_8888_h_3c, snapshot_8888_h_3c);
    processor.save_snapshot(block_hash_8936_h_c, snapshot_8936_h_c);
    processor.save_snapshot(block_hash_8936_h_2c, snapshot_8936_h_2c);
    processor.save_snapshot(block_hash_8936_h_3c, snapshot_8936_h_3c);

    let old_result = process_qrinfo_from_message_internal(old_bytes.as_ptr(), old_bytes.len(), chain, false, true, 70221, processor, context.cache, context as *mut _ as *mut std::ffi::c_void);
    let old_result2 = process_qrinfo_from_message_internal(old_bytes2.as_ptr(), old_bytes2.len(), chain, false, true, 70221, processor, context.cache, context as *mut _ as *mut std::ffi::c_void);
    // println!("old_result tip: {}", old_result.result_at_tip.block_hash.clone().reversed()); // 1740902
    // println!("old_result h: {}", old_result.result_at_h.block_hash.clone().reversed()); // 1738648
    // println!("old_result h-c: {}", old_result.result_at_h_c.block_hash.clone().reversed()); // 1738360
    // println!("old_result h-2c: {}", old_result.result_at_h_2c.block_hash.clone().reversed()); // 1738072
    // println!("old_result h-3c: {}", old_result.result_at_h_3c.block_hash.clone().reversed()); // 1737784
    // println!("old_result2 tip: {}", old_result2.result_at_tip.block_hash.clone().reversed()); // 1740902
    // println!("old_result2 h: {}", old_result2.result_at_h.block_hash.clone().reversed()); // 1738648
    // println!("old_result2 h-c: {}", old_result2.result_at_h_c.block_hash.clone().reversed()); // 1738360
    // println!("old_result2 h-2c: {}", old_result2.result_at_h_2c.block_hash.clone().reversed()); // 1738072
    // println!("old_result2 h-3c: {}", old_result2.result_at_h_3c.block_hash.clone().reversed()); // 1737784
    // println!("old_result h-4c: {}", old_result.result_at_h_4c.block_hash);

    let result = process_qrinfo_from_message_internal(
        bytes.as_ptr(),
        bytes.len(),
        chain,
        false,
        true,
        70221,
        processor,
        context.cache,
        context as *mut _ as *mut std::ffi::c_void
    );
    let block_height = 1738944;
    let mut last_quorum = result.last_quorum_per_index.first().cloned().unwrap();
    let nodes = processor.get_rotated_masternodes_for_quorum(
        last_quorum.llmq_type,
        last_quorum.llmq_hash,
        block_height,
        &mut context.cache.llmq_members,
        &mut context.cache.llmq_indexed_members,
        &context.cache.mn_lists,
        &context.cache.llmq_snapshots,
        &mut context.cache.needed_masternode_lists,
        true
    );
    let node_hashes = nodes.clone().into_iter().map(|m| m.provider_registration_transaction_hash).collect::<Vec<UInt256>>();

    println!("{}: {:?}", context.block_for_hash(last_quorum.llmq_hash).unwrap().height, last_quorum);


    let needed_hashes = [
            UInt256::from_hex("d2426992844e311149b2962f4ea991656ea5d1b36541e314e251915a6ffa1316").unwrap().reverse(),
            UInt256::from_hex("82dcf337825f4eb6726ffe56ab981034c49785c1f92da274e51fe925380265a6").unwrap().reverse(),
            UInt256::from_hex("30c14c1d75694404dbc96d44d1c63f28bf685d27e44ef753883c5eb0742c5805").unwrap().reverse(),
            UInt256::from_hex("631a5e99e59bab2bf06e4ec9874b3c3ced59187dc724c689ffa0fe99f2d3f210").unwrap().reverse(),
            UInt256::from_hex("113673823cf38eb56f30795d7f830b2d82b70a35b96812ad79d0889465b69df3").unwrap().reverse(),
            UInt256::from_hex("0e01a796d779a8494be44311d6daf07fdae8642b45a9b866024cd97d8dfd3545").unwrap().reverse(),
            UInt256::from_hex("feb146a52985cbf69f0a46b3a56f5f505538440bd42e993a0685a11fac1fdda5").unwrap().reverse(),
            UInt256::from_hex("48d665ade991114900349635c788775204057220728212e2b6b63e3545f67007").unwrap().reverse(),
            UInt256::from_hex("adcc454391d9f53e6fa523e047db5f335e38d9ead70dc8e4e648118c285d1d77").unwrap().reverse(),
            UInt256::from_hex("aadd2eac24d6cc9a0b439d3c9d5483b0ed64ebda21786760bb375efacd6e2a67").unwrap().reverse(),
            UInt256::from_hex("a90df0c30b2002a85cef6b695b986a3815c7dc82027c6896b7747258675a76da").unwrap().reverse(),
            UInt256::from_hex("e13fbad0ab0bca68cbcd4d68127de06037b4007c5acbe8550edcd60b7d4503e8").unwrap().reverse(),
            UInt256::from_hex("200ccffb2ef1da266efda63b35bc80bc8a547f29af3b8108f81ab222a1ceac35").unwrap().reverse(),
            UInt256::from_hex("21173e00737bac467d09effb15959d0459b853160805057a3975436d3e206591").unwrap().reverse(),
            UInt256::from_hex("321a1300b3ec63b3082d1626c7377d3c9e4e332eadefe12073f5e17b5099a49e").unwrap().reverse(),
            UInt256::from_hex("c1666e28121c0e2375e785edcbd2e84e1136f9fa0e415a9310d203aaa06e2153").unwrap().reverse(),
            UInt256::from_hex("c75b5aaf2a9c127b3f4199d4700abc1eab9fb8ebdf31b4d296951ce395bd52e0").unwrap().reverse(),
            UInt256::from_hex("7878b048a1411dc9e7cc71b4d3aae6f4bdb93f299f186fee4cc7a4833a7114a7").unwrap().reverse(),
            UInt256::from_hex("2899b3e2bb291e84943ab9436f93e0708cbb59860f9c05b41ffac4678dd714a5").unwrap().reverse(),
            UInt256::from_hex("320cac56da899ad7b5133d7d0aa54d0f12d2a7fddea9c97c6749e2a7efdf0b98").unwrap().reverse(),
            UInt256::from_hex("93bd23b7357a62c5ab2cc306054bf0b1fa6f52265fbbef23f600298d8daebd1a").unwrap().reverse(),
            UInt256::from_hex("f9d8fbb02e8f909b7922cc34b8259ea6b73dab88738c2af2e02e1ac929ccdd5e").unwrap().reverse(),
            UInt256::from_hex("b86f9ae4178a3425abb8053a40e1eb138b2a68b466ba2b3bd04167060cd5c1b5").unwrap().reverse(),
            UInt256::from_hex("d2f9d05f8983ce1d645487efbc4c3b2c31b8ec914b1e9085e70202001e11bcd4").unwrap().reverse(),
            UInt256::from_hex("c98ab7576c4b83d94e668f0e3910aa514a6ff0171684f9fc8e30143664d20999").unwrap().reverse(),
            UInt256::from_hex("70f8c30d741529a6ffd4e080993523709b00ad45e6026a19097e13f68d687d55").unwrap().reverse(),
            UInt256::from_hex("4f45b42e8b170dac1cfe288815423020135e105f4c98e1b2f09db0cf059d1ac6").unwrap().reverse(),
            UInt256::from_hex("a8ac86f944f9477ba1642bc3f45dba51bb020af43afc1c0ca2f1f1910be209be").unwrap().reverse(),
            UInt256::from_hex("fdb28cd79093c2a6be18f7b168f50008125a2416984ac4126dab6c8646657dd6").unwrap().reverse(),
            UInt256::from_hex("8e483b9308737fe836d71967ddb634559a5c63c8cb8d495275952e0f6cb5a717").unwrap().reverse(),
            UInt256::from_hex("9389d805cb14e9cb6af5afa7c75136507a4a42e1e5bbffb6703dc57ed10de429").unwrap().reverse(),
            UInt256::from_hex("92a22ba783087016770ff1acae996253a8a3803566aee90d914c3adf9f611116").unwrap().reverse(),
            UInt256::from_hex("390e6b731013c3da296b61a7365b9954c5a403c95f997e89bf979469f9c1d704").unwrap().reverse(),
            UInt256::from_hex("867fc5a33fdc09b811cc4a7a307a3ca082a3adc3bc8511ed67e81cc6884ba68d").unwrap().reverse(),
            UInt256::from_hex("784a0529d150a44ee26ec8f08ab1eaab0d0651ed2ee1f8e611a4304626448e79").unwrap().reverse(),
            UInt256::from_hex("4016510e2ef1b82fc571781b5a0cd8afa268d0c21b5d352f8f8929c542d1aef3").unwrap().reverse(),
            UInt256::from_hex("776cbacfaeafd9880d791357c00c5085ddc8d00af507a35dad37c3d0f7a24256").unwrap().reverse(),
            UInt256::from_hex("5480f489c2e7d4e9992349a0eb85b4a81796979c66a5f2c67f5df05866e01df1").unwrap().reverse(),
            UInt256::from_hex("5ec696060fb6d43e5065e858a590b98af558c1e0d48d1a08b634189131b17ee7").unwrap().reverse(),
            UInt256::from_hex("17e5abab4d2bc5b03d14163d57bd0b3014bd7ec56a1051b8bded4373b0d08ba7").unwrap().reverse(),
            UInt256::from_hex("754285ad30ce2aa5f9d799770a2e06afff77613e09c240d3019d1f266e2f1167").unwrap().reverse(),
            UInt256::from_hex("68b9a31205e3c190346a162f29804feeee0fe17caa3bda09c1d97e4913967b56").unwrap().reverse(),
            UInt256::from_hex("b0219f34cc8fccaf88796889a8cd034eac82415cd7f7f3baae950c841857f85c").unwrap().reverse(),
            UInt256::from_hex("e8f49c2f481b18d8b3ffdfe36dd7cce85bbd0332cd723b7b329d2192cef1fb61").unwrap().reverse(),
            UInt256::from_hex("7fd0a504278a1e6a0cc1188c98178c279667450502a87e8e841dfaab7896657b").unwrap().reverse(),
            UInt256::from_hex("168162f349bd2961fab43cd0a19a7e6a34c7a18d5dbe4805c06a4fcbcb138e78").unwrap().reverse(),
            UInt256::from_hex("758d75feab2349545b83db14acb101419bd8874118a493587bce5ed344ad959d").unwrap().reverse(),
            UInt256::from_hex("db6fbedc75d6acd52b505d9f553716255c8e25b434f8df211a190bc17dd7e4ff").unwrap().reverse(),
            UInt256::from_hex("d74519d65b78ab5c34c8c8bf8951e967a35ebc17b2eaea3f5d1fbe719583d5fd").unwrap().reverse(),
            UInt256::from_hex("56d86d05eb95672667a7f7a5fa8908617e4f7624a1d2897e49469e2a591b65d3").unwrap().reverse(),
            UInt256::from_hex("d3ddea9366c7deb606d1ba58e2911a3958d81f887dc8c1af69e6f90a0b0ec8df").unwrap().reverse(),
            UInt256::from_hex("f0cf643f2098605eb570f22a72d6fb93735563e35c5b9bb93885236d5388e0f6").unwrap().reverse(),
            UInt256::from_hex("af6ab1210c32d3e0c8c1425402986c0563c926173e71a85410b20e04d9728579").unwrap().reverse(),
            UInt256::from_hex("5b9bb413ec3d6dd8c7675471c28be5a12196ca73eec70a84f96648987f28220e").unwrap().reverse(),
            UInt256::from_hex("4030316b3a1aaa4aa174e94e7c82fd847e116a50cc280bf37a44a94215ebd747").unwrap().reverse(),
            UInt256::from_hex("65c11e751596f3e2f8829b41bce171d872d3a0e9bd6ea8b65757f61b47b6d50d").unwrap().reverse(),
            UInt256::from_hex("35584dc1f4cc507012648a42b30c9aaaee19d028abd16ee31c87a04bf4a91470").unwrap().reverse(),
            UInt256::from_hex("d802702597a6c8a2d8e6011f5ff365595db3893d5531c65ecd5be63c8dfc13c9").unwrap().reverse(),
            UInt256::from_hex("d66ba16f114b921b3e48c7a0681ba465a68f09029a130999859cb0abb48a0717").unwrap().reverse(),
            UInt256::from_hex("fac4265c8c8213b069b75bb3c565309accb214631bff9cfb41a4ec1b6feda7fc").unwrap().reverse()
    ];
    println!("##############");
    println!("{:#?}", node_hashes);
    println!("##############");
    println!("{:#?}", needed_hashes);
    println!("##############");
    // assert_eq!(node_hashes, needed_hashes, "Quorum Combo must be equal");

    let new_quarter_members = [
        UInt256::from_hex("168162f349bd2961fab43cd0a19a7e6a34c7a18d5dbe4805c06a4fcbcb138e78").unwrap().reverse(),
        UInt256::from_hex("758d75feab2349545b83db14acb101419bd8874118a493587bce5ed344ad959d").unwrap().reverse(),
        UInt256::from_hex("db6fbedc75d6acd52b505d9f553716255c8e25b434f8df211a190bc17dd7e4ff").unwrap().reverse(),
        UInt256::from_hex("d74519d65b78ab5c34c8c8bf8951e967a35ebc17b2eaea3f5d1fbe719583d5fd").unwrap().reverse(),
        UInt256::from_hex("56d86d05eb95672667a7f7a5fa8908617e4f7624a1d2897e49469e2a591b65d3").unwrap().reverse(),
        UInt256::from_hex("d3ddea9366c7deb606d1ba58e2911a3958d81f887dc8c1af69e6f90a0b0ec8df").unwrap().reverse(),
        UInt256::from_hex("f0cf643f2098605eb570f22a72d6fb93735563e35c5b9bb93885236d5388e0f6").unwrap().reverse(),
        UInt256::from_hex("af6ab1210c32d3e0c8c1425402986c0563c926173e71a85410b20e04d9728579").unwrap().reverse(),
        UInt256::from_hex("5b9bb413ec3d6dd8c7675471c28be5a12196ca73eec70a84f96648987f28220e").unwrap().reverse(),
        UInt256::from_hex("4030316b3a1aaa4aa174e94e7c82fd847e116a50cc280bf37a44a94215ebd747").unwrap().reverse(),
        UInt256::from_hex("65c11e751596f3e2f8829b41bce171d872d3a0e9bd6ea8b65757f61b47b6d50d").unwrap().reverse(),
        UInt256::from_hex("35584dc1f4cc507012648a42b30c9aaaee19d028abd16ee31c87a04bf4a91470").unwrap().reverse(),
        UInt256::from_hex("d802702597a6c8a2d8e6011f5ff365595db3893d5531c65ecd5be63c8dfc13c9").unwrap().reverse(),
        UInt256::from_hex("d66ba16f114b921b3e48c7a0681ba465a68f09029a130999859cb0abb48a0717").unwrap().reverse(),
        UInt256::from_hex("fac4265c8c8213b069b75bb3c565309accb214631bff9cfb41a4ec1b6feda7fc").unwrap().reverse()
    ];
    //assert_eq!(node_hashes, new_quarter_members, "Quorum Combo must be equal");
    let quarters_h_c = [
        UInt256::from_hex("9389d805cb14e9cb6af5afa7c75136507a4a42e1e5bbffb6703dc57ed10de429").unwrap().reverse(),
        UInt256::from_hex("92a22ba783087016770ff1acae996253a8a3803566aee90d914c3adf9f611116").unwrap().reverse(),
        UInt256::from_hex("390e6b731013c3da296b61a7365b9954c5a403c95f997e89bf979469f9c1d704").unwrap().reverse(),
        UInt256::from_hex("867fc5a33fdc09b811cc4a7a307a3ca082a3adc3bc8511ed67e81cc6884ba68d").unwrap().reverse(),
        UInt256::from_hex("784a0529d150a44ee26ec8f08ab1eaab0d0651ed2ee1f8e611a4304626448e79").unwrap().reverse(),
        UInt256::from_hex("4016510e2ef1b82fc571781b5a0cd8afa268d0c21b5d352f8f8929c542d1aef3").unwrap().reverse(),
        UInt256::from_hex("776cbacfaeafd9880d791357c00c5085ddc8d00af507a35dad37c3d0f7a24256").unwrap().reverse(),
        UInt256::from_hex("5480f489c2e7d4e9992349a0eb85b4a81796979c66a5f2c67f5df05866e01df1").unwrap().reverse(),
        UInt256::from_hex("5ec696060fb6d43e5065e858a590b98af558c1e0d48d1a08b634189131b17ee7").unwrap().reverse(),
        UInt256::from_hex("17e5abab4d2bc5b03d14163d57bd0b3014bd7ec56a1051b8bded4373b0d08ba7").unwrap().reverse(),
        UInt256::from_hex("754285ad30ce2aa5f9d799770a2e06afff77613e09c240d3019d1f266e2f1167").unwrap().reverse(),
        UInt256::from_hex("68b9a31205e3c190346a162f29804feeee0fe17caa3bda09c1d97e4913967b56").unwrap().reverse(),
        UInt256::from_hex("b0219f34cc8fccaf88796889a8cd034eac82415cd7f7f3baae950c841857f85c").unwrap().reverse(),
        UInt256::from_hex("e8f49c2f481b18d8b3ffdfe36dd7cce85bbd0332cd723b7b329d2192cef1fb61").unwrap().reverse(),
        UInt256::from_hex("7fd0a504278a1e6a0cc1188c98178c279667450502a87e8e841dfaab7896657b").unwrap().reverse(),
    ];
    let quarter_h_2c = [
        UInt256::from_hex("c1666e28121c0e2375e785edcbd2e84e1136f9fa0e415a9310d203aaa06e2153").unwrap().reverse(),
        UInt256::from_hex("c75b5aaf2a9c127b3f4199d4700abc1eab9fb8ebdf31b4d296951ce395bd52e0").unwrap().reverse(),
        UInt256::from_hex("7878b048a1411dc9e7cc71b4d3aae6f4bdb93f299f186fee4cc7a4833a7114a7").unwrap().reverse(),
        UInt256::from_hex("2899b3e2bb291e84943ab9436f93e0708cbb59860f9c05b41ffac4678dd714a5").unwrap().reverse(),
        UInt256::from_hex("320cac56da899ad7b5133d7d0aa54d0f12d2a7fddea9c97c6749e2a7efdf0b98").unwrap().reverse(),
        UInt256::from_hex("93bd23b7357a62c5ab2cc306054bf0b1fa6f52265fbbef23f600298d8daebd1a").unwrap().reverse(),
        UInt256::from_hex("f9d8fbb02e8f909b7922cc34b8259ea6b73dab88738c2af2e02e1ac929ccdd5e").unwrap().reverse(),
        UInt256::from_hex("b86f9ae4178a3425abb8053a40e1eb138b2a68b466ba2b3bd04167060cd5c1b5").unwrap().reverse(),
        UInt256::from_hex("d2f9d05f8983ce1d645487efbc4c3b2c31b8ec914b1e9085e70202001e11bcd4").unwrap().reverse(),
        UInt256::from_hex("c98ab7576c4b83d94e668f0e3910aa514a6ff0171684f9fc8e30143664d20999").unwrap().reverse(),
        UInt256::from_hex("70f8c30d741529a6ffd4e080993523709b00ad45e6026a19097e13f68d687d55").unwrap().reverse(),
        UInt256::from_hex("4f45b42e8b170dac1cfe288815423020135e105f4c98e1b2f09db0cf059d1ac6").unwrap().reverse(),
        UInt256::from_hex("a8ac86f944f9477ba1642bc3f45dba51bb020af43afc1c0ca2f1f1910be209be").unwrap().reverse(),
        UInt256::from_hex("fdb28cd79093c2a6be18f7b168f50008125a2416984ac4126dab6c8646657dd6").unwrap().reverse(),
        UInt256::from_hex("8e483b9308737fe836d71967ddb634559a5c63c8cb8d495275952e0f6cb5a717").unwrap().reverse()
    ];
    let quarter_h_3c = [
        UInt256::from_hex("d2426992844e311149b2962f4ea991656ea5d1b36541e314e251915a6ffa1316").unwrap().reverse(),
        UInt256::from_hex("82dcf337825f4eb6726ffe56ab981034c49785c1f92da274e51fe925380265a6").unwrap().reverse(),
        UInt256::from_hex("30c14c1d75694404dbc96d44d1c63f28bf685d27e44ef753883c5eb0742c5805").unwrap().reverse(),
        UInt256::from_hex("631a5e99e59bab2bf06e4ec9874b3c3ced59187dc724c689ffa0fe99f2d3f210").unwrap().reverse(),
        UInt256::from_hex("113673823cf38eb56f30795d7f830b2d82b70a35b96812ad79d0889465b69df3").unwrap().reverse(),
        UInt256::from_hex("0e01a796d779a8494be44311d6daf07fdae8642b45a9b866024cd97d8dfd3545").unwrap().reverse(),
        UInt256::from_hex("feb146a52985cbf69f0a46b3a56f5f505538440bd42e993a0685a11fac1fdda5").unwrap().reverse(),
        UInt256::from_hex("48d665ade991114900349635c788775204057220728212e2b6b63e3545f67007").unwrap().reverse(),
        UInt256::from_hex("adcc454391d9f53e6fa523e047db5f335e38d9ead70dc8e4e648118c285d1d77").unwrap().reverse(),
        UInt256::from_hex("aadd2eac24d6cc9a0b439d3c9d5483b0ed64ebda21786760bb375efacd6e2a67").unwrap().reverse(),
        UInt256::from_hex("a90df0c30b2002a85cef6b695b986a3815c7dc82027c6896b7747258675a76da").unwrap().reverse(),
        UInt256::from_hex("e13fbad0ab0bca68cbcd4d68127de06037b4007c5acbe8550edcd60b7d4503e8").unwrap().reverse(),
        UInt256::from_hex("200ccffb2ef1da266efda63b35bc80bc8a547f29af3b8108f81ab222a1ceac35").unwrap().reverse(),
        UInt256::from_hex("21173e00737bac467d09effb15959d0459b853160805057a3975436d3e206591").unwrap().reverse(),
        UInt256::from_hex("321a1300b3ec63b3082d1626c7377d3c9e4e332eadefe12073f5e17b5099a49e").unwrap().reverse(),
    ];
    let payload_valid = last_quorum.validate_payload();
    println!("quorum payload valid: {}", payload_valid);
    assert!(payload_valid, "Invalid payload");
    let signature_valid = last_quorum.validate(nodes, block_height);
    println!("quorum signature valid: {}", signature_valid);
    assert!(payload_valid, "Invalid signature");


    // println!("node_hashes: {:#?}", node_hashes);
    // println!("new_quarter_members: {:#?}", new_quarter_members);
    // assert_eq!(node_hashes, new_quarter_members, "New quorum quarter members should be equal");

    //([0-9A-Fa-f]{64})
    //UInt256::from_hex("($1)").unwrap().reverse()
}

/*
20:10:45 26 QuorumRotationState.computeQuorumMembersByQuarterRotation: QuarterComposition h[1738944] i[0]:

H-3[
d2426992844e311149b2962f4ea991656ea5d1b36541e314e251915a6ffa1316 |
82dcf337825f4eb6726ffe56ab981034c49785c1f92da274e51fe925380265a6 |
30c14c1d75694404dbc96d44d1c63f28bf685d27e44ef753883c5eb0742c5805 |
631a5e99e59bab2bf06e4ec9874b3c3ced59187dc724c689ffa0fe99f2d3f210 |
113673823cf38eb56f30795d7f830b2d82b70a35b96812ad79d0889465b69df3 |
0e01a796d779a8494be44311d6daf07fdae8642b45a9b866024cd97d8dfd3545 |
feb146a52985cbf69f0a46b3a56f5f505538440bd42e993a0685a11fac1fdda5 |
48d665ade991114900349635c788775204057220728212e2b6b63e3545f67007 |
adcc454391d9f53e6fa523e047db5f335e38d9ead70dc8e4e648118c285d1d77 |
aadd2eac24d6cc9a0b439d3c9d5483b0ed64ebda21786760bb375efacd6e2a67 |
a90df0c30b2002a85cef6b695b986a3815c7dc82027c6896b7747258675a76da |
e13fbad0ab0bca68cbcd4d68127de06037b4007c5acbe8550edcd60b7d4503e8 |
200ccffb2ef1da266efda63b35bc80bc8a547f29af3b8108f81ab222a1ceac35 |
21173e00737bac467d09effb15959d0459b853160805057a3975436d3e206591 |
321a1300b3ec63b3082d1626c7377d3c9e4e332eadefe12073f5e17b5099a49e |
]
H-2[
c1666e28121c0e2375e785edcbd2e84e1136f9fa0e415a9310d203aaa06e2153 |
c75b5aaf2a9c127b3f4199d4700abc1eab9fb8ebdf31b4d296951ce395bd52e0 |
7878b048a1411dc9e7cc71b4d3aae6f4bdb93f299f186fee4cc7a4833a7114a7 |
2899b3e2bb291e84943ab9436f93e0708cbb59860f9c05b41ffac4678dd714a5 |
320cac56da899ad7b5133d7d0aa54d0f12d2a7fddea9c97c6749e2a7efdf0b98 |
93bd23b7357a62c5ab2cc306054bf0b1fa6f52265fbbef23f600298d8daebd1a |
f9d8fbb02e8f909b7922cc34b8259ea6b73dab88738c2af2e02e1ac929ccdd5e |
b86f9ae4178a3425abb8053a40e1eb138b2a68b466ba2b3bd04167060cd5c1b5 |
d2f9d05f8983ce1d645487efbc4c3b2c31b8ec914b1e9085e70202001e11bcd4 |
c98ab7576c4b83d94e668f0e3910aa514a6ff0171684f9fc8e30143664d20999 |
70f8c30d741529a6ffd4e080993523709b00ad45e6026a19097e13f68d687d55 |
4f45b42e8b170dac1cfe288815423020135e105f4c98e1b2f09db0cf059d1ac6 |
a8ac86f944f9477ba1642bc3f45dba51bb020af43afc1c0ca2f1f1910be209be |
fdb28cd79093c2a6be18f7b168f50008125a2416984ac4126dab6c8646657dd6 |
8e483b9308737fe836d71967ddb634559a5c63c8cb8d495275952e0f6cb5a717 |  ]

H-1
[9389d805cb14e9cb6af5afa7c75136507a4a42e1e5bbffb6703dc57ed10de429 |
92a22ba783087016770ff1acae996253a8a3803566aee90d914c3adf9f611116 |
390e6b731013c3da296b61a7365b9954c5a403c95f997e89bf979469f9c1d704 |
867fc5a33fdc09b811cc4a7a307a3ca082a3adc3bc8511ed67e81cc6884ba68d |
784a0529d150a44ee26ec8f08ab1eaab0d0651ed2ee1f8e611a4304626448e79 |
4016510e2ef1b82fc571781b5a0cd8afa268d0c21b5d352f8f8929c542d1aef3 |
776cbacfaeafd9880d791357c00c5085ddc8d00af507a35dad37c3d0f7a24256 |
5480f489c2e7d4e9992349a0eb85b4a81796979c66a5f2c67f5df05866e01df1 |
5ec696060fb6d43e5065e858a590b98af558c1e0d48d1a08b634189131b17ee7 |
17e5abab4d2bc5b03d14163d57bd0b3014bd7ec56a1051b8bded4373b0d08ba7 |
754285ad30ce2aa5f9d799770a2e06afff77613e09c240d3019d1f266e2f1167 |
68b9a31205e3c190346a162f29804feeee0fe17caa3bda09c1d97e4913967b56 |
b0219f34cc8fccaf88796889a8cd034eac82415cd7f7f3baae950c841857f85c |
e8f49c2f481b18d8b3ffdfe36dd7cce85bbd0332cd723b7b329d2192cef1fb61 |
7fd0a504278a1e6a0cc1188c98178c279667450502a87e8e841dfaab7896657b |
]

new Quarter members
[168162f349bd2961fab43cd0a19a7e6a34c7a18d5dbe4805c06a4fcbcb138e78 |
758d75feab2349545b83db14acb101419bd8874118a493587bce5ed344ad959d |
db6fbedc75d6acd52b505d9f553716255c8e25b434f8df211a190bc17dd7e4ff |
d74519d65b78ab5c34c8c8bf8951e967a35ebc17b2eaea3f5d1fbe719583d5fd |
56d86d05eb95672667a7f7a5fa8908617e4f7624a1d2897e49469e2a591b65d3 |
d3ddea9366c7deb606d1ba58e2911a3958d81f887dc8c1af69e6f90a0b0ec8df |
f0cf643f2098605eb570f22a72d6fb93735563e35c5b9bb93885236d5388e0f6 |
af6ab1210c32d3e0c8c1425402986c0563c926173e71a85410b20e04d9728579 |
5b9bb413ec3d6dd8c7675471c28be5a12196ca73eec70a84f96648987f28220e |
4030316b3a1aaa4aa174e94e7c82fd847e116a50cc280bf37a44a94215ebd747 |
65c11e751596f3e2f8829b41bce171d872d3a0e9bd6ea8b65757f61b47b6d50d |
35584dc1f4cc507012648a42b30c9aaaee19d028abd16ee31c87a04bf4a91470 |
d802702597a6c8a2d8e6011f5ff365595db3893d5531c65ecd5be63c8dfc13c9 |
d66ba16f114b921b3e48c7a0681ba465a68f09029a130999859cb0abb48a0717 |
fac4265c8c8213b069b75bb3c565309accb214631bff9cfb41a4ec1b6feda7fc |
]

 */
