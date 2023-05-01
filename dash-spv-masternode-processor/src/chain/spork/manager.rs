use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::time::SystemTime;
use crate::crypto::UInt256;
use crate::chain::Chain;
use crate::chain::common::ChainType;
use crate::chain::spork::{Identifier, Spork};
use crate::chain::network::Peer;
use crate::default_shared;
use crate::storage::manager::managed_context::ManagedContext;
use crate::util::{Shared, TimeUtil};
use crate::util::timer::Timer;

pub const SPORK_15_MIN_PROTOCOL_VERSION: u32 = 70213;

pub trait PeerSporkDelegate: Send + Sync + Debug {
    fn peer_relayed_spork(&self, peer: &mut Peer, spork: Spork);
    fn peer_has_spork_hashes(&self, peer: &Peer, hashes: Vec<UInt256>);
}

#[derive(Clone, Default)]
pub struct Manager {
    /// this is the time after a successful spork sync, this is not persisted between sessions
    pub last_requested_sporks: u64,
    /// this is the time after a successful spork sync, this is not persisted between sessions
    pub last_synced_sporks: u64,
    /// spork #2
    pub instant_send_active: bool,
    /// spork #15
    pub deterministic_masternode_list_enabled: bool,
    /// spork #17
    pub quorum_dkg_enabled: bool,
    /// spork #19
    pub chain_locks_enabled: bool,
    /// spork #20
    pub llmq_instant_send_enabled: bool,

    pub spork_dictionary: HashMap<Identifier, Spork>,
    pub chain_type: ChainType,
    pub chain: Shared<Chain>,
    // pub context: &'static ManagedContext,

    spork_hashes_marked_for_retrieval: Vec<UInt256>,
    spork_timer: Option<Timer>,
}

impl Manager {
    pub fn update_with_spork(&mut self, spork: Spork) -> bool {
        self.last_synced_sporks = SystemTime::seconds_since_1970();
        let identifier = &spork.identifier;
        let current_spork: Option<Spork> = self.spork_dictionary.get(identifier).cloned();
        let mut updated_spork: Option<Spork> = None;
        self.check_triggers_for_spork(&spork);
        if let Some(ref old_spork) = current_spork {
            if !spork.eq(old_spork) {
                self.spork_dictionary.insert(identifier.clone(), spork.clone());
                updated_spork = current_spork;
            } else {
                return false;
            }
        } else {
            self.spork_dictionary.insert(identifier.clone(), spork.clone());
        }
        if /*current_spork.as_ref().is_none() ||*/ updated_spork.is_some() {
            todo!("save and notify")
        }
        true
    }

    pub fn update_with_spork_hashes(&mut self, peer: &Peer, hashes: Vec<UInt256>) {
        let new_sporks = hashes.iter()
            .filter_map(|hash| (!self.spork_hashes_marked_for_retrieval.contains(&hash))
                .then_some(hash)).collect::<Vec<_>>();
        let need_request = !new_sporks.is_empty();
        self.spork_hashes_marked_for_retrieval.extend(new_sporks);
        if need_request {
            self.get_sporks();
        }
    }
}

default_shared!(Manager);

// impl<'a> Default for &'a Manager {
//     fn default() -> Self {
//         &Manager::default()
//     }
// }

impl Debug for Manager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.spork_dictionary.fmt(f)
    }
}

impl Manager {

    pub fn new(chain_type: ChainType) -> Self {
        let s = Self { chain_type, ..Default::default() };
        //s.load_in_context(&s.context);
        // s.check_triggers();
        s
    }

    // pub fn new(chain_type: ChainType, chain: Shared<Chain>) -> Self {
    //     let s = Self { chain_type, chain/*, context: chain.chain_context()*/, ..Default::default() };
    //     //s.load_in_context(&s.context);
    //     s
    // }

    fn load_in_context(&mut self, context: &ManagedContext) {
        // todo: check this out
        /*self.context.perform_block_and_wait(|context| {
            match SporkEntity::get_all_for_chain_type(self.chain.r#type(), context) {
                Ok(entities) => {
                    let (spork_dictionary, spork_hashes_marked_for_retrieval) = entities.iter().fold((HashMap::new(), Vec::new()), |(mut dict, mut hashes), entity| {
                        if entity.marked_for_retrieval > 0 {
                            hashes.push(&entity.spork_hash);
                        } else {
                            dict.insert(Identifier::from(entity.identifier), Spork::from_entity(&entity, self.chain));
                        }
                        (dict, hashes)
                    });
                    self.spork_dictionary = spork_dictionary;
                    self.spork_hashes_marked_for_retrieval = spork_hashes_marked_for_retrieval;
                    self.check_triggers();
                },
                Err(err) => println!("Error retrieving sporks for chain {:?}", self.chain.r#type())
            }
        });*/
    }
}

