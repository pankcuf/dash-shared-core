use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use crate::chain::common::{DevnetType, IHaveChainSettings};
use crate::chain::ext::Settings;
use crate::chain::params::{create_devnet_params_for_type, MAINNET_PARAMS, Params, TESTNET_PARAMS};
use crate::chain::{block, Checkpoint, spork, SyncPhase, Wallet};
use crate::chain::spork::Spork;
use crate::chain::block::{BLOCK_UNKNOWN_HEIGHT, IBlock, MerkleBlock};
use crate::chain::network::Peer;
use crate::chain::network::peer::WEEK_TIME_INTERVAL;
use crate::chain::wallet::Account;
use crate::chain::wallet::ext::constants::BIP39_CREATION_TIME;
use crate::crypto::{byte_util::Zeroable, UInt256};
use crate::default_shared;
use crate::chain::derivation::{DerivationPath, DerivationPathKind, factory::Factory};
use crate::manager::{AuthenticationManager, MasternodeManager, PeerManager, TransactionManager};
use crate::storage::context::StoreContext;
use crate::storage::{Keychain, UserDefaults};
use crate::util::shared::{Shareable, Shared};

/// This is about the time if we consider a block every 10 mins (for 500 blocks)
pub const HEADER_WINDOW_BUFFER_TIME: u64 = WEEK_TIME_INTERVAL / 2;

#[derive(Clone, Debug, Default)]
pub struct Chain {
    pub params: Params,
    pub wallets: Vec<Wallet>,
    pub store_context: StoreContext,
    pub derivation_factory: Factory,

    pub last_sync_block: Option<Arc<block::Kind>>,
    pub last_terminal_block: Option<Arc<block::Kind>>,
    // pub last_sync_block: Option<Weak<block::Kind>>,
    // pub last_terminal_block: Option<Weak<block::Kind>>,
    pub terminal_blocks: HashMap<UInt256, Arc<block::Kind>>,
    pub sync_blocks: HashMap<UInt256, Arc<block::Kind>>,

    pub authentication_manager: Shared<AuthenticationManager>,
    pub masternode_manager: MasternodeManager,
    pub peer_manager: PeerManager,
    pub spork_manager: spork::Manager,
    pub transaction_manager: TransactionManager,
    pub sync_phase: SyncPhase,
    viewing_account: Account,

    // pub network_context: NetworkContext,


    pub chain_sync_start_height: u32,
    pub terminal_sync_start_height: u32,
    best_estimated_block_height: u32,
    pub estimated_block_heights: HashMap<u32, Vec<Peer>>,

    /// An array of known hardcoded checkpoints for the chain
    pub checkpoints: Vec<Checkpoint>,
    terminal_headers_override_use_checkpoint: Option<Checkpoint>,
    sync_headers_override_use_checkpoint: Option<Checkpoint>,
    // last_checkpoint: Option<Checkpoint>,


    last_persisted_chain_sync_block_hash: UInt256,
    last_persisted_chain_sync_block_chain_work: UInt256,
    last_persisted_chain_sync_block_height: u32,
    last_persisted_chain_sync_timestamp: u64,
    last_persisted_chain_sync_locators: Vec<UInt256>,

    last_persisted_chain_terminal_block_hash: UInt256,
    last_persisted_chain_terminal_block_chain_work: UInt256,
    last_persisted_chain_terminal_block_height: u32,
    last_persisted_chain_terminal_timestamp: u64,
    last_persisted_chain_terminal_locators: Vec<UInt256>,

    pub sync_from_genesis: bool,
    pub should_sync_from_height: bool,
    pub sync_from_height: u32,

}

impl Shareable for Chain {}

default_shared!(Chain);
default_shared!(Vec<Wallet>);

impl PartialEq<Self> for Chain {
    fn eq(&self, other: &Self) -> bool {
        self == other || other.r#type().genesis_hash().eq(&self.r#type().genesis_hash())
    }
}

impl Eq for Chain {}

impl Chain {

    pub fn setup(&mut self) {
        self.restore_wallets_and_standalone_derivation_paths();
    }

    pub fn set_sync_from_genesis(&mut self, sync_from_genesis: bool) {
        if sync_from_genesis {
            self.sync_from_height = 0;
            self.should_sync_from_height = true;
        } else if let Some(sync_from_height) = UserDefaults::uint_for_key::<u32>("syncFromHeight") {
            if self.sync_from_height == 0 {
                UserDefaults::delete("syncFromHeight");
                self.should_sync_from_height = false;
            }
        }
    }

