use byte::ctx::Bytes;
use byte::{BytesExt, TryRead};
use crate::chain::chain::Chain;
use crate::chain::common::{ChainType, LLMQType};
use crate::chain::tx::{InstantSendLock, ITransaction, Transaction, TransactionInput, TransactionOutput, TransactionType};
use crate::chain::tx::protocol::{ReadContext, SIGHASH_ALL};
use crate::consensus::Encodable;
use crate::consensus::encode::VarInt;
use crate::crypto::byte_util::Zeroable;
use crate::crypto::{UInt256, UInt384, UInt768};
// use crate::storage::manager::managed_context::ManagedContext;
// use crate::storage::models::chain::chain::ChainEntity;
// use crate::storage::models::tx::transaction::NewTransactionEntity;
use crate::util::Shared;

#[derive(Debug, Clone)]
pub struct QuorumCommitmentTransaction {
    pub base: Transaction,
    pub quorum_commitment_transaction_version: u16,
    pub quorum_commitment_height: u32,
    pub qf_commit_version: u16,
    pub llmq_type: LLMQType,
    pub quorum_hash: UInt256,
    pub signers_count: VarInt,
    pub signers_bitset: Vec<u8>,
    pub valid_members_count: VarInt,
    pub valid_members_bitset: Vec<u8>,
    pub quorum_public_key: UInt384,
    pub quorum_verification_vector_hash: UInt256,
    pub quorum_threshold_signature: UInt768,
    pub all_commitment_aggregated_signature: UInt768,
}

impl ITransaction for QuorumCommitmentTransaction {
    fn chain(&self) -> Shared<Chain> {
        self.base.chain()
    }
    fn chain_type(&self) -> ChainType {
        self.base.chain_type()
    }

    fn r#type(&self) -> TransactionType {
        TransactionType::QuorumCommitment
    }

    fn block_height(&self) -> u32 {
        self.base.block_height()
    }

    fn tx_hash(&self) -> UInt256 {
        self.base.tx_hash()
    }

    fn tx_lock_time(&self) -> u32 {
        self.base.tx_lock_time()
    }

    fn inputs(&self) -> Vec<TransactionInput> {
        self.base.inputs()
    }

    fn outputs(&self) -> Vec<TransactionOutput> {
        self.base.outputs()
    }

    fn input_addresses(&self) -> Vec<String> {
        self.base.input_addresses()
    }

    fn output_addresses(&self) -> Vec<String> {
        self.base.output_addresses()
    }

    fn size(&self) -> usize {
        if self.tx_hash().is_zero() {
            self.base.size() + VarInt(self.payload_data().len() as u64).len() + self.payload_data().len()
        } else {
            self.to_data().len()
        }
    }

    fn payload_data(&self) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        self.quorum_commitment_transaction_version.enc(&mut writer);
        self.quorum_commitment_height.enc(&mut writer);
        self.qf_commit_version.enc(&mut writer);
        let llmq_type: u8 = self.llmq_type.into();
        llmq_type.enc(&mut writer);
        self.quorum_hash.enc(&mut writer);
        self.signers_count.enc(&mut writer);
        self.signers_bitset.enc(&mut writer);
        self.valid_members_count.enc(&mut writer);
        self.valid_members_bitset.enc(&mut writer);
        self.quorum_public_key.enc(&mut writer);
        self.quorum_verification_vector_hash.enc(&mut writer);
        self.quorum_threshold_signature.enc(&mut writer);
        self.all_commitment_aggregated_signature.enc(&mut writer);
        writer
    }

    fn to_data_with_subscript_index(&self, subscript_index: Option<u64>) -> Vec<u8> {
        let mut data = self.base.to_data_with_subscript_index(subscript_index);
        self.payload_data().enc(&mut data);
        if subscript_index.is_some() {
            SIGHASH_ALL.enc(&mut data);
        }
        data
    }

    fn set_instant_send_received_with_instant_send_lock(&mut self, instant_send_lock: Option<Shared<InstantSendLock>>) {
        self.base.set_instant_send_received_with_instant_send_lock(instant_send_lock);
    }

    fn is_coinbase_classic_transaction(&self) -> bool {
        self.base.is_coinbase_classic_transaction()
    }

    // fn has_non_dust_output_in_wallet(&self, wallet: &Wallet) -> bool {
    //     self.base.has_non_dust_output_in_wallet(wallet)
    // }
    //
    // fn set_initial_persistent_attributes_in_context(&mut self, context: &ManagedContext) -> bool {
    //     todo!()
    // }
    //
    // fn to_entity_with_chain_entity(&self, chain_entity: ChainEntity) -> NewTransactionEntity {
    //     todo!()
    // }
    // fn load_blockchain_identities_from_derivation_paths(&mut self, derivation_paths: Vec<&dyn IDerivationPath>) {
    //     self.base.load_blockchain_identities_from_derivation_paths(derivation_paths)
    // }
}


impl<'a> TryRead<'a, ReadContext> for QuorumCommitmentTransaction {
    fn try_read(bytes: &'a [u8], context: ReadContext) -> byte::Result<(Self, usize)> {
        let (mut base, mut offset) = Transaction::try_read(bytes, context)?;
        base.tx_type = TransactionType::QuorumCommitment;
        let _extra_payload_size = bytes.read_with::<VarInt>(&mut offset, byte::LE)?;
        let quorum_commitment_transaction_version = bytes.read_with::<u16>(&mut offset, byte::LE)?;
        let quorum_commitment_height = bytes.read_with::<u32>(&mut offset, byte::LE)?;
        let qf_commit_version = bytes.read_with::<u16>(&mut offset, byte::LE)?;
        let llmq_type = bytes.read_with::<LLMQType>(&mut offset, byte::LE)?;
        let quorum_hash = bytes.read_with::<UInt256>(&mut offset, byte::LE)?;
        let signers_count = bytes.read_with::<VarInt>(&mut offset, byte::LE)?;
        let signers_buffer_length: usize = ((signers_count.0 as usize) + 7) / 8;
        let signers_bitset: &[u8] = bytes.read_with(&mut offset, Bytes::Len(signers_buffer_length))?;
        let valid_members_count = bytes.read_with::<VarInt>(&mut offset, byte::LE)?;
        let valid_members_count_buffer_length: usize = ((valid_members_count.0 as usize) + 7) / 8;
        let valid_members_bitset: &[u8] =
            bytes.read_with(&mut offset, Bytes::Len(valid_members_count_buffer_length))?;
        let quorum_public_key = bytes.read_with::<UInt384>(&mut offset, byte::LE)?;
        let quorum_verification_vector_hash = bytes.read_with::<UInt256>(&mut offset, byte::LE)?;
        let quorum_threshold_signature = bytes.read_with::<UInt768>(&mut offset, byte::LE)?;
        let all_commitment_aggregated_signature = bytes.read_with::<UInt768>(&mut offset, byte::LE)?;
        base.payload_offset = offset;
        let mut tx = Self {
            base,
            quorum_commitment_transaction_version,
            quorum_commitment_height,
            qf_commit_version,
            llmq_type,
            quorum_hash,
            signers_count,
            signers_bitset: signers_bitset.to_vec(),
            valid_members_count,
            valid_members_bitset: valid_members_bitset.to_vec(),
            quorum_public_key,
            quorum_verification_vector_hash,
            quorum_threshold_signature,
            all_commitment_aggregated_signature
        };
        // todo verify inputs hash
        assert_eq!(tx.payload_data().len(), offset, "Payload length doesn't match ");
        tx.base.tx_hash = UInt256::sha256d(tx.to_data());
        Ok((tx, offset))
    }
}
