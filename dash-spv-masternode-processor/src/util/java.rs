use std::collections::BTreeMap;
use std::io;
use std::io::Write;
use hashes::hex::ToHex;
use secp256k1::rand::{Rng, thread_rng};
use crate::chain::common::chain_type::DevnetType;
use crate::common::{ChainType, LLMQType};
use crate::crypto::{byte_util::{Reversable, Zeroable}, UInt256};
use crate::ffi::from::FromFFI;
use crate::models::{LLMQEntry, LLMQSnapshot, MasternodeEntry};
use crate::{models, types};
use crate::util::file::save_java_class;
use crate::util::save_json_file;

// Utils for generation java code for dashj

fn get_params_name_for_chain_type(chain_type: ChainType) -> &'static str {
    let params_name = match chain_type {
        ChainType::MainNet => "MainNetParams",
        ChainType::TestNet => "TestNet3Params",
        ChainType::DevNet(devnet) => match devnet {
            // DevnetType::TwoIslands => "TwoIslandsDevNetParams",
            DevnetType::WhiteRussian => "WhiteRussianDevNetParams",
            _ => "DevNetParams"
        }
    };
    params_name
}

fn get_chain_name_for_chain_type(chain_type: ChainType) -> &'static str {
    let chain_name = match chain_type {
        ChainType::MainNet => "mainnet",
        ChainType::TestNet => "testnet",
        _ => "devnet"
    };
    chain_name
}

fn get_llmq_name_for_llmq_type(llmq_type: LLMQType) -> &'static str {
    match llmq_type {
        LLMQType::LlmqtypeUnknown => "LLMQ_NONE",
        LLMQType::Llmqtype50_60 => "LLMQ_50_60",
        LLMQType::Llmqtype400_60 => "LLMQ_400_60",
        LLMQType::Llmqtype400_85 => "LLMQ_400_85",
        LLMQType::Llmqtype100_67 => "LLMQ_100_67",
        LLMQType::Llmqtype60_75 => "LLMQ_60_75",
        LLMQType::Llmqtype25_67 => "LLMQ_25_67",
        LLMQType::LlmqtypeTest => "LLMQ_TEST",
        LLMQType::LlmqtypeDevnet => "LLMQ_DEVNET",
        LLMQType::LlmqtypeTestV17 => "LLMQ_TEST_V17",
        LLMQType::LlmqtypeTestDIP0024 => "LLMQ_TEST_DIP0024",
        LLMQType::LlmqtypeTestInstantSend => "LLMQ_TEST_INSTANTSEND",
        LLMQType::LlmqtypeDevnetDIP0024 => "LLMQ_DEVNET_DIP0024",
        LLMQType::LlmqtypeTestnetPlatform => "LLMQ_TEST_PLATFORM",
        LLMQType::LlmqtypeDevnetPlatform => "LLMQ_DEVNET_PLATFORM",
    }
}

pub fn generate_test<F: Fn(&mut W), W: Write>(contents: F, writer: &mut W) -> io::Result<()> {
    writer.write_all("@Test\n".as_bytes())?;
    writer.write_all(format!("public void test_{}() throws UnknownHostException {{\n", thread_rng().gen_range(0..8184)).as_bytes())?;
    contents(writer);
    writer.write_all(format!("}}\n").as_bytes())
}

pub fn generate_qr_test<F: Fn(&mut W), W: Write>(contents: F, writer: &mut W) -> io::Result<()> {
    writer.write_all("@Test\n".as_bytes())?;
    writer.write_all(format!("public void testRotationQuorums_{}() throws UnknownHostException, BlockStoreException {{\n", thread_rng().gen_range(0..8184)).as_bytes())?;
    contents(writer);
    writer.write_all(format!("}}\n").as_bytes())
}

pub fn setup_context<W: Write>(chain_type: ChainType, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("Context context = new Context({}.get());\n", match chain_type {
        ChainType::MainNet => "MainNetParams",
        ChainType::TestNet => "TestNet3Params",
        ChainType::DevNet(devnet) => "DevNetParams",
    }).as_bytes())?;
    writer.write_all("context.initDash(true, true);\n".as_bytes())?;
    writer.write_all("NetworkParameters params = context.getParams();\n".as_bytes())?;
    writer.write_all(format!("params.setBasicBLSSchemeActivationHeight({});\n", chain_type.core19_activation_height()).as_bytes())
}