    pub fn sync_from_genesis(&self) -> bool {
        if UserDefaults::has("syncFromHeight") {
            self.sync_from_height == 0 && self.should_sync_from_height
        } else {
            false
        }
        // UserDefaults::object_for_key::<u32>("syncFromHeight")
        //     .map_or(false, self.sync_from_height == 0 && self.should_sync_from_height)

    }



    pub fn should_continue_sync_blockchain_after_height(&mut self, height: u32) -> bool {
        self.r#type().syncs_blockchain() &&
            (self.last_sync_block_height() != self.last_terminal_block_height()) ||
            self.last_sync_block_height() < height
    }

    pub fn restore_wallets_and_standalone_derivation_paths(&mut self) {

        // match Keychain::get_array::<String>(self.r#type().chain_wallets_key(), vec![]) {
        //     Ok(wallet_ids) => {
        //         wallet_ids.into_iter().for_each(|wallet_id| {
        //             self.add_wallet(Wallet::init_with_chain_and_unique_id(wallet_id, false, self.clone()));
        //         });
        //         // we should load identities after all wallets are in the chain, as identities
        //         // might be on different wallets and have interactions between each other
        //         self.wallets.iter().for_each(|wallet| wallet.load_identities());
        //     },
        //     Err(err) => println!("Error restoring wallets {:?}", err)
        // }
        // match Keychain::get_array::<String>(self.r#type().chain_standalone_derivation_paths_key(), vec![]) {
        //     Ok(derivation_path_ids) => {
        //         derivation_path_ids.into_iter().for_each(|derivation_path_id| {
        //             if let Some(path) = DerivationPath::init_with_extended_public_key_identifier(derivation_path_id.as_str(), self.r#type(), self.borrow()) {
        //                 self.add_standalone_derivation_path(DerivationPathKind::Default(path));
        //             }
        //         });
        //     },
        //     Err(err) => println!("Error restoring standalone derivation paths {:?}", err)
        // }
    }
}

// /// Managers
// impl Arc<Chain> {
//
//
//
    // pub fn restore_wallets_and_standalone_derivation_paths(&mut self) {
        // self.with(|chain| {
        //     match Keychain::get_array::<String>(chain.r#type().chain_wallets_key(), vec![]) {
        //         Ok(wallet_ids) => {
        //             wallet_ids.into_iter().for_each(|wallet_id| {
        //
        //                 chain.add_wallet(Wallet::init_with_chain_and_unique_id(wallet_id, false, self.clone()));
        //             });
        //             // we should load identities after all wallets are in the chain, as identities
        //             // might be on different wallets and have interactions between each other
        //             chain.wallets.iter().for_each(|wallet| wallet.load_identities());
        //         },
        //         Err(err) => println!("Error restoring wallets {:?}", err)
        //     }
        //     match Keychain::get_array::<String>(chain.r#type().chain_standalone_derivation_paths_key(), vec![]) {
        //         Ok(derivation_path_ids) => {
        //             derivation_path_ids.into_iter().for_each(|derivation_path_id| {
        //                 if let Some(path) = DerivationPath::init_with_extended_public_key_identifier(derivation_path_id.as_str(), chain.r#type(), self.borrow()) {
        //                     chain.add_standalone_derivation_path(DerivationPathKind::Default(path));
        //                 }
        //             });
        //         },
        //         Err(err) => println!("Error restoring standalone derivation paths {:?}", err)
        //     }
        // });
    // }
//
//
// }

