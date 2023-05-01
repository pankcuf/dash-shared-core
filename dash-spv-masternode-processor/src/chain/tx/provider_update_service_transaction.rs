use byte::{BytesExt, TryRead};
use crate::chain::chain::Chain;
use crate::chain::common::ChainType;
use crate::chain::tx::{InstantSendLock, ITransaction, ProviderRegistrationTransaction, Transaction, TransactionInput, TransactionOutput, TransactionType};
use crate::chain::tx::protocol::{ReadContext, SIGHASH_ALL};
use crate::consensus::Encodable;
use crate::consensus::encode::VarInt;
use crate::crypto::{UInt128, UInt256, UInt768, VarBytes};
use crate::crypto::byte_util::Zeroable;
use crate::keys::{BLSKey, IKey};
use crate::util::address::address;
// use crate::storage::manager::managed_context::ManagedContext;
// use crate::storage::models::chain::chain::ChainEntity;
// use crate::storage::models::tx::transaction::NewTransactionEntity;
use crate::util::Shared;

#[derive(Clone, Debug, Default)]
pub struct ProviderUpdateServiceTransaction {
    pub base: Transaction,
    pub provider_update_service_transaction_version: u16,
    pub provider_registration_transaction_hash: UInt256,
    pub ip_address: UInt128, // v6, but only v4 supported
    pub port: u16,
    pub script_payout: Vec<u8>,
    pub inputs_hash: UInt256,
    pub payload_signature: Vec<u8>,
    provider_registration_transaction: Option<Shared<ProviderRegistrationTransaction>>,
}

impl ITransaction for ProviderUpdateServiceTransaction {
    fn chain(&self) -> Shared<Chain> {
        self.base.chain()
    }
    fn chain_type(&self) -> ChainType {
        self.base.chain_type()
    }
    fn r#type(&self) -> TransactionType {
        TransactionType::ProviderUpdateService
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
        let mut writer: Vec<u8> = Vec::new();
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
    //     self.base.set_initial_persistent_attributes_in_context(context)
    // }
    //
    // fn to_entity_with_chain_entity(&self, chain_entity: ChainEntity) -> NewTransactionEntity {
    //     todo!()
    // }
    //
    fn trigger_updates_for_local_references(&self) {
        // if let Some(mut local_masternode) = self.chain().with(|chain| chain.masternode_manager.local_masternode_having_provider_registration_transaction_hash(&self.provider_registration_transaction_hash)) {
        //     local_masternode.update_with_update_service_transaction(self, true);
        // }
    }
    //
    // fn load_blockchain_identities_from_derivation_paths(&mut self, derivation_paths: Vec<&dyn IDerivationPath>) {
    //     self.base.load_blockchain_identities_from_derivation_paths(derivation_paths)
    // }
}

impl ProviderUpdateServiceTransaction {
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
        //     self.provider_registration_transaction = self.chain().with(|chain| chain.transaction_for_hash(&self.provider_registration_transaction_hash));
        // }
    }

    pub fn payload_hash(&self) -> UInt256 {
        UInt256::sha256d(self.payload_data_for_hash())
    }

    pub fn check_payload_signature(&mut self) -> bool {
        assert!(self.provider_registration_transaction().is_some(), "We need a provider registration transaction");
        //[DSBLSKey keyWithPublicKey:self.providerRegistrationTransaction.operatorKey useLegacy:[self.chain useLegacyBLS]]
        // todo: check use_legacy_bls has taken from appropriate place
        let mut key = BLSKey::key_with_public_key(self.provider_registration_transaction().unwrap().with(|tx| tx.operator_key.clone()), self.chain_type().use_legacy_bls());
        key.verify(&self.payload_hash().0.to_vec(), &self.payload_signature)
    }

    pub fn sign_payload_with_key(&mut self, key: &BLSKey) {
        self.payload_signature = key.sign(&self.payload_data_for_hash())
    }

    pub fn payout_address(&mut self) -> Option<String> {
        if let Some(tx) = self.provider_registration_transaction() {
            if !self.script_payout.is_empty() {
                return address::with_script_pub_key(&self.script_payout, &tx.with(|tx| tx.chain_type().script_map()));
            }
        }
        None // no payout address
    }

    pub fn base_payload_data(&self) -> Vec<u8> {
        let mut writer: Vec<u8> = Vec::new();
        self.provider_update_service_transaction_version.enc(&mut writer);
        self.provider_registration_transaction_hash.enc(&mut writer);
        self.ip_address.enc(&mut writer);
        self.port.swap_bytes().enc(&mut writer);
        VarInt(self.script_payout.len() as u64).enc(&mut writer);
        self.script_payout.enc(&mut writer);
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

impl<'a> TryRead<'a, ReadContext> for ProviderUpdateServiceTransaction {
    fn try_read(bytes: &'a [u8], context: ReadContext) -> byte::Result<(Self, usize)> {
        let (mut base, mut offset) = Transaction::try_read(bytes, context)?;
        base.tx_type = TransactionType::ProviderUpdateService;
        let _extra_payload_size = bytes.read_with::<VarInt>(&mut offset, byte::LE)?;
        let provider_update_service_transaction_version = bytes.read_with::<u16>(&mut offset, byte::LE)?;
        let provider_registration_transaction_hash = bytes.read_with::<UInt256>(&mut offset, byte::LE)?;
        let ip_address = bytes.read_with::<UInt128>(&mut offset, byte::LE)?;
        // // todo: choose one way of BE vs swap_bytes()
        let port = bytes.read_with::<u16>(&mut offset, byte::BE)?;
        let script_payout = bytes.read_with::<VarBytes>(&mut offset, byte::LE)?.1.to_vec();
        let inputs_hash = bytes.read_with::<UInt256>(&mut offset, byte::LE)?;
        let payload_signature = bytes.read_with::<UInt768>(&mut offset, byte::LE)?;
        base.payload_offset = offset;
        let mut tx = Self {
            base,
            provider_update_service_transaction_version,
            provider_registration_transaction_hash,
            ip_address,
            port,
            script_payout,
            inputs_hash,
            payload_signature: payload_signature.0.to_vec(),
            provider_registration_transaction: None
        };
        // // todo verify inputs hash
        assert_eq!(tx.payload_data().len(), offset, "Payload length doesn't match ");
        tx.base.tx_hash = UInt256::sha256d(tx.to_data());
        Ok((tx, offset))
    }
}