pub fn generate_final_commitment<W: Write>(quorum: &LLMQEntry, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("FinalCommitment finalCommitment = new FinalCommitment(params, (short) {}, {}, \
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
             quorum.version.use_bls_legacy()).as_bytes()
    )
}

pub fn generate_masternode_list_from_map<W: Write>(map: &BTreeMap<UInt256, MasternodeEntry>, block_height: u32, writer: &mut W) -> io::Result<()> {
    let vec = map.iter().map(|(k, v)| v).cloned().collect();
    generate_masternode_list(&vec, block_height, writer)
}

pub fn generate_mn_array_list_json(masternodes: &Vec<MasternodeEntry>, block_height: u32, nodes_var: &str) {
    //let value = serde_json::to_value(masternodes).unwrap();
    // serde_json::Value::Array(masternodes.clone())

}

pub fn generate_mn_array_list<W: Write>(masternodes: &Vec<MasternodeEntry>, block_height: u32, nodes_var: &str, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("ArrayList<SimplifiedMasternodeListEntry> {nodes_var} = new ArrayList<>();\n").as_bytes())?;
    Ok(masternodes.iter().for_each(|mn| {
        writer.write_all(format!("{nodes_var}.add(new SimplifiedMasternodeListEntry(params, \
            (short) {}, \
            {}, \
            Sha256Hash.wrap(\"{}\"), \
            Sha256Hash.wrap(\"{}\"), \
            new MasternodeAddress(InetAddress.getByAddress(Utils.HEX.decode(\"{}\")), {}), \
            KeyId.fromBytes(Hex.decode(\"{}\")), \
            new BLSLazyPublicKey(params, Hex.decode(\"{}\"), 0, {}), \
            {}, \
            {}, \
            {}));\n",
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
        ).as_bytes()).expect("ff");
    }))
}

pub fn generate_masternode_list<W: Write>(masternodes: &Vec<MasternodeEntry>, block_height: u32, writer: &mut W) -> io::Result<()> {
    let nodes_var = format!("nodes_{block_height}");
    writer.write_all(format!("SimplifiedMasternodeList list_at_{block_height}(NetworkParameters params) throws UnknownHostException {{\n").as_bytes())?;
    generate_mn_array_list(masternodes, block_height, nodes_var.as_str(), writer)?;
    writer.write_all(format!("return new SimplifiedMasternodeList(params, {nodes_var});\n").as_bytes())?;
    writer.write_all(format!("}}\n").as_bytes())
}

pub fn generate_valid_masternodes<W: Write>(masternodes: &Vec<MasternodeEntry>, block_height: u32, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("ArrayList<Masternode> valid_nodes_at_{block_height}(NetworkParameters params) throws UnknownHostException {{\n").as_bytes())?;
    writer.write_all("ArrayList<Masternode> nodes = new ArrayList<>();\n".as_bytes())?;
    masternodes.iter().for_each(|mn| {
        writer.write_all(format!("nodes.add(new SimplifiedMasternodeListEntry(params, \
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
        ).as_bytes()).expect("");
    });
    writer.write_all("return nodes;\n".as_bytes())?;
    writer.write_all(format!("}}\n").as_bytes())
}

pub fn generate_snapshot_from_bytes<W: Write>(bytes: &[u8], writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("QuorumSnapshot snapshot = new QuorumSnapshot(params, Utils.HEX.decode(\"{}\"), 0);", bytes.to_hex()).as_bytes())
}

pub fn generate_snapshot<W: Write>(snapshot: &LLMQSnapshot, height: u32, writer: &mut W) -> io::Result<()> {
    writer.write_all("List<Integer> skipList = new ArrayList<>();\n".as_bytes())?;
    // snapshot.skip_list.iter().for_each(|i| {
    //     println!("skipList.add({});", i);
    // });
    writer.write_all(format!("skiplist: {:?}: {:?}\n", snapshot.skip_list_mode, snapshot.skip_list).as_bytes())?;
    writer.write_all(format!("QuorumSnapshot snapshot_{} = new QuorumSnapshot(Utils.HEX.decode(\"{}\"), {}, skipList);\n", height, snapshot.member_list.to_hex(), u32::from(snapshot.skip_list_mode)).as_bytes())
}