impl Chain {
    fn create_shared(params: Params) -> Arc<RwLock<Self>> {
        let chain_type = params.chain_type.clone();
        let chain = Self {
            params,
            store_context: StoreContext::new(),
            derivation_factory: Factory::new(),
            authentication_manager: Default::default(),
            masternode_manager: MasternodeManager::new(chain_type),
            peer_manager: PeerManager::new(chain_type),
            spork_manager: spork::Manager::new(chain_type),
            transaction_manager: TransactionManager::new(chain_type),
            viewing_account: Account::view_only_account_with_number(0),
            // network_context: NetworkContext::new(),
            ..Default::default()
        };
        let arc_chain = Arc::new(RwLock::new(chain));
        if let Ok(mut c) = arc_chain.try_write() {
            c.masternode_manager.chain = Shared::RwLock(arc_chain.clone());
            c.peer_manager.chain = Shared::RwLock(arc_chain.clone());
            c.spork_manager.chain = Shared::RwLock(arc_chain.clone());
            c.transaction_manager.chain = Shared::RwLock(arc_chain.clone());
            match Keychain::get_wallet_ids(chain_type) {
                Ok(wallet_ids) => {
                    c.wallets.extend(wallet_ids.into_iter().map(|unique_id| Wallet::init_with_chain_and_unique_id(unique_id, false, arc_chain.clone())));
                    // we should load identities after all wallets are in the chain, as identities
                    // might be on different wallets and have interactions between each other
                    c.wallets.iter().for_each(|wallet| wallet.load_identities())
                },
                Err(err) => println!("Error restoring wallets {:?}", err)
            }
            match Keychain::get_standalone_derivation_path_ids(chain_type) {
                Ok(derivation_path_ids) => {
                    derivation_path_ids.into_iter().for_each(|derivation_path_id| {
                        if let Some(path) = DerivationPath::init_with_extended_public_key_identifier(derivation_path_id.as_str(), chain_type, Weak::new()) {
                            c.add_standalone_derivation_path(DerivationPathKind::Default(path));
                        }
                    });
                },
                Err(err) => println!("Error restoring standalone derivation paths {:?}", err)
            }
        }
        arc_chain
    }

    pub fn create_shared_mainnet() -> Arc<RwLock<Self>> {
        Self::create_shared(MAINNET_PARAMS)
    }

    pub fn create_shared_testnet() -> Arc<RwLock<Self>> {
        Self::create_shared(TESTNET_PARAMS)
    }