impl Manager {
    pub fn instant_send_active(&mut self) -> bool {
        self.spork_dictionary
            .get(&Identifier::Spork2InstantSendEnabled)
            .map_or(true,|spork| self.chain.with(|chain| chain.has_spork_activated(spork)))
    }

    pub fn sporks_updated_signatures(&mut self) -> bool {
        self.spork_dictionary
            .get(&Identifier::Spork6NewSigs)
            .map_or(false, |spork| self.chain.with(|chain| chain.has_spork_activated(spork)))
    }

    pub fn deterministic_masternode_list_enabled(&mut self) -> bool {
        self.spork_dictionary
            .get(&Identifier::Spork15DeterministicMasternodesEnabled)
            .map_or(true, |spork| self.chain.with(|chain| chain.has_spork_activated(spork)))
    }

    pub fn llmq_instant_send_enabled(&mut self) -> bool {
        self.spork_dictionary
            .get(&Identifier::Spork20InstantSendLLMQBased)
            .map_or(true, |spork| self.chain.with(|chain| chain.has_spork_activated(spork)))
    }

    pub fn quorum_dkg_enabled(&mut self) -> bool {
        self.spork_dictionary
            .get(&Identifier::Spork17QuorumDKGEnabled)
            .map_or(true, |spork| self.chain.with(|chain| chain.has_spork_activated(spork)))
    }

    pub fn chain_locks_enabled(&mut self) -> bool {
        self.spork_dictionary
            .get(&Identifier::Spork19ChainLocksEnabled)
            .map_or(true, |spork| self.chain.with(|chain| chain.has_spork_activated(spork)))
    }
}

impl Manager {

    /// Spork Sync

    pub fn perform_spork_request(&mut self) {
        // after syncing, get sporks from other peers
        todo!()
        /*self.chain.with(|chain| chain.peer_manager.connected_peers.iter().for_each(|mut p| {
            if p.status == PeerStatus::Connected {
                // let CALLED_BACK: AtomicU64 = AtomicU64::new(self.last_requested_sporks);
                let callback = |success| {
                    if success {
                        self.last_requested_sporks = SystemTime::seconds_since_1970();
                        // CALLED_BACK.store(SystemTime::seconds_since_1970(), Ordering::SeqCst);
                        p.send_get_sporks();
                    }
                };

                p.send_ping_message(Arc::new(callback));
            }
        }))*/
    }


    pub fn get_sporks(&mut self) {
        // if !self.chain_type.sync_type().contains(SyncType::Sporks)) {
        //     // make sure we care about sporks
        //     return;
        // } else if self.spork_timer.is_none() {
        //     // let callback = std::sync::
        //     // let s = Shared::Borrowed(self);
        //     let timer = Timer::new(|| {
        //         // wait 10 minutes between requests
        //         if self.last_synced_sporks < SystemTime::ten_minutes_ago_1970() {
        //             self.perform_spork_request();
        //         }
        //     });
        //     self.spork_timer = Some(timer);
        //     timer.schedule(Duration::ZERO, Duration::from_secs(600));
        // }
    }

    pub fn stop_getting_sporks(&mut self) {
        self.spork_timer.as_mut().map(|timer| timer.stop());
    }

    pub fn check_triggers(&mut self) {
        todo!()
        // self.spork_dictionary.values()
        //     .for_each(|spork| self.check_triggers_for_spork(spork))
        //
        // let self_ref = &mut *self;
        // // self.spork_dictionary.values().for_each(|spork| self_ref.check_triggers_for_spork(spork));
        // for spork in self.spork_dictionary.values_mut() {
        //     self.check_triggers_for_spork(spork);
        // }

    }

    pub fn check_triggers_for_spork(&self, spork: &Spork) {
        // let mut changed = false;
        // let identifier = &spork.identifier;
        // let changed = !self.spork_dictionary.contains_key(identifier) || self.spork_dictionary.get(identifier).unwrap().value != spork.value;

        // if Identifi/**/er::Spork15DeterministicMasternodesEnabled.eq(identifier) {
            /*if self.chain_type.is_devnet_any() && self.chain.estimated_block_height() as u64 >= spork.value && self.chain_type.min_protocol_version() < SPORK_15_MIN_PROTOCOL_VERSION {
                //use estimated block height here instead
                self.chain.set_min_protocol_version(SPORK_15_MIN_PROTOCOL_VERSION);
            }*/
        // }
        // todo: ?? unused var
    }

    pub fn set_spork_value(&mut self, spork: Spork) {
        self.check_triggers_for_spork(&spork);
        self.spork_dictionary.insert(spork.identifier.clone(), spork.clone());
    }

    pub fn wipe_spork_info(&mut self) {
        self.spork_dictionary.clear();
        self.stop_getting_sporks();
    }

}