pub fn generate_llmq_hash<W: Write>(llmq_type:LLMQType, hash: UInt256, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("Sha256Hash modifier = LLMQUtils.buildLLMQBlockHash(LLMQParameters.LLMQType.fromValue({}), Sha256Hash.wrap(\"{}\"));\n", u8::from(llmq_type), hash).as_bytes())
}

fn embed_copyright<W: Write>(class_name: &str, chain_name: &str, writer: &mut W) -> io::Result<()> {
    writer.write_all("// *********************************************\n".as_bytes())?;
    writer.write_all("// Class generated by dash-shared-core/dash-spv-masternode-processor using \"generate-dashj-tests\" feature \n".as_bytes())?;
    writer.write_all("// Before running this test: \n".as_bytes())?;
    writer.write_all(format!("// 1) Move contents of USER/Caches/logs/{class_name}.java into dashj/core/src/test/resources/org/bitcoinj/quorums folder\n").as_bytes())?;
    writer.write_all("// 2) Move other content from USER/Caches/logs into dashj/core/src/test/resources/org/bitcoinj/quorums folder\n".as_bytes())?;
    writer.write_all("// 3) Run ForwardingService to populate block store (need to setup properly)\n".as_bytes())?;
    writer.write_all("// 4) Move resulting \"forwarding-service-{chain_name}.spvchain\" into dashj/core/src/test/resources/org/bitcoinj/quorums folder\n".as_bytes())?;
    writer.write_all("// *********************************************\n".as_bytes())
}

fn embed_startup_params<W: Write>(class_name: &str, chain_name: &str, params_name: &str, writer: &mut W) -> io::Result<()> {
    writer.write_all("\tContext context;\n".as_bytes())?;
    writer.write_all("\tNetworkParameters params;\n".as_bytes())?;
    writer.write_all("\tBlockStore store;\n".as_bytes())?;
    writer.write_all("\t@Before\n".as_bytes())?;
    writer.write_all(format!("\tpublic void startup() throws BlockStoreException {{\n").as_bytes())?;
    writer.write_all(format!("\t\tparams = {params_name}.get();\n").as_bytes())?;
    writer.write_all("\t\tcontext = new Context(params);\n".as_bytes())?;
    writer.write_all(format!("\t\tstore = new SPVBlockStore(params, new File(Objects.requireNonNull({class_name}.class.getResource(\"forwarding-service-{chain_name}.spvchain\")).getPath()));\n").as_bytes())?;
    writer.write_all("\t\tcontext.initDash(true, true);\n".as_bytes())?;
    writer.write_all("\t\tcontext.blockChain = new BlockChain(context, store);\n".as_bytes())?;
    writer.write_all(format!("\t}}\n").as_bytes())
}

fn embed_masternode_list_parser<W: Write>(class_name: &str, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("\tstatic SimplifiedMasternodeList masternodeListFromJson(NetworkParameters params, String fileName) throws IOException {{\n").as_bytes())?;
    writer.write_all(format!("\t\tJsonNode json = new ObjectMapper().readTree(new InputStreamReader(Objects.requireNonNull({class_name}.class.getResourceAsStream(fileName)), StandardCharsets.UTF_8));\n").as_bytes())?;
    writer.write_all(format!("\t\tArrayList<SimplifiedMasternodeListEntry> nodes = new ArrayList<>();\n").as_bytes())?;
    writer.write_all(format!("\t\tfor (JsonNode node : json) {{\n").as_bytes())?;
    writer.write_all(format!("\t\t\tshort version = (short) node.get(\"version\").asInt();\n").as_bytes())?;
    writer.write_all(format!("\t\t\tKeyId keyid = null;\n").as_bytes())?;
    writer.write_all(format!("\t\t\tif (node.has(\"platform_node_id\")) {{\n").as_bytes())?;
    writer.write_all(format!("\t\t\t\tkeyid = KeyId.fromBytes(Hex.decode(node.get(\"platform_node_id\").asText()));\n").as_bytes())?;
    writer.write_all(format!("\t\t\t}}\n").as_bytes())?;
    writer.write_all(format!("\t\t\tnodes.add(new SimplifiedMasternodeListEntry(params, version, node.get(\"mn_type\").asInt(), Sha256Hash.wrapReversed(Sha256Hash.wrap(node.get(\"provider_registration_transaction_hash\").asText()).getBytes()), Sha256Hash.wrapReversed(Sha256Hash.wrap(node.get(\"confirmed_hash\").asText()).getBytes()), new MasternodeAddress(InetAddress.getByAddress(Utils.HEX.decode(node.get(\"ip_address\").asText())), node.get(\"port\").asInt()), KeyId.fromBytes(Hex.decode(node.get(\"key_id_voting\").asText())), new BLSLazyPublicKey(params, Hex.decode(node.get(\"operator_public_key\").asText()), 0, version < 2), keyid, node.get(\"platform_http_port\").asInt(), node.get(\"is_valid\").asBoolean()));\n").as_bytes())?;
    writer.write_all(format!("\t\t}}\n").as_bytes())?;
    writer.write_all(format!("\t\treturn new SimplifiedMasternodeList(params, nodes);\n").as_bytes())?;
    writer.write_all(format!("\t}}\n").as_bytes())
}

