use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use reachability::ReachabilityManager;
use crate::chain::Chain;
use crate::chain::common::chain_type::DevnetType;
use crate::chain::common::ChainType;
use crate::chain::ext::wallets::WalletCreation;
use crate::default_shared;
use crate::environment::{Environment, Language};
use crate::manager::authentication_manager::AuthenticationManager;
use crate::util::Shared;
use crate::util::shared::Shareable;

pub const DEVNET_CHAINS_KEY: &str = "DEVNET_CHAINS_KEY";

pub struct ChainsManager {
    pub mainnet: Arc<RwLock<Chain>>,
    pub testnet: Arc<RwLock<Chain>>,
    pub devnet_chains: HashMap<DevnetType, Arc<RwLock<Chain>>>,
    // pub mainnet: Arc<RwLock<Chain>>,
    // pub testnet: Arc<RwLock<Chain>>,
    // pub devnet_chains: Arc<Vec<RwLock<Chain>>>,
    pub environment: Environment,
    pub authentication_manager: AuthenticationManager,
    pub reachability_manager: ReachabilityManager,

}
default_shared!(Vec<Chain>);
impl Shareable for ChainsManager {}
impl ChainsManager {
    pub fn new() -> Self {
        ChainsManager {
            mainnet: Chain::create_shared_mainnet(),
            testnet: Chain::create_shared_testnet(),
            devnet_chains: HashMap::new(),
            environment: Environment::new(Language::English),
            authentication_manager: AuthenticationManager::default(),
            reachability_manager: ReachabilityManager::new(),
        }
    }
    pub fn new_shared() -> Shared<Self> {
        Self::new().to_shared()
    }
}

impl ChainsManager {

    pub fn wallet_with_seed_phrase<L: bip0039::Language>(self, seed_phrase: &str, is_transient: bool, created_at: u64, chain_type: ChainType) {
        match chain_type {
            ChainType::MainNet =>
                self.mainnet.wallet_with_seed_phrase::<L>(seed_phrase, is_transient, created_at, chain_type),
            ChainType::TestNet =>
                self.testnet.wallet_with_seed_phrase::<L>(seed_phrase, is_transient, created_at, chain_type),
            _ => {}
            // ChainType::DevNet(devnet_type) =>
            //     self.devnet_chains.get(&devnet_type)
            //         .and_then(|devnet| devnet.wallet_with_seed_phrase::<L>(seed_phrase, is_transient, created_at, chain_type))
        }
    }
}
