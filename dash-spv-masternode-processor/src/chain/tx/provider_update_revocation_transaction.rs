use byte::{BytesExt, TryRead};
use crate::chain::chain::Chain;
use crate::chain::common::ChainType;
use crate::chain::tx::instant_send_lock::InstantSendLock;
use crate::chain::tx::protocol::{ReadContext, SIGHASH_ALL};
use crate::chain::tx::provider_registration_transaction::ProviderRegistrationTransaction;
use crate::chain::tx::{ITransaction, Transaction, TransactionInput, TransactionOutput, TransactionType};
use crate::consensus::Encodable;
use crate::consensus::encode::VarInt;
use crate::crypto::{UInt256, UInt768};
use crate::crypto::byte_util::Zeroable;
use crate::keys::{BLSKey, IKey};
// use crate::storage::manager::managed_context::ManagedContext;
// use crate::storage::models::chain::chain::ChainEntity;
// use crate::storage::models::tx::transaction::NewTransactionEntity;
use crate::util::Shared;

#[derive(Clone, Debug, Default)]
pub struct ProviderUpdateRevocationTransaction {
    pub base: Transaction,
    pub provider_registration_transaction_hash: UInt256,
    pub provider_update_revocation_transaction_version: u16,
    pub reason: u16,
    pub inputs_hash: UInt256,

    pub payload_signature: Vec<u8>,

    pub provider_registration_transaction: Option<Shared<ProviderRegistrationTransaction>>,
}

impl ITransaction for ProviderUpdateRevocationTransaction {
    fn chain(&self) -> Shared<Chain> {
        self.base.chain()
    }
    fn chain_type(&self) -> ChainType {
        self.base.chain_type()
    }

    fn r#type(&self) -> TransactionType {
        TransactionType::ProviderUpdateRevocation
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
        if !self.tx_hash().is_zero() {
            return self.to_data().len();
        }
        self.base.size() + VarInt(self.payload_data().len() as u64).len() + self.base_payload_data().len() + 96
    }

    fn payload_data(&self) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        self.base_payload_data().enc(&mut writer);
        self.payload_signature.enc(&mut writer);
        writer
    }

    fn payload_data_for_hash(&self) -> Vec<u8> {
        self.base_payload_data()
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

    fn has_set_inputs_and_outputs(&mut self) {
        self.update_inputs_hash();
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
    //
    fn trigger_updates_for_local_references(&self) {
        // if let Some(mut local_masternode) = self.chain().with(|chain| chain.masternode_manager.local_masternode_having_provider_registration_transaction_hash(&self.provider_registration_transaction_hash)) {
        //     local_masternode.update_with_update_revocation_transaction(self, true);
        // }
    }
    //
    // fn load_blockchain_identities_from_derivation_paths(&mut self, derivation_paths: Vec<&dyn IDerivationPath>) {
    //     self.base.load_blockchain_identities_from_derivation_paths(derivation_paths)
    // }
}

impl ProviderUpdateRevocationTransaction {

    pub fn provider_registration_transaction(&mut self) -> Option<Shared<ProviderRegistrationTransaction>> {
        todo!()
        // if let Some(tx) = &self.provider_registration_transaction {
        //     Some(tx)
        // } else {
        //     let tx = self.chain().transaction_for_hash(&self.provider_registration_transaction_hash);
        //     self.provider_registration_transaction = tx;
        //     tx
        // }
    }

    pub fn set_provider_registration_transaction_hash(&mut self, hash: UInt256) {
        todo!()
        // self.provider_registration_transaction_hash = hash;
        // if self.provider_registration_transaction.is_none() {
        //     self.provider_registration_transaction = self.chain().transaction_for_hash(&self.provider_registration_transaction_hash);
        // }
    }

    pub fn payload_hash(&self) -> UInt256 {
        UInt256::sha256d(self.payload_data_for_hash())
    }

    pub fn check_payload_signature(&mut self) -> bool {
        assert!(self.provider_registration_transaction.is_some(), "We need a provider registration transaction");
        // todo: check use_legacy_bls has taken from appropriate place
        let mut key = BLSKey::key_with_public_key(self.provider_registration_transaction().unwrap().with(|tx| tx.operator_key.clone()), self.chain_type().use_legacy_bls());
        self.check_payload_signature_with_key(&mut key)
    }

    pub fn check_payload_signature_with_key(&mut self, key: &mut BLSKey) -> bool {
        // todo: where migrate to bytes to avoid Vec<u8> <--> UInt256 conversion?
        key.verify(&self.payload_hash().0.to_vec(), &self.payload_signature)
    }

    pub fn sign_payload_with_key(&mut self, private_key: &BLSKey) {
        // ATTENTION If this ever changes from ECDSA, change the max signature size defined above
        self.payload_signature = private_key.sign_data(&self.payload_data_for_hash()).0.to_vec();
    }

    pub fn base_payload_data(&self) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        self.provider_update_revocation_transaction_version.enc(&mut writer);
        self.provider_registration_transaction_hash.enc(&mut writer);
        self.reason.enc(&mut writer);
        self.inputs_hash.enc(&mut writer);
        writer
    }

    pub fn update_inputs_hash(&mut self) {
        let mut writer = Vec::<u8>::new();
        self.inputs().iter().for_each(|input| {
            input.input_hash.enc(&mut writer);
            input.index.enc(&mut writer);
        });
        self.inputs_hash = UInt256::sha256d(writer);
    }

}

impl<'a> TryRead<'a, ReadContext> for ProviderUpdateRevocationTransaction {
    fn try_read(bytes: &'a [u8], context: ReadContext) -> byte::Result<(Self, usize)> {
        let (mut base, mut offset) = Transaction::try_read(bytes, context)?;
        base.tx_type = TransactionType::ProviderUpdateRevocation;
        let _extra_payload_size = bytes.read_with::<VarInt>(&mut offset, byte::LE)?;
        let provider_update_revocation_transaction_version = bytes.read_with::<u16>(&mut offset, byte::LE)?;
        let provider_registration_transaction_hash = bytes.read_with::<UInt256>(&mut offset, byte::LE)?;
        let reason = bytes.read_with::<u16>(&mut offset, byte::LE)?;
        let inputs_hash = bytes.read_with::<UInt256>(&mut offset, byte::LE)?;
        let payload_signature = bytes.read_with::<UInt768>(&mut offset, byte::LE)?;
        base.payload_offset = offset;
        let mut tx = Self {
            base,
            provider_registration_transaction_hash,
            provider_update_revocation_transaction_version,
            reason,
            inputs_hash,
            payload_signature: payload_signature.0.to_vec(),
            provider_registration_transaction: None
        };
        // todo verify inputs hash
        assert_eq!(tx.payload_data().len(), offset, "Payload length doesn't match ");
        tx.base.tx_hash = UInt256::sha256d(tx.to_data());
        Ok((tx, offset))
    }
}