fn embed_snapshot_parser<W: Write>(class_name: &str, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("\tstatic QuorumSnapshot quorumSnapshotFromJson(String fileName) throws IOException {{\n").as_bytes())?;
    writer.write_all(format!("\t\tJsonNode json = new ObjectMapper().readTree(new InputStreamReader(Objects.requireNonNull({class_name}.class.getResourceAsStream(fileName)), StandardCharsets.UTF_8));\n").as_bytes())?;

    writer.write_all(format!("\t\tList<Integer> skipList = new ArrayList<>();\n").as_bytes())?;
    writer.write_all(format!("\t\tArrayNode skipListNode = (ArrayNode) json.get(\"skip_list\");\n").as_bytes())?;
    writer.write_all(format!("\t\tfor (JsonNode node : skipListNode) {{\n").as_bytes())?;
    writer.write_all(format!("\t\t\tskipList.add(node.asInt());\n").as_bytes())?;
    writer.write_all(format!("\t\t}}\n").as_bytes())?;

    writer.write_all(format!("\t\tList<Boolean> memberList = new ArrayList<>();\n").as_bytes())?;
    writer.write_all(format!("\t\tArrayNode memberListNode = (ArrayNode) json.get(\"member_list\");\n").as_bytes())?;
    writer.write_all(format!("\t\tfor (JsonNode node : memberListNode) {{\n").as_bytes())?;
    writer.write_all(format!("\t\t\tmemberList.add(node.asBoolean());\n").as_bytes())?;
    writer.write_all(format!("\t\t}}\n").as_bytes())?;

    writer.write_all(format!("\t\treturn new QuorumSnapshot(memberList, json.get(\"skip_list_mode\").asInt(), skipList);\n").as_bytes())?;
    writer.write_all(format!("\t}}\n").as_bytes())
}

fn embed_imports<W: Write>(writer: &mut W) -> io::Result<()> {
    writer.write_all("package org.bitcoinj.quorums;\n".as_bytes())?;
    writer.write_all("import com.fasterxml.jackson.databind.JsonNode;\n".as_bytes())?;
    writer.write_all("import com.fasterxml.jackson.databind.ObjectMapper;\n".as_bytes())?;
    writer.write_all("import com.fasterxml.jackson.databind.node.ArrayNode;\n".as_bytes())?;
    writer.write_all("import org.bitcoinj.core.*;\n".as_bytes())?;
    writer.write_all("import org.bitcoinj.crypto.*;\n".as_bytes())?;
    writer.write_all("import org.bitcoinj.evolution.*;\n".as_bytes())?;
    writer.write_all("import org.bitcoinj.params.*;\n".as_bytes())?;
    writer.write_all("import org.bitcoinj.store.*;\n".as_bytes())?;
    writer.write_all("import org.bouncycastle.util.encoders.Hex;\n".as_bytes())?;
    writer.write_all("import org.junit.*;\n".as_bytes())?;
    writer.write_all("import java.io.*;\n".as_bytes())?;
    writer.write_all("import java.net.InetAddress;\n".as_bytes())?;
    writer.write_all("import java.nio.charset.StandardCharsets;\n".as_bytes())?;
    writer.write_all("import java.util.*;\n".as_bytes())
}


