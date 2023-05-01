use crate::chain::Chain;
use crate::chain::common::{ChainType, IHaveChainSettings};
use crate::chain::params::{BIP32ScriptMap, DIP14ScriptMap, ScriptMap};
use crate::crypto::UInt256;

pub trait Settings {
    fn r#type(&self) -> ChainType;
    fn genesis(&self) -> UInt256;
    fn coin_type(&self) -> u32;
    fn is_mainnet(&self) -> bool {
        self.r#type() == ChainType::MainNet
    }
    fn is_testnet(&self) -> bool {
        self.r#type() == ChainType::TestNet
    }
    fn is_devnet_any(&self) -> bool {
        !self.is_mainnet() && !self.is_testnet()
    }
    fn is_evolution_enabled(&self) -> bool { false }
    fn base_reward(&self) -> u64;
    fn script(&self) -> ScriptMap;
    fn bip32(&self) -> BIP32ScriptMap;
    fn dip14(&self) -> DIP14ScriptMap;

}

impl Settings for Chain {
    fn r#type(&self) -> ChainType {
        self.params.chain_type
    }

    fn genesis(&self) -> UInt256 {
        self.r#type().genesis_hash()
    }

    fn coin_type(&self) -> u32 {
        self.r#type().coin_type()
    }

    fn base_reward(&self) -> u64 {
        self.params.base_reward
    }

    fn script(&self) -> ScriptMap {
        self.r#type().script_map()
    }

    fn bip32(&self) -> BIP32ScriptMap {
        self.r#type().bip32_script_map()
    }

    fn dip14(&self) -> DIP14ScriptMap {
        self.r#type().dip14_script_map()
    }

}
