use std::sync::{Arc, Weak};
use byte::{BytesExt, TryRead};
use crate::consensus::Encodable;
use crate::consensus::encode::VarInt;
use crate::crypto::{UInt256, UInt384, UInt768};
use crate::crypto::byte_util::Zeroable;
use crate::models::{LLMQEntry, MasternodeList};
use crate::chain::chain::Chain;
use crate::chain::common::chain_type::IHaveChainSettings;
use crate::chain::common::ChainType;
use crate::chain::network::message;
use crate::crypto::UTXO;
use crate::keys::{BLSKey, IKey};
use crate::util::Shared;

#[derive(Clone)]
pub struct ReadContext {
    pub chain_type: ChainType,
    pub chain: Shared<Chain>,
    pub deterministic: bool,
}

#[derive(Clone, Debug, Default)]
pub struct InstantSendLock {
    pub version: u8,
    pub transaction_hash: UInt256,
    pub signature: UInt768,
    pub input_outpoints: Vec<UTXO>,
    pub cycle_hash: UInt256,
    pub quorum_verified: bool,
    pub signature_verified: bool,
    // verifies the signature and quorum together
    pub intended_quorum: Option<Weak<LLMQEntry>>,
    pub saved: bool,
    pub deterministic: bool,
    pub request_id: UInt256,

    pub chain: Shared<Chain>,
    pub chain_type: ChainType,

}

// impl InstantSendLock {
//     pub fn from_entity(entity: InstantSendLockEntity, tx_hash: UInt256, transaction_inputs: &Vec<TransactionInput>, chain: &Chain) -> Self {
//         Self {
//             chain,
//             transaction_hash: tx_hash,
//             signature: entity.signature,
//             input_outpoints: transaction_inputs.iter().map(|input| input.outpoint()).collect(),
//             quorum_verified: true,
//             signature_verified: true,
//             saved: true,
//             // todo impl deterministic flag in db?
//             intended_quorum: None,
//             deterministic: false,
//             version: 0,
//             cycle_hash: Default::default(),
//             request_id: Default::default()
//         }
//     }
// }

impl<'a> TryRead<'a, ReadContext> for InstantSendLock {
    // transaction hash (32)
    // transaction outpoint (36)
    // masternode outpoint (36)
    // if spork 15 is active
    //  quorum hash 32
    //  confirmed hash 32
    // masternode signature
    // size - varint
    // signature 65 or 96 depending on spork 15
    fn try_read(bytes: &'a [u8], context: ReadContext) -> byte::Result<(Self, usize)> {
        // if !chain.spork_manager().deterministic_masternode_list_enabled || !chain.spork_manager().llmq_instant_send_enabled {
        //     return None;
        // }
        let deterministic = context.deterministic;
        let offset = &mut 0;
        let version = if deterministic {
            bytes.read_with::<u8>(offset, byte::LE)?
        } else {
            0
        };
        let count = bytes.read_with::<VarInt>(offset, byte::LE)?;
        let mut input_outpoints = Vec::<UTXO>::new();
        (0..count.0).for_each(|i| {
            let utxo = bytes.read_with::<UTXO>(offset, byte::LE).unwrap();
            input_outpoints.push(utxo);
        });
        let transaction_hash = bytes.read_with::<UInt256>(offset, byte::LE)?;
        let cycle_hash = if deterministic {
            bytes.read_with::<UInt256>(offset, byte::LE)?
        } else {
            UInt256::MIN
        };
        let signature = bytes.read_with::<UInt768>(offset, byte::LE)?;
        let lock = Self {
            version,
            transaction_hash,
            signature,
            input_outpoints,
            cycle_hash,
            chain_type: context.chain_type,
            chain: context.chain,
            quorum_verified: false,
            signature_verified: false,
            intended_quorum: None,
            saved: false,
            deterministic,
            request_id: UInt256::MIN
        };
        Ok((lock, *offset))
    }
}


impl InstantSendLock {
    pub fn to_data(&self) -> Vec<u8> {
        let mut writer: Vec<u8> = Vec::new();
        VarInt(self.input_outpoints.len() as u64).enc(&mut writer);
        self.input_outpoints.iter().for_each(|utxo| {
            utxo.enc(&mut writer);
        });
        self.transaction_hash.enc(&mut writer);
        self.signature.enc(&mut writer);
        writer
    }