fn print_quorum_members<W: Write>(quorum_members_var: &str, quorum_base_block_var: &str, quorum_name: &str, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("\t\tSystem.out.println(\"****************** getAllQuorumMembers ******************\" + {quorum_base_block_var}.getHeight());\n").as_bytes())?;
    writer.write_all(format!("\t\tArrayList<Masternode> {quorum_members_var} = state.getAllQuorumMembers(LLMQParameters.LLMQType.{quorum_name}, {quorum_base_block_var}.getHeader().getHash());\n").as_bytes())?;
    writer.write_all("\t\tSystem.out.println(\"******************* quorum_members *******************\");\n".as_bytes())?;
    writer.write_all("\t\tSystem.out.println(\"[\");\n".as_bytes())?;
    writer.write_all(format!("\t\tfor (Masternode m : {quorum_members_var}) {{\n").as_bytes())?;
    writer.write_all("\t\t\tSystem.out.println(m.getProTxHash().toString() + \",\");\n".as_bytes())?;
    writer.write_all(format!("\t\t}}\n").as_bytes())?;
    writer.write_all("\t\tSystem.out.println(\"]\");\n".as_bytes())
}

fn generate_test_class<F: Fn(&mut W), W: Write>(class_name: &str, chain_name: &str, params_name: &str, contents: F, writer: &mut W) -> io::Result<()> {
    embed_copyright(class_name, chain_name, writer)?;
    embed_imports(writer)?;
    writer.write_all(format!("public class {class_name} {{\n").as_bytes())?;
    embed_startup_params(class_name, chain_name, params_name, writer)?;
    embed_snapshot_parser(class_name, writer)?;
    embed_masternode_list_parser(class_name, writer)?;
    writer.write_all("\t@Test\n".as_bytes())?;
    writer.write_all(format!("\tpublic void testCase() throws IOException, BlockStoreException {{\n").as_bytes())?;
    contents(writer);
    writer.write_all(format!("\t}}\n").as_bytes())?;
    writer.write_all(format!("}}\n").as_bytes())
}

fn put_mn_list_to_cache_for_block_var_name<W: Write>(block_var: &str, nodes_var: &str, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("\t\tstate.mnListsCache.put({block_var}.getHeader().getHash(), new SimplifiedMasternodeList(params, {nodes_var}));\n").as_bytes())
}

fn put_snapshot_to_cache_for_block_var_name<W: Write>(block_var: &str, snapshot_hex: &str, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("\t\tstate.quorumSnapshotCache.put({block_var}.getHeader().getHash(), new QuorumSnapshot(params, Utils.HEX.decode(\"{snapshot_hex}\"), 0));\n").as_bytes())
}

fn block_from_store_for_height(height: u32) -> String {
    format!("store.get({height})")
}

fn generate_stored_block_from_store<W: Write>(block_var: &str, height: u32, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("\t\tStoredBlock {block_var} = store.get({height});\n").as_bytes())
}
fn generate_stored_block_from_ancestor<W: Write>(ancestor_block_var: &str, block_var: &str, ancestor_height: u32, writer: &mut W) -> io::Result<()> {
    writer.write_all(format!("\t\tStoredBlock {ancestor_block_var} = {block_var}.getAncestor(store, {ancestor_height});\n").as_bytes())
}

fn generate_commitment_verify<W: Write>(commitment_var: &str, block_var: &str, quorum_members_var: &str, writer: &mut W) -> io::Result<()> {
    writer.write_all("\t\tSystem.out.println(\"****************** verify ******************\");\n".as_bytes())?;
    writer.write_all(format!("\t\tboolean verified = {commitment_var}.verify({block_var}, {quorum_members_var}, true);\n").as_bytes())?;
    writer.write_all("\t\tSystem.out.println(\"verified: \" + verified + \",\");\n".as_bytes())?;
    writer.write_all("\t\tassertTrue(verified);\n".as_bytes())
}

pub fn save_snapshot_to_json(snapshot: &LLMQSnapshot, block_height: u32) {
    let file_name = format!("snapshot_{}.json", block_height);
    save_json_file(file_name.as_str(), snapshot)
        .expect("Can't save snapshot");
}

