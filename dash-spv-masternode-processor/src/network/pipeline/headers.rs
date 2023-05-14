//
// Copyright 2018-2019 Tamas Blummer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
//!
//! # Download headers
//!
use log::{debug, error};
use std::{
    time::Duration,
};
use crate::chain::Chain;
use crate::chain::common::ChainType;
use crate::chain::ext::Settings;
use crate::chain::network::{InvType, Request};
use crate::chain::network::message::inventory::Inventory;
use crate::chain::network::message::message::{Message, MessagePayload};
use crate::chain::network::message::response::Response;
use crate::crypto::UInt256;
use crate::manager::peer_manager;
use crate::manager::peer_manager::SERVICES_NODE_NETWORK;
use crate::network::p2p::{ExpectedReply, P2PControlDispatcher, P2PNotification, P2PNotificationDispatcher, P2PNotificationReceiver, PeerId, SharedTimeout};
use crate::network::pipeline::{Pipeline, ThreadName};
use crate::util::Shared;

pub struct Headers {
    dispatcher: P2PControlDispatcher,
    chain: Shared<Chain>,
    timeout: SharedTimeout<ExpectedReply>,
}

impl Pipeline for Headers {

    fn setup(self, back_pressure: usize, chain_type: ChainType) -> P2PNotificationDispatcher {
        self.setup_with(ThreadName::PipelineHeaders, back_pressure, chain_type)
    }

    fn run(&mut self, receiver: P2PNotificationReceiver) {
        loop {
            while let Ok(msg) = receiver.recv_timeout(Duration::from_millis(1000)) {
                if let Err(e) = match msg {
                    P2PNotification::Connected(pid, _) if self.is_serving_blocks(pid) => self.get_headers(pid),
                    P2PNotification::Incoming(pid, msg) => {
                        match msg {
                            Response::Inventory(ref inv) if self.is_serving_blocks(pid) => self.inv(inv, pid),
                            Response::Headers(ref headers, _, _) if self.is_serving_blocks(pid) => self.headers(headers, pid),
                            Response::Ping(_) | _ => Ok(())
                        }
                    },
                    _ => Ok(())
                } {
                    error!("Error processing headers: {}", e);
                }
            }
            self.timeout.lock().unwrap().check(vec!(ExpectedReply::Headers));
        }
    }
}

impl Headers {
    pub fn new(chain: Shared<Chain>, chain_type: ChainType, dispatcher: P2PControlDispatcher, timeout: SharedTimeout<ExpectedReply>) -> P2PNotificationDispatcher {
        let back_pressure = dispatcher.back_pressure;
        let pipeline = Self { chain, dispatcher: dispatcher, timeout };
        pipeline.setup(back_pressure, chain_type)
    }

    fn is_serving_blocks(&self, peer: PeerId) -> bool {
        self.dispatcher.peer_version(peer).map_or(false, |v| v.services & SERVICES_NODE_NETWORK != 0)
    }

    // process an incoming inventory announcement
    fn inv(&mut self, v: &Inventory, peer: PeerId) -> Result<(), peer_manager::Error> {
        let mut ask_for_headers = false;
        for (inventory, hashes) in &v.map {
            // only care for blocks
            if *inventory == InvType::Block {
                self.chain.with(|chain| {
                    hashes.iter().for_each(|hash| {
                        if let Some(block) = chain.block_for_block_hash(hash) {
                            debug!("received inv for new block {:?} peer={}", block, peer);
                            ask_for_headers = true;
                        }
                    });
                });
            }
        }
        if ask_for_headers {
            self.get_headers(peer)?;
        }
        Ok(())
    }

    /// get headers this peer is ahead of us
    fn get_headers(&mut self, peer: PeerId) -> Result<(), peer_manager::Error> {
        if !self.timeout.lock().unwrap().is_busy_with(peer, ExpectedReply::Headers) {
            self.chain.with(|chain| {
                let locators = chain.block_locator_array_for_block(None);
                if locators.len() > 0 {
                    let first = *locators.first().unwrap();
                    self.timeout.lock().unwrap().expect(peer, 1, ExpectedReply::Headers);
                    let request = Request::GetHeaders(locators, first, chain.r#type().protocol_version());
                    let message = Message::from(MessagePayload::Request(request));
                    self.dispatcher.send_network(peer, message);
                }
            });
        }
        Ok(())
    }

    fn headers(&mut self, headers: &Vec<UInt256>, peer: PeerId) -> Result<(), peer_manager::Error> {
        self.timeout.lock().unwrap().received(peer, 1, ExpectedReply::Headers);
        println!("headers: {:?}", headers);
        // if headers.len() > 0 {
        //     // current height
        //     let mut height;
        //     // some received headers were not yet known
        //     let mut some_new = false;
        //     let mut moved_tip = None;
        //     {
        //         height = self.chain.with(|chain| chain.last_terminal_block_height());
        //         if height == 0 {
        //             return Err(peer_manager::Error::NoTip);
        //         }
        //     }
        //
        //     let mut headers_queue = VecDeque::new();
        //     headers_queue.extend(headers.iter());
        //     while !headers_queue.is_empty() {
        //         let mut disconnected_headers = Vec::new();
        //         let mut connected_headers = Vec::new();
        //         {
        //             self.chain.with(|chain| {
        //                 while let Some(header) = headers_queue.pop_front() {
        //
        //                 }
        //                 chain.add_header()
        //             });
        //
        //             let mut chaindb = self.chaindb.write().unwrap();
        //             while let Some(header) = headers_queue.pop_front() {
        //                 // add to blockchain - this also checks proof of work
        //                 match chaindb.add_header(&header) {
        //                     Ok(Some((stored, unwinds, forwards))) => {
        //                         connected_headers.push((stored.header.clone(), stored.height));
        //                         // POW is ok, stored top chaindb
        //                         some_new = true;
        //
        //                         if let Some(forwards) = forwards {
        //                             moved_tip = Some(forwards.last().unwrap().clone());
        //                         }
        //                         height = stored.height;
        //
        //                         if let Some(unwinds) = unwinds {
        //                             disconnected_headers.extend(unwinds.iter()
        //                                 .map(|h| chaindb.get_header(h).unwrap().stored.header));
        //                             break;
        //                         }
        //                     }
        //                     Ok(None) => {}
        //                     Err(Error::SpvBadProofOfWork) => {
        //                         info!("Incorrect POW, banning peer={}", peer);
        //                         self.p2p.ban(peer, 100);
        //                         return Ok(());
        //                     }
        //                     Err(e) => {
        //                         debug!("error {} processing header {} ", e, header.bitcoin_hash());
        //                         return Ok(());
        //                     }
        //                 }
        //             }
        //             chaindb.batch()?;
        //         }
        //     }
        //
        //     if some_new {
        //         // ask if peer knows even more
        //         self.get_headers(peer)?;
        //     }
        //
        //     if let Some(new_tip) = moved_tip {
        //         info!("received {} headers new tip={} from peer={}", headers.len(), new_tip, peer);
        //         self.p2p.send(P2PControlNotification::Height(height));
        //     } else {
        //         debug!("received {} known or orphan headers [{} .. {}] from peer={}", headers.len(), headers[0].bitcoin_hash(), headers[headers.len()-1].bitcoin_hash(), peer);
        //     }
        // }
        Ok(())
    }
}
