use byte::{BytesExt, TryRead};
use crate::chain::{Chain, common::{ChainType, IHaveChainSettings}};
use crate::consensus::{Encodable, encode::VarInt};
use crate::crypto::{UInt256, UInt384, UInt768};
use crate::keys::BLSKey;
use crate::models::{LLMQEntry, MasternodeList};
use crate::util::Shared;

#[derive(Clone, Debug, Default)]
pub struct ChainLock {
    pub height: u32,
    pub block_hash: UInt256,
    pub request_id: Option<UInt256>,
    pub signature: UInt768,
    pub signature_verified: bool,
    pub quorum_verified: bool,
    pub saved: bool,
    pub intended_quorum: Option<Shared<LLMQEntry>>,
    pub chain_type: ChainType,
    pub chain: Shared<Chain>,
    // pub input_outpoints: Vec<>
}

#[derive(Clone)]
pub struct ReadContext(pub ChainType, pub Shared<Chain>);

impl<'a> TryRead<'a, ReadContext> for ChainLock {
    fn try_read(bytes: &'a [u8], context: ReadContext) -> byte::Result<(Self, usize)> {
        if bytes.len() < 132 {
            return Err(byte::Error::Incomplete);
        }
        let offset = &mut 0;
        let height = bytes.read_with::<u32>(offset, byte::LE)?;
        let block_hash = bytes.read_with::<UInt256>(offset, byte::LE)?;
        let signature = bytes.read_with::<UInt768>(offset, byte::LE)?;
        println!("the chain lock signature received for height {} (sig {}) (blockhash {})", height, signature, block_hash);
        Ok((Self {
            height,
            block_hash,
            signature,
            chain_type: context.0,
            chain: context.1,
            ..Default::default()
        }, *offset))
    }
}

impl ChainLock {
    pub fn new(block_hash: UInt256, signature: UInt768, signature_verified: bool, quorum_verified: bool, chain: Shared<Chain>) -> Self {
        Self {
            block_hash,
            signature,
            signature_verified,
            quorum_verified,
            chain,
            saved: true, // this is coming already from the persistant store and not from the network
            ..Default::default()
        }
    }

    fn calculate_request_id(&self) -> UInt256 {
        let mut writer: Vec<u8> = Vec::new();
        "clsig".to_string().enc(&mut writer);
        self.height.enc(&mut writer);
        let req_id = UInt256::sha256d(writer);
        req_id
    }

    pub fn get_request_id(&self) -> UInt256 {
        self.request_id.unwrap_or(self.calculate_request_id())
    }

    pub fn get_request_id_mut(&mut self) -> UInt256 {
        self.request_id.unwrap_or({
            let req_id = self.calculate_request_id();
            self.request_id = Some(req_id);
            req_id
        })
    }

    pub fn sign_id_for_quorum_entry(&self, llmq_hash: UInt256) -> UInt256 {
        let mut buffer: Vec<u8> = Vec::new();
        let chain_locks_type = VarInt(u8::from(self.chain_type.chain_locks_type()) as u64);
        chain_locks_type.enc(&mut buffer);
        llmq_hash.enc(&mut buffer);
        self.get_request_id().enc(&mut buffer);
        self.block_hash.enc(&mut buffer);
        UInt256::sha256d(buffer)
    }

    pub fn verify_signature_against_quorum(&mut self, public_key: UInt384, llmq_hash: UInt256, use_legacy_bls_scheme: bool) -> bool {
        // let public_key = entry.public_key;
        // let use_legacy = entry.version.use_bls_legacy();
        let sign_id = self.sign_id_for_quorum_entry(llmq_hash);
        println!("verifying signature <REDACTED> with public key <REDACTED> for transaction hash <REDACTED> against quorum {:?}", llmq_hash);
        BLSKey::verify_with_public_key(sign_id, self.signature, public_key, use_legacy_bls_scheme)
    }

    pub fn verify_signature_with_quorum_offset(&mut self, offset: u32) -> bool {
        /*if let Some(quorum) = self.chain.masternode_manager().quorum_entry_for_chain_lock_request_id(self.request_id, self.height - offset) {
            if quorum.verified {
                self.signature_verified = self.verify_signature_against_quorum(quorum);
            }
            if self.signature_verified {
                self.intended_quorum = Some(quorum);
                // We should also set the chain's last chain lock
                self.chain.update_last_chain_lock_if_need(self);
            } else if quorum.verified && offset == 8 {
                return self.verify_signature_with_quorum_offset(0);
            } else if quorum.verified && offset == 0 {
                return self.verify_signature_with_quorum_offset(16);
            }
            println!("returning chain lock signature verified {} with offset {}", self.signature_verified, offset);
        }*/
        self.signature_verified
    }

    pub fn verify_signature(&mut self) -> bool {
        self.verify_signature_with_quorum_offset(8)
    }

    pub fn find_signing_quorum_return_masternode_list(&mut self) -> (Option<&LLMQEntry>, Option<&MasternodeList>) {
        let llmq_type = self.chain_type.chain_locks_type();
        //let recent_masternode_lists = self.chain.masternode_manager().recent_masternode_lists();
        let quorum: Option<&LLMQEntry> = None;
        let list: Option<&MasternodeList> = None;
        /*for masternode_list in recent_masternode_lists {
            if let Some(quorums) = masternode_list.quorums.get(&llmq_type) {
                for (_, entry) in quorums {
                    let signature_verified = self.verify_signature_against_quorum(entry);
                    if signature_verified {
                        quorum = Some(&entry);
                        list = Some(&masternode_list);
                        break;
                    }
                }
            }
            if quorum.is_some() {
                break;
            }
        }*/
        (quorum, list)
    }

    pub fn save_initial(&mut self) {
        if self.saved {
            return;
        }
        // TODO: saving here will only create, not update
        /*self.chain.chain_context().perform_block_and_wait(|context| {
            if let Err(err) = ChainLockEntity::create_if_need(self, context) {
                println!("ChainLock saving error: {}", err);
            } else {
                self.saved = true;
            }
        });*/
    }

    pub fn save_signature_valid(&mut self) {
        if !self.saved {
            self.save_initial();
            return;
        }
        /*self.chain.chain_context().perform_block_and_wait(|context| {
            ChainLockEntity::update_signature_valid_if_need(self, context)
                .expect("Can't update signature for chain lock entity");
        });*/
    }

}