pub fn save_masternode_list_to_json(masternode_list: &models::MasternodeList, block_height: u32) {
    let masternodes = serde_json::to_value(masternode_list.masternodes.values().collect::<Vec<_>>()).unwrap();
    let file_name = format!("masternodes_{}.json", block_height);
    save_json_file(file_name.as_str(), &masternodes)
        .expect("Can't save masternodes");
    let file_name = format!("quorums_{}.json", block_height);
    let quorums = masternode_list.quorums.values()
        .flat_map(|inner_map| inner_map.values())
        .cloned()
        .collect::<Vec<_>>();
    let quorums = serde_json::to_value(quorums)
        .expect("Can't serialize quorums");
    save_json_file(file_name.as_str(), &quorums)
        .expect("Can't save quorums");
    // info!("SimplifiedMasternodeList list_at_{block_height} = SimplifiedMasternodeList.fromJson(\"{file_name}\");");
}

fn masternodes_json_name(height: u32) -> String {
    format!("masternodes_{height}.json")
}

fn snapshot_json_name(height: u32) -> String {
    format!("snapshot_{height}.json")
}

fn work_block_var(height: u32) -> String {
    format!("workBlock_{height}")
}

fn put_snapshot_from_json_into_cache(height: u32) -> String {
    // let block_var = work_block_var(height);
    let block_var = block_from_store_for_height(height);
    let file_name = snapshot_json_name(height);
    format!("\t\tstate.quorumSnapshotCache.put({block_var}.getHeader().getHash(), quorumSnapshotFromJson(\"{file_name}\"));\n")
}

fn put_masternode_list_from_json_into_cache(height: u32) -> String {
    // let block_var = work_block_var(height);
    let block_var = block_from_store_for_height(height);
    let file_name = masternodes_json_name(height);
    format!("\t\tstate.mnListsCache.put({block_var}.getHeader().getHash(), masternodeListFromJson(params, \"{file_name}\"));\n")
}

fn extract_masternode_lists_and_snapshots(result: types::QRInfoResult) -> (u32, Vec<u32>, Vec<u32>) {
    // unsafe {
    //     let list_at_h_4c = (*(*result.result_at_h_4c).masternode_list).decode();
    //     let list_at_h_3c = (*(*result.result_at_h_3c).masternode_list).decode();
    //     let list_at_h_2c = (*(*result.result_at_h_2c).masternode_list).decode();
    //     let list_at_h_c = (*(*result.result_at_h_c).masternode_list).decode();
    //     let list_at_h = (*(*result.result_at_h).masternode_list).decode();
    //     let list_at_tip = (*(*result.result_at_tip).masternode_list).decode();
    //     let h_4c = list_at_h_4c.known_height;
    //     let h_3c = list_at_h_3c.known_height;
    //     let h_2c = list_at_h_2c.known_height;
    //     let h_c = list_at_h_c.known_height;
    //     let h = list_at_h.known_height;
    //     let h_tip = list_at_tip.known_height;
    //     let mut masternode_lists = vec![
    //         (h_4c, list_at_h_4c),
    //         (h_3c, list_at_h_3c),
    //         (h_2c, list_at_h_2c),
    //         (h_c, list_at_h_c),
    //         (h, list_at_h),
    //         (h_tip, list_at_tip),
    //     ];
    //     let mut snapshots = vec![
    //         (h_4c, (*result.snapshot_at_h_4c).decode()),
    //         (h_3c, (*result.snapshot_at_h_3c).decode()),
    //         (h_2c, (*result.snapshot_at_h_2c).decode()),
    //         (h_c, (*result.snapshot_at_h_c).decode())
    //     ];
    //     (0..result.mn_list_diff_list_count)
    //         .into_iter()
    //         .for_each(|i| {
    //             let list = (*(*(*result.mn_list_diff_list.add(i))).masternode_list).decode();
    //             let h = list.known_height;
    //             snapshots.push((h, (*(*result.quorum_snapshot_list.add(i))).decode()));
    //             masternode_lists.push((h, list));
    //         });
    //     (masternode_list_heights, snapshot_heights)
    // }
    unsafe {
        let list_at_h_4c = (*(*result.result_at_h_4c).masternode_list).decode();
        let list_at_h_3c = (*(*result.result_at_h_3c).masternode_list).decode();
        let list_at_h_2c = (*(*result.result_at_h_2c).masternode_list).decode();
        let list_at_h_c = (*(*result.result_at_h_c).masternode_list).decode();
        let list_at_h = (*(*result.result_at_h).masternode_list).decode();
        let list_at_tip = (*(*result.result_at_tip).masternode_list).decode();
        let h_4c = list_at_h_4c.known_height;
        let h_3c = list_at_h_3c.known_height;
        let h_2c = list_at_h_2c.known_height;
        let h_c = list_at_h_c.known_height;
        let h = list_at_h.known_height;
        let h_tip = list_at_tip.known_height;
        let mut masternode_lists = vec![h_4c, h_3c, h_2c, h_c, h, h_tip];
        let mut snapshots = vec![h_4c, h_3c, h_2c, h_c];
        (0..result.mn_list_diff_list_count)
            .into_iter()
            .for_each(|i| {
                let list = (*(*(*result.mn_list_diff_list.add(i))).masternode_list).decode();
                let h = list.known_height;
                snapshots.push(h);
                masternode_lists.push(h);
            });
        (h, masternode_lists, snapshots)
    }
}