    pub fn request_id(&mut self) -> UInt256 {
        if !self.request_id.is_zero() { return self.request_id }
        let mut writer: Vec<u8> = Vec::new();
        // TODO: shall we write isdlock here?
        let is_lock_type: String = message::MessageType::Islock.into();
        is_lock_type.enc(&mut writer);
        VarInt(self.input_outpoints.len() as u64).enc(&mut writer);
        self.input_outpoints.iter().for_each(|utxo| {
            utxo.enc(&mut writer);
        });
        let req_id = UInt256::sha256d(writer);
        self.request_id = req_id;
        println!("the request ID is {}", req_id);
        req_id
    }

    pub fn sign_id_for_quorum_entry(&mut self, llmq_hash: UInt256) -> UInt256 {
        let mut writer = Vec::<u8>::new();
        // todo: check is vs isd type locks
        let var_int: VarInt = self.chain_type.is_llmq_type().into();
        var_int.enc(&mut writer);
        llmq_hash.enc(&mut writer);
        self.request_id().enc(&mut writer);
        self.transaction_hash.enc(&mut writer);
        UInt256::sha256d(writer)
    }

    pub fn find_signing_quorum_and_masternode_list(&mut self) -> Option<(&LLMQEntry, &MasternodeList)> {
        // todo: check is vs isd type locks
        todo!()
        // let r#type = self.chain_type.is_llmq_type();
        /*let mut result: Option<(&LLMQEntry, &MasternodeList)> = None;
        'outer: for list in self.chain.masternode_manager().recent_masternode_lists() {
            for quorum in list.quorums_of_type(r#type) {
                if self.verify_signature_against_quorum(&quorum) {
                    result = Some((&quorum, &list));
                    break 'outer;
                }
            }
        }
        result*/
        /*self.chain.masternode_manager().recent_masternode_lists().iter()
            .find_map(|list|
                list.quorums_of_type(r#type).iter()
                    .find_map(|&quorum|
                        self.verify_signature_against_quorum(quorum)
                            .then(|| (quorum, list))))*/
    }

    fn verify_signature_against_quorum(&mut self, public_key: UInt384, llmq_hash: UInt256, use_legacy_bls_scheme: bool) -> bool {
        // todo: check use_legacy_bls has taken from appropriate place
        let mut key = BLSKey::key_with_public_key(public_key, use_legacy_bls_scheme);
        let sign_id = self.sign_id_for_quorum_entry(llmq_hash);
        key.verify(&sign_id.0.to_vec(), &self.signature.0.to_vec())
    }

    pub fn verify_signature_with_quorum_offset(&mut self, offset: u32) -> bool {
        let request_id = self.request_id();
        let quorum = self.chain.with(|chain| chain.masternode_manager.quorum_entry_for_instant_send_request_id(&request_id, offset));
        match quorum {
            Some(quorum) => {
                match quorum.upgrade() {
                    Some(quorum) => {
                        if quorum.verified {
                            self.signature_verified = self.verify_signature_against_quorum(quorum.public_key, quorum.llmq_hash, quorum.version.use_bls_legacy());
                            println!("verifying IS signature with offset {}: {}", offset, self.signature_verified);
                            if self.signature_verified {
                                self.intended_quorum = Some(Arc::downgrade(&quorum));
                            } else if offset == 8 {
                                // try again a few blocks more in the past
                                println!("trying with offset 0");
                                return self.verify_signature_with_quorum_offset(0);
                            }
                        } else {
                            println!("llmq entry ({}) found but is not yet verified", quorum.llmq_hash);
                        }
                        println!("returning signature verified {} with offset {}", self.signature_verified, offset);
                        self.signature_verified
                    },
                    None => false
                }
            },
            None => {
                println!("no quorum entry found");
                false
            }
        }
    }

    pub fn verify_signature(&mut self) -> bool {
        // TODO: check (has taken from production code where verified always 'true')
        self.verify_signature_with_quorum_offset(8)
    }


    pub fn save_initial(&self) {
        if self.saved {
            return;
        }
        /*self.chain.chain_context().perform_block_and_wait(|context| {
            InstantSendLockEntity::create_if_need(self, context)
                .expect("Can't create instant send lock entity");
        });*/
    }

    pub fn save_signature_valid(&self) {
        if !self.saved {
            self.save_initial();
            return;
        }
        // saving here will only create, not update.
        /*self.chain.chain_context().perform_block_and_wait(|context| {
            InstantSendLockEntity::update_signature_validity_for_lock_with_tx_hash(true, &self.transaction_hash, context)
                .expect("Can't save insant send lock entity");
        });*/
    }

}
