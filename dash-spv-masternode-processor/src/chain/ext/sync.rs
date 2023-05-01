use crate::chain::{Chain, SyncCountInfo};
use crate::storage::manager::managed_context::ManagedContext;

pub trait SyncProgress {
    fn assign_sync_weights(&mut self);
    fn chain_blocks_to_sync(&mut self) -> u32;
    fn chain_sync_progress(&mut self) -> f64;
    fn terminal_headers_to_sync(&mut self) -> u32;
    fn terminal_header_sync_progress(&mut self) -> f64;
    fn combined_sync_progress(&mut self) -> f64;
    fn start_sync(&mut self);
    fn stop_sync(&mut self);
    fn remove_non_mainnet_trusted_peer(&mut self);
    fn reset_sync_count_info(&mut self, sync_count_info: SyncCountInfo, context: &ManagedContext) {
        self.set_count(0, sync_count_info, context)
    }

    fn set_count(&mut self, count: u32, sync_count_info: SyncCountInfo, context: &ManagedContext);
}

impl SyncProgress for Chain {

    fn assign_sync_weights(&mut self) {
        /*let chain_blocks: u32 = self.chain_blocks_to_sync();
        let terminal_blocks: u32 = self.terminal_headers_to_sync();
        let masternode_lists_to_sync: u32 = self.masternode_manager().estimated_masternode_lists_to_sync();
        // a unit of weight is the time it would take to sync 1000 blocks;
        // terminal headers are 4 times faster the blocks
        // the first masternode list is worth 20000 blocks
        // each masternode list after that is worth 2000 blocks
        let chain_weight: u32 = chain_blocks;
        let terminal_weight = terminal_blocks / 4;

        let masternode_weight = if masternode_lists_to_sync > 0 { 20000 + 2000 * (masternode_lists_to_sync - 1) } else { 0 };
        let total_weight = (chain_weight + terminal_weight + masternode_weight) as f64;
        if total_weight == 0.0 {
            self.terminal_header_sync_weight = 0f64;
            self.masternode_list_sync_weight = 0f64;
            self.chain_sync_weight = 1f64;
        } else {
            self.chain_sync_weight = chain_weight as f64 / total_weight;
            self.terminal_header_sync_weight = terminal_weight as f64 / total_weight;
            self.masternode_list_sync_weight = masternode_weight as f64 / total_weight;
        }
        */
    }

    fn chain_blocks_to_sync(&mut self) -> u32 {
        0
        /*if self.last_sync_block_height() >= self.estimated_block_height() {
            0
        } else {
            self.estimated_block_height() - self.last_sync_block_height()
        }*/
    }

    fn chain_sync_progress(&mut self) -> f64 {
        0f64
        /*if self.peer_manager().download_peer.is_none() && self.chain_sync_start_height == 0 {
            return 0f64;
        }
        if self.last_sync_block_height() >= self.estimated_block_height() {
            return 1f64;
        }
        let last_block_height = self.last_sync_block_height();
        let estimated_block_height = self.estimated_block_height() as f64;
        let sync_start_height = self.chain_sync_start_height;
        if estimated_block_height == 0f64 {
            return 0f64;
        }
        if sync_start_height > last_block_height {
            1f64.min(0f64.max(0.1 + 0.9 * last_block_height as f64 / estimated_block_height))
        } else if estimated_block_height as u32 - sync_start_height == 0 {
            0f64
        } else {
            1f64.min(0f64.max(0.1 + 0.9 * (last_block_height as f64 - sync_start_height as f64) / (estimated_block_height - sync_start_height as f64)))
        }*/
    }

    fn terminal_headers_to_sync(&mut self) -> u32 {
        0
        //if self.last_terminal_block_height() >= self.estimated_block_height() { 0 } else { self.estimated_block_height() - self.last_terminal_block_height() }
    }

    fn terminal_header_sync_progress(&mut self) -> f64 {
        0f64
        /*if self.peer_manager().download_peer.is_none() && self.terminal_sync_start_height == 0 {
            0f64
        } else if self.last_terminal_block_height() >= self.estimated_block_height() {
            1f64
        } else {
            let mut last_block_height = self.last_terminal_block_height() as f64;
            let estimated_block_height = self.estimated_block_height() as f64;
            let sync_start_height = self.terminal_sync_start_height as f64;
            1f64.min(0f64.max(0.1 + 0.9 * if sync_start_height > last_block_height {
                last_block_height / estimated_block_height
            } else {
                (last_block_height - sync_start_height) / (estimated_block_height - sync_start_height)
            }))
        }*/
    }

    fn combined_sync_progress(&mut self) -> f64 {
        0f64
        /*if (self.terminal_header_sync_weight + self.chain_sync_weight + self.masternode_list_sync_weight) == 0.0 {
            if self.peer_manager().connected { 1f64 } else { 0f64 }
        } else {
            let progress = self.terminal_header_sync_progress() * self.terminal_header_sync_weight + self.masternode_manager().masternode_list_and_quorums_sync_progress() * self.masternode_list_sync_weight + self.chain_sync_progress() * self.chain_sync_weight;
            if progress < 0.99995 {
                progress
            } else {
                1f64
            }
        }*/
    }

    /// Blockchain Sync

    fn start_sync(&mut self) {
        // dispatch_async(dispatch_get_main_queue(), ^{
        //     [[NSNotificationCenter defaultCenter] postNotificationName:DSChainManagerSyncWillStartNotification
        //     object:nil
        //     userInfo:@{DSChainManagerNotificationChainKey: self.chain}];
        // });
        if self.can_connect() {
            if self.terminal_header_sync_progress() < 1.0 {
                self.reset_terminal_sync_start_height();
            }
            if self.chain_sync_progress() < 1.0 {
                self.reset_chain_sync_start_height();
            }
            let earliest_wallet_creation_time = self.earliest_wallet_creation_time();
            self.peer_manager.connect(earliest_wallet_creation_time);
        }
    }

    fn stop_sync(&mut self) {
        /*self.peer_manager().disconnect();
        self.sync_phase = SyncPhase::Offline;*/
    }

    fn remove_non_mainnet_trusted_peer(&mut self) {
        /*if !self.is_mainnet() {
            self.stop_sync();
            self.peer_manager().remove_trusted_peer_host();
            self.peer_manager().clear_peers();
            /*match PeerEntity::delete_for_chain_type::<crate::schema::peers::dsl::peers>(self.r#type(), self.chain_context()) {
                Ok(deleted) => println!("All peer entities for chain {:?} are deleted", self.r#type()),
                Err(err) => println!("Error deleting peer entities: {}", err)
            }*/
        }*/
    }

    fn set_count(&mut self, count: u32, sync_count_info: SyncCountInfo, context: &ManagedContext) {
        /*match sync_count_info {
            SyncCountInfo::GovernanceObject => {
                self.total_governance_objects_count = count;
                self.save_in_context(context);
            },
            SyncCountInfo::GovernanceObjectVote => {
                if let Some(mut obj) = &self.governance_sync_manager().current_governance_sync_object {
                    obj.total_governance_vote_count = count;
                    obj.save();
                }
            },
            _ => {}
        }*/
    }

}