pub fn generate_qr_state_test_file_json(chain_type: ChainType, result: types::QRInfoResult) {
    let (quorum_base_block_height, lists, snapshots) = extract_masternode_lists_and_snapshots(result);
    let class_name = format!("QuorumRotationStateTest_{quorum_base_block_height}");
    let chain_name = get_chain_name_for_chain_type(chain_type);
    let params_name = get_params_name_for_chain_type(chain_type);
    let quorum_name = get_llmq_name_for_llmq_type(LLMQType::Llmqtype60_75);
    let mut writer = Vec::<u8>::new();
    generate_test_class(class_name.as_str(), chain_name, params_name, |mut w| {
        let quorum_base_block_var = "quorumBaseBlock";
        w.write_all("\t\tQuorumRotationState state = new QuorumRotationState(context);\n".as_bytes())
            .expect("");
        w.write_all("\t\tstate.setBlockChain(context.blockChain, context.blockChain);\n".as_bytes())
            .expect("");
        generate_stored_block_from_store(quorum_base_block_var, quorum_base_block_height, &mut w).expect("");
        for height in &snapshots {
            w.write_all(put_snapshot_from_json_into_cache(*height).as_bytes())
                .expect("");
        }
        for height in &lists {
            w.write_all(put_masternode_list_from_json_into_cache(*height).as_bytes())
                .expect("");
        }
        print_quorum_members("quorum_members", quorum_base_block_var, quorum_name, &mut w)
            .expect("");
    }, &mut writer)
        .expect("Can't generate QuorumRotationState class");
    save_java_class(format!("{class_name}.java").as_str(), &writer)
        .expect("Can't write java class");
}

pub fn generate_final_commitment_test_file(chain_type: ChainType, quorum_base_block_height: u32, quorum: &LLMQEntry, masternodes: &Vec<MasternodeEntry>) {
    let class_name = format!("FinalCommiment_{quorum_base_block_height}_Test");
    let chain_name = get_chain_name_for_chain_type(chain_type);
    let params_name = get_params_name_for_chain_type(chain_type);
    let quorum_name = get_llmq_name_for_llmq_type(quorum.llmq_type);
    let mut writer = Vec::<u8>::new();
    generate_test_class(class_name.as_str(), chain_name, params_name, |mut w| {
        let quorum_base_block_var = "quorumBaseBlock";
        let quorum_members_var = "quorum_members";
        let nodes_var = "nodes";
        w.write_all("\t\tQuorumState state = new QuorumState(context, MasternodeListSyncOptions.SYNC_MINIMUM);\n".as_bytes())
            .expect("");
        w.write_all("\t\tstate.setBlockChain(context.blockChain, context.blockChain);\n".as_bytes())
            .expect("");
        generate_final_commitment(quorum, &mut w)
            .expect("");
        generate_stored_block_from_store(quorum_base_block_var, quorum_base_block_height, &mut w)
            .expect("");
        generate_mn_array_list(masternodes, quorum_base_block_height, nodes_var, &mut w)
            .expect("");
        put_mn_list_to_cache_for_block_var_name(quorum_base_block_var, nodes_var, &mut w)
            .expect("");
        print_quorum_members(quorum_members_var, quorum_base_block_var, quorum_name, &mut w)
            .expect("");
        generate_commitment_verify("finalCommitment", quorum_base_block_var, quorum_members_var, &mut w)
            .expect("Can't generate FinalCommitment");
    }, &mut writer)
        .expect("Can't generate FinalCommitment class");
    save_java_class(format!("{class_name}.java").as_str(), &writer)
        .expect("Can't write java class");
}
