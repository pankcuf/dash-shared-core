use std::error::Error;
use std::thread;
// use std::io::{self, BufRead};
// use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
// use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
// use tokio::net::TcpStream;
use dash_spv_masternode_processor::chain::{Chain, Wallet};
use dash_spv_masternode_processor::chain::common::ChainType;
use dash_spv_masternode_processor::chain::ext::sync::SyncProgress;
use dash_spv_masternode_processor::chain::ext::wallets::WalletCreation;
use dash_spv_masternode_processor::util::TimeUtil;

fn main() -> Result<(), Box<dyn Error>> {
    let chain_type = ChainType::TestNet;
    let seed_phrase = "upper renew that grow pelican pave subway relief describe enforce suit hedgehog blossom dose swallow";
    let shared_testnet = Chain::create_shared_testnet();
    shared_testnet.wallet_with_seed_phrase::<bip0039::English>(seed_phrase, false, SystemTime::seconds_since_1970(), chain_type);
    let mut testnet = shared_testnet.try_write().unwrap();
    println!("Chain -> start sync");
    testnet.start_sync();
    println!("Chain -> start sync .... sleep 10 sec");
    thread::sleep(Duration::from_secs(10));
    println!("Chain -> stop sync");
    testnet.stop_sync();
    println!("Chain -> stop sync .... exit");
    Ok(())
}

// fn main() -> Result<(), Box<dyn Error>> {
//     use dash_spv_masternode_processor::manager::PeerManager;
//     use dash_spv_masternode_processor::chain::wallet::ext::constants::BIP39_CREATION_TIME;
//
//     let mut peer_manager = PeerManager::new(chain_type);
//     peer_manager.chain = chain.borrow();
//     peer_manager.connect(BIP39_CREATION_TIME);
//
//     Ok(())
// }