    pub fn create_shared_devnet(r#type: DevnetType) -> Arc<RwLock<Self>> {
        Self::create_shared(create_devnet_params_for_type(r#type))
    }
}

impl Chain {
    // pub fn add_wallet(&mut self, wallet: Arc<RwLock<Wallet>>) -> bool {
    //     let not_present_yet = self.wallets.iter().find(|w| wallet.unique_id_string() == w.unique_id_string()).is_none();
    //     if not_present_yet {
    //         self.wallets.push(wallet);
    //     }
    //     not_present_yet
    // }

    pub fn add_standalone_derivation_path(&mut self, path: DerivationPathKind) {
        self.viewing_account.add_derivation_path(path);
        // if let Some(mut acc) = self.viewing_account.take() {
        //     acc.add_derivation_path(path);
        // } else {
        //     let mut acc = Account::view_only_account_with_number(0);
        //     acc.add_derivation_path(path);
        //     self.viewing_account = Some(acc);
        // }
    }


    pub fn start_sync_from_time(&mut self) -> u64 {
        if self.r#type().syncs_blockchain() {
            self.earliest_wallet_creation_time()
        } else {
            self.checkpoints.last().map_or(0, |checkpoint| checkpoint.timestamp.into())
        }
    }


    pub fn last_sync_block_with_use_checkpoints(&mut self, use_checkpoints: bool) -> Option<Arc<block::Kind>> {
        self.last_sync_block.take().or_else(|| {
            let mut last: Option<Arc<block::Kind>> = None;
            if !self.last_persisted_chain_sync_block_hash.is_zero() &&
                !self.last_persisted_chain_sync_block_chain_work.is_zero() &&
                self.last_persisted_chain_sync_block_height != BLOCK_UNKNOWN_HEIGHT as u32 {
                last = Some(Arc::new(block::Kind::MerkleBlock(MerkleBlock::with(
                    2,
                    self.last_persisted_chain_sync_block_hash.clone(),
                    UInt256::MIN,
                    UInt256::MIN,
                    self.last_persisted_chain_sync_timestamp as u32,
                    0,
                    self.last_persisted_chain_sync_block_chain_work,
                    0,
                    0,
                    vec![],
                    vec![],
                    self.last_persisted_chain_sync_block_height,
                    None,
                    self.r#type(),
                    Shared::BorrowedRwLock(Weak::new()),
                ))));
                self.last_sync_block = last.clone();
            }
            if self.last_sync_block.is_none() && use_checkpoints {
                println!("No last Sync Block, setting it from checkpoints");
                self.set_last_sync_block_from_checkpoints();
            }
            last
        })
    }

    // pub fn set_last_sync_block_from_checkpoints(&mut self) {
    //     let checkpoint = if let Some(cp) = self.sync_headers_override_use_checkpoint.as_ref() {
    //         Some(cp)
    //     } else if self.options.sync_from_genesis() {
    //         self.checkpoints.get(self.r#type().genesis_height() as usize)
    //     } else if self.options.should_sync_from_height {
    //         self.last_checkpoint_on_or_before_height(self.options.sync_from_height)
    //     } else {
    //         let start_sync_time = self.start_sync_from_time();
    //         let timestamp = if start_sync_time as u32 == BIP39_CREATION_TIME { BIP39_CREATION_TIME } else { (start_sync_time - HEADER_WINDOW_BUFFER_TIME) as u32 };
    //         self.last_checkpoint_on_or_before_timestamp(timestamp)
    //     };
    //     if let Some(cp) = checkpoint {
    //         if let Some(sb) = self.sync_blocks.get(&cp.hash) {
    //             self.last_sync_block = Some(Arc::downgrade(sb));
    //         } else {
    //             let sb = Arc::new(block::Kind::MerkleBlock(MerkleBlock::init_with_checkpoint(&cp, self.r#type(), Shared::None)));
    //             self.last_sync_block = Some(Arc::downgrade(&sb));
    //             self.sync_blocks.insert(cp.hash.clone(), sb);
    //         }
    //     }
    // }
    pub fn set_last_sync_block_from_checkpoints(&mut self) {
        let checkpoint = if let Some(cp) = self.sync_headers_override_use_checkpoint.as_ref() {
            Some(cp)
        } else if self.sync_from_genesis() {
            self.checkpoints.get(self.r#type().genesis_height() as usize)
        } else if self.should_sync_from_height {
            self.last_checkpoint_on_or_before_height(self.sync_from_height)
        } else {
            let start_sync_time = self.start_sync_from_time();
            let timestamp = if start_sync_time == BIP39_CREATION_TIME { BIP39_CREATION_TIME } else { start_sync_time - HEADER_WINDOW_BUFFER_TIME };
            self.last_checkpoint_on_or_before_timestamp(timestamp as u32)
        };
        if let Some(cp) = checkpoint {
            if let Some(sb) = self.sync_blocks.get(&cp.hash) {
                self.last_sync_block = Some(sb.clone());
            } else {
                let sb = Arc::new(block::Kind::MerkleBlock(MerkleBlock::init_with_checkpoint(&cp, self.r#type(), Shared::BorrowedRwLock(Weak::new()))));
                self.sync_blocks.insert(cp.hash.clone(), sb.clone());
                self.last_sync_block = Some(sb);
            }
        }
    }
    pub fn set_last_terminal_block_from_checkpoints(&mut self) {
        let checkpoint = if let Some(cp) = self.terminal_headers_override_use_checkpoint.as_ref() {
            Some(cp)
        } else if let Some(cp) = self.last_checkpoint() {
            Some(cp)
        } else {
            None
        };
        if let Some(cp) = checkpoint {
            if let Some(tb) = self.terminal_blocks.get(&cp.hash) {
                self.last_terminal_block = Some(tb.clone());
            } else {
                let tb = Arc::new(block::Kind::MerkleBlock(MerkleBlock::init_with_checkpoint(&cp, self.r#type(), Shared::BorrowedRwLock(Weak::new()))));
                self.terminal_blocks.insert(cp.hash.clone(), tb.clone());
                self.last_terminal_block = Some(tb);
            }
        }
    }
    // pub fn set_last_terminal_block_from_checkpoints(&mut self) {
    //     if let Some(cp) = self.terminal_headers_override_use_checkpoint.as_ref() {
    //         if let Some(tb) = self.terminal_blocks.get(&cp.hash) {
    //             self.last_terminal_block = Some(Arc::downgrade(tb));
    //         } else {
    //             let tb = Arc::new(block::Kind::MerkleBlock(MerkleBlock::init_with_checkpoint(&cp, self.r#type(), Shared::None)));
    //             self.last_terminal_block = Some(Arc::downgrade(&tb));
    //             self.terminal_blocks.insert(cp.hash.clone(), tb);
    //         }
    //     } else if let Some(cp) = self.last_checkpoint() {
    //         if let Some(tb) = self.terminal_blocks.get(&cp.hash) {
    //             self.last_terminal_block = Some(Arc::downgrade(tb));
    //         } else {
    //             let tb = Arc::new(block::Kind::MerkleBlock(MerkleBlock::init_with_checkpoint(&cp, self.r#type(), Shared::None)));
    //             self.last_terminal_block = Some(Arc::downgrade(&tb));
    //             self.terminal_blocks.insert(cp.hash.clone(), tb);
    //         }
    //     }
    // }

    pub fn last_sync_block(&mut self) -> Option<Arc<block::Kind>> {
        self.last_sync_block_with_use_checkpoints(true)
    }

    pub fn last_terminal_block(&mut self) -> Option<Arc<block::Kind>> {
        self.last_terminal_block.take().or({
            // if let Ok(entity) = BlockEntity::get_last_terminal_block(self.r#type(), self.chain_context()) {
            //     if let Some(b) = MerkleBlock::from_entity(&entity, self) {
            //         self.last_terminal_block = Some(&b);
            //         println!("last terminal block at height {} recovered from db (hash is {})", b.height(), b.block_hash());
            //     }
            // }
            if self.last_terminal_block.is_none() {
                let last_sync_block_height = self.last_sync_block_height();
                // if we don't have any headers yet, use the latest checkpoint
                let last_checkpoint = if let Some(cp) = self.terminal_headers_override_use_checkpoint.as_ref() {
                    Some(cp)
                } else if let Some(cp) = self.last_checkpoint() {
                    Some(cp)
                } else {
                    None
                };
                if last_checkpoint.is_some() && last_checkpoint.unwrap().height >= last_sync_block_height {
                    self.set_last_terminal_block_from_checkpoints();
                } else {
                    self.last_terminal_block = self.last_sync_block.clone();
                }
            }
            let best_height = self.last_terminal_block.as_ref().map_or(0, |b| b.height());
            if best_height > self.estimated_block_height() {
                self.best_estimated_block_height = best_height;
            }
            self.last_terminal_block.clone()
        })
    }

    pub fn last_sync_block_height(&mut self) -> u32 {
        self.last_sync_block()
            .map_or(0, |b| b.height())
    }


    pub fn last_terminal_block_height(&mut self) -> u32 {
        self.last_terminal_block()
            .map_or(0, |b| b.height())
    }

    pub fn block_for_block_hash(&self, block_hash: &UInt256) -> Option<Arc<block::Kind>> {
        self.sync_blocks.get(block_hash).cloned()
            .or(self.terminal_blocks.get(block_hash).cloned())
            // .or(
            //     if self.params.allow_insight_blocks_for_verification() {
            //         self.insight_verified_blocks_by_hash_dictionary.get(block_hash).copied()
            //     } else {
            //         None
            //     }
            // )
    }



    /// this is used as part of a getblocks/getheaders request
    /// append 10 most recent block checkpointHashes, decending, then continue appending, doubling the step back each time,
    /// finishing with the genesis block (top, -1, -2, -3, -4, -5, -6, -7, -8, -9, -11, -15, -23, -39, -71, -135, ..., 0)
    pub fn block_locator_array_for_block(&self, block: Option<Arc<block::Kind>>) -> Vec<UInt256> {
        let mut locators = Vec::<UInt256>::new();
        let mut step = 1i32;
        let mut start = 0i32;
        let mut b = block.as_ref();
        let mut last_height = b.map_or(0, |b| b.height());
        while b.is_some() && b.unwrap().height() > 0 {
            locators.push(b.unwrap().block_hash());
            last_height = b.unwrap().height();
            start += 1;
            if start >= 10 {
                step *= 2;
            }
            let mut i = 0;
            while b.is_some() && i < step {
                let prev = b.unwrap().prev_block();
                b = self.sync_blocks.get(&prev);
                if b.is_none() {
                    b = self.terminal_blocks.get(&prev);
                }
                i += 1;
            }
        }
        let mut last_checkpoint: Option<&Checkpoint> = None;
        // add the last checkpoint we know about previous to this block
        for checkpoint in &self.checkpoints {
            if checkpoint.height < last_height && checkpoint.timestamp < b.unwrap().timestamp() {
                last_checkpoint = Some(checkpoint);
            } else {
                break;
            }
        }
        if let Some(last) = last_checkpoint {
            locators.push(last.hash);
        }
        locators
    }

    pub fn terminal_block_locators_array(&self) -> Vec<UInt256> {
        let mut locators = Vec::<UInt256>::new();
        let mut step = 1;
        let mut start = 0;
        let mut last_height = 0;//b.height;
        let mut b = self.last_terminal_block.as_ref();
        while b.is_some() && b.unwrap().height() > 0 {
            locators.push(b.unwrap().block_hash());
            last_height = b.unwrap().height();
            start += 1;
            if start >= 10 {
                step *= 2;
            }
            let mut i = 0;
            while b.is_some() && i < step {
                b = self.terminal_blocks.get(&b.unwrap().prev_block());
                i += 1;
            }
        }
        // then add the last checkpoint we know about previous to this header
        if let Some(last_checkpoint) = self.checkpoints.iter().find(|checkpoint| checkpoint.height < last_height) {
            locators.push(last_checkpoint.hash);
        }
        locators
    }


    /// Checkpoints

    pub fn block_height_has_checkpoint(&self, block_height: u32) -> bool {
        self.last_checkpoint_on_or_before_height(block_height)
            .map_or(false, |checkpoint| checkpoint.height == block_height)
    }

    pub fn last_checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }

    fn last_checkpoint_where<F>(&self, expression: F) -> Option<&Checkpoint> where F: Fn(&Checkpoint) -> bool {
        self.checkpoints
            .iter()
            .rev()
            .find(|checkpoint| checkpoint.height == self.r#type().genesis_height() || !self.r#type().syncs_blockchain() || expression(checkpoint))
    }

    pub fn last_checkpoint_on_or_before_height(&self, height: u32) -> Option<&Checkpoint> {
        // if we don't have any blocks yet, use the latest checkpoint that's at least a week older than earliest_key_time
        self.last_checkpoint_where(|checkpoint| checkpoint.height <= height)
    }

    pub fn last_checkpoint_on_or_before_timestamp(&self, timestamp: u32) -> Option<&Checkpoint> {
        // if we don't have any blocks yet, use the latest checkpoint that's at least a week older than earliest_key_time
        self.last_checkpoint_where(|checkpoint| checkpoint.timestamp <= timestamp)
    }


    pub fn should_request_merkle_blocks_for_next_sync_block_height(&mut self) -> bool {
        let block_height = self.last_sync_block_height() + 1;
        self.should_request_merkle_blocks_for_zone_after_height(block_height)
    }

    pub fn should_request_merkle_blocks_for_zone_after_height(&mut self, block_height: u32) -> bool {
        todo!()
        // let block_zone: u16 = (block_height / 500) as u16;
        // let left_over: u16 = (block_height % 500) as u16;
        // if self.chain_synchronization_fingerprint.is_some() {
        //     self.chain_synchronization_block_zones().contains(&block_zone) ||
        //         self.chain_synchronization_block_zones().contains(&(block_zone + 1)) ||
        //         self.chain_synchronization_block_zones().contains(&(block_zone + 2)) ||
        //         self.chain_synchronization_block_zones().contains(&(block_zone + 3)) ||
        //         (left_over == 0 && self.should_request_merkle_blocks_for_zone_after_height(((block_zone + 1) * 500) as u32))
        // } else {
        //     true
        // }
    }
    pub fn set_estimated_block_height(&mut self, height: u32, peer: &Peer, threshold_peer_count: usize) {
        todo!()
    }

    pub fn estimated_block_height(&mut self) -> u32 {
        if self.best_estimated_block_height > 0 {
            self.best_estimated_block_height
        } else {
            let h = self.decide_from_peer_soft_consensus_estimated_block_height();
            self.best_estimated_block_height = h;
            h
        }
    }

    pub fn decide_from_peer_soft_consensus_estimated_block_height(&self) -> u32 {
        let mut max_count = 0u32;
        let mut temp_best_estimated_block_height = 0u32;
        for (height, announcers) in &self.estimated_block_heights {
            let announcers_count = announcers.len() as u32;
            if announcers_count > max_count {
                temp_best_estimated_block_height = *height;
                max_count = announcers_count;
            } else if announcers_count == max_count && temp_best_estimated_block_height < *height {
                // use the latest if deadlocked
                temp_best_estimated_block_height = *height;
            }
        }
        temp_best_estimated_block_height
    }

    pub fn needs_initial_terminal_headers_sync(&mut self) -> bool {
        self.estimated_block_height() != self.last_terminal_block_height()
    }

    pub fn chain_sync_block_locator_array(&mut self) -> Vec<UInt256> {
        todo!()
        // if self.last_sync_block.is_some() && self.last_sync_block.unwrap().height() == 1 && self.is_devnet_any() {
        //     self.block_locator_array_for_block(self.last_sync_block)
        // } else if let Some(locators) = &self.last_persisted_chain_info.locators {
        //     locators.clone()
        // } else {
        //     let locators: Vec<UInt256> = self.block_locator_array_on_or_before_timestamp(BIP39_CREATION_TIME, false);
        //     self.last_persisted_chain_info.locators = Some(locators);
        //     locators.clone()
        // }
    }

    pub fn can_connect(&self) -> bool {
        !self.r#type().syncs_blockchain() || self.can_construct_a_filter()
    }

    pub fn has_a_wallet(&self) -> bool {
        !self.wallets.is_empty()
    }

    pub fn has_a_standalone_derivation_path(&self) -> bool {
        !self.viewing_account.fund_derivation_paths().is_empty()
    }

    pub fn can_construct_a_filter(&self) -> bool {
        println!("can_construct_a_filter: has_a_standalone_derivation_path: {} has_a_wallet: {}", self.has_a_standalone_derivation_path(), self.has_a_wallet());
        self.has_a_standalone_derivation_path() || self.has_a_wallet()
    }

    /// This is a time interval since 1970
    pub fn earliest_wallet_creation_time(&mut self) -> u64 {
        self.wallets.iter_mut()
            .map(|wallet| wallet.wallet_creation_time())
            .min_by(|t1, t2| t1.cmp(t2))
            .unwrap_or(BIP39_CREATION_TIME)
    }

    pub fn reset_chain_sync_start_height(&mut self) {
        let key = self.params.chain_type.chain_sync_start_height_key();
        if self.chain_sync_start_height == 0 {
            self.chain_sync_start_height = UserDefaults::uint_for_key::<u32>(key.as_str()).unwrap_or(0u32);
        }
        if self.chain_sync_start_height == 0 {
            self.chain_sync_start_height = self.last_sync_block_height();
            UserDefaults::set_num(key, self.chain_sync_start_height);
        }
    }

    pub fn restart_chain_sync_start_height(&mut self) {
        self.chain_sync_start_height = 0;
        UserDefaults::set_num(self.params.chain_type.chain_sync_start_height_key(), 0u32);
    }

    pub fn reset_terminal_sync_start_height(&mut self) {
        let key = self.params.chain_type.terminal_sync_start_height_key();
        if self.terminal_sync_start_height == 0 {
            self.terminal_sync_start_height = UserDefaults::uint_for_key(key.as_str()).unwrap_or(0);
        }
        if self.terminal_sync_start_height == 0 {
            self.terminal_sync_start_height = self.last_terminal_block_height();
            UserDefaults::set_num(key, self.terminal_sync_start_height);
        }
    }

    pub fn restart_chain_terminal_sync_start_height(&mut self) {
        self.terminal_sync_start_height = 0;
        UserDefaults::set_num(self.params.chain_type.terminal_sync_start_height_key(), 0u32);
    }

    pub fn has_spork_activated(&mut self, spork: &Spork) -> bool {
        spork.value <= self.last_terminal_block_height() as u64
    }

    pub fn should_request_merkle_blocks_for_zone_after_last_sync_height(&mut self) -> bool {
        !self.needs_initial_terminal_headers_sync() &&
                self.should_request_merkle_blocks_for_next_sync_block_height()
    }
    pub fn remove_estimated_block_heights_of_peer(&mut self, peer: &Peer) {
        // for (height, mut announcers) in self.estimated_block_heights {
        //     if let Some(pos) = announcers.iter().position(|x| x == peer) {
        //         announcers.remove(pos);
        //     }
        //     if announcers.is_empty() {
        //         self.estimated_block_heights.remove_entry(&height);
        //     }
        //     // keep best estimate if no other peers reporting on estimate
        //     if !self.estimated_block_heights.is_empty() && height == self.best_estimated_block_height.unwrap() {
        //         self.best_estimated_block_height = Some(0);
        //     }
        // }
    }

    pub fn reset_last_relayed_item_time(&self) {
        //self.last_chain_relay_time = 0;
    }

}
