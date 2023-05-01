use std::collections::BTreeMap;
use hashes::hex::ToHex;
use secp256k1::rand::{Rng, thread_rng};
use crate::chain::common::LLMQType;
use crate::crypto::byte_util::{Reversable, Zeroable};
use crate::crypto::UInt256;
use crate::models::{LLMQEntry, LLMQSnapshot, MasternodeEntry};

// Utils for generation java code for dashj

pub fn generate_test<F: Fn()>(contents: F) {
    println!("@Test");
    println!("public void test_{}() {{", thread_rng().gen_range(0..8184));
    contents();
    println!("}}");
}

pub fn generate_final_commitment(quorum: &LLMQEntry) {
    println!("FinalCommitment finalCommitment = new FinalCommitment(params, (short) {}, {}, \
            Sha256Hash.wrap(\"{}\"), \
            {}, \
            {}, Utils.HEX.decode(\"{}\"), \
            {}, Utils.HEX.decode(\"{}\"), \
            Utils.HEX.decode(\"{}\"), \
            Sha256Hash.wrap(\"{}\"), \
            new BLSLazySignature(params, Utils.HEX.decode(\"{}\"), 0, {}), \
            new BLSLazySignature(params, Utils.HEX.decode(\"{}\"), 0, {})\
            );",
             u16::from(quorum.version),
             u8::from(quorum.llmq_type),
             quorum.llmq_hash.reversed(),
             quorum.index.unwrap_or(0),
             quorum.signers_count.0,
             quorum.signers_bitset.to_hex(),
             quorum.valid_members_count.0,
             quorum.valid_members_bitset.to_hex(),
             quorum.public_key,
             quorum.verification_vector_hash.reversed(),
             quorum.threshold_signature,
             quorum.version.use_bls_legacy(),
             quorum.all_commitment_aggregated_signature,
             quorum.version.use_bls_legacy()
    );

}
pub fn generate_masternode_list_from_map(map: &BTreeMap<UInt256, MasternodeEntry>) {
    let vec = map.iter().map(|(k, v)| v).cloned().collect();
    generate_masternode_list(&vec);
}

pub fn generate_masternode_list(masternodes: &Vec<MasternodeEntry>) {
    println!("ArrayList<SimplifiedMasternodeListEntry> nodes = new ArrayList<>();");
    masternodes.iter().for_each(|mn| {
        println!("nodes.add(new SimplifiedMasternodeListEntry(params, \
            (short) {}, \
            {}, \
            Sha256Hash.wrap(\"{}\"), \
            Sha256Hash.wrap(\"{}\"), \
            new MasternodeAddress(InetAddress.getByAddress(Utils.HEX.decode(\"{}\")), {}), \
            KeyId.fromBytes(Hex.decode(\"{}\")), \
            new BLSLazyPublicKey(params, Hex.decode(\"{}\"), 0, {}), \
            {}, \
            {}, \
            {}));",
                 mn.operator_public_key.version,
                 u16::from(mn.mn_type),
                 mn.provider_registration_transaction_hash.reversed(),
                 mn.confirmed_hash.reversed(),
                 mn.socket_address.ip_address,
                 mn.socket_address.port,
                 mn.key_id_voting,
                 mn.operator_public_key.data,
                 mn.operator_public_key.version < 2,
                 if mn.platform_node_id.is_zero() { format!("null") } else { format!("KeyId.fromBytes(Hex.decode(\"{}\"))", mn.platform_node_id) },
                 mn.platform_http_port,
                 mn.is_valid
        );
    });
    println!("SimplifiedMasternodeList allMns = new SimplifiedMasternodeList(params, nodes);");
}

pub fn generate_snapshot_from_bytes(bytes: &[u8]) {
    println!("QuorumSnapshot snapshot = new QuorumSnapshot(params, Utils.HEX.decode(\"{}\"), 0);", bytes.to_hex());
}

pub fn generate_snapshot(snapshot: &LLMQSnapshot, height: u32) {
    println!("List<Integer> skipList = new ArrayList<>();");
    // snapshot.skip_list.iter().for_each(|i| {
    //     println!("skipList.add({});", i);
    // });
    println!("skiplist: {:?}: {:?}", snapshot.skip_list_mode, snapshot.skip_list);
    println!("QuorumSnapshot snapshot_{} = new QuorumSnapshot(Utils.HEX.decode(\"{}\"), {}, skipList);", height, snapshot.member_list.to_hex(), u32::from(snapshot.skip_list_mode));
}

pub fn generate_llmq_hash(llmq_type:LLMQType, hash: UInt256) {
    println!("Sha256Hash modifier = LLMQUtils.buildLLMQBlockHash(LLMQParameters.LLMQType.fromValue({}), Sha256Hash.wrap(\"{}\"));", u8::from(llmq_type), hash);
}
