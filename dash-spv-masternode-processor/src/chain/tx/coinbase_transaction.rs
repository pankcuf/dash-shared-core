use byte::{BytesExt, TryRead};
use crate::chain::chain::Chain;
use crate::chain::common::ChainType;
use crate::chain::tx::{InstantSendLock, ITransaction, Transaction, TransactionInput, TransactionOutput, TransactionType};
use crate::chain::tx::protocol::ReadContext;
use crate::consensus::encode::VarInt;
use crate::consensus::Encodable;
use crate::crypto::byte_util::Zeroable;
use crate::crypto::UInt256;
// use crate::storage::manager::managed_context::ManagedContext;
// use crate::storage::models::chain::chain::ChainEntity;
// use crate::storage::models::entity::Entity;
// use crate::storage::models::tx::transaction::NewTransactionEntity;
use crate::util::data_append::DataAppend;
use crate::util::Shared;

#[derive(Debug, Default, Clone)]
pub struct CoinbaseTransaction {
    pub base: Transaction,
    pub coinbase_transaction_version: u16,
    pub height: u32,
    pub merkle_root_mn_list: UInt256,
    pub merkle_root_llmq_list: Option<UInt256>,
    pub locked_amount: u64,
}

// impl_bytes_decodable!(CoinbaseTransaction);

impl<'a> TryRead<'a, ReadContext> for CoinbaseTransaction {
    fn try_read(bytes: &'a [u8], context: ReadContext) -> byte::Result<(Self, usize)> {
        let (mut base, mut offset) = Transaction::try_read(bytes, context)?;
        base.tx_type = TransactionType::Coinbase;
        let _extra_payload_size = bytes.read_with::<VarInt>(&mut offset, byte::LE)?;
        let coinbase_transaction_version = bytes.read_with::<u16>(&mut offset, byte::LE)?;
        let height = bytes.read_with::<u32>(&mut offset, byte::LE)?;
        let merkle_root_mn_list = bytes.read_with::<UInt256>(&mut offset, byte::LE)?;
        // todo: check maybe better use version >= 2 here
        let merkle_root_llmq_list = if coinbase_transaction_version >= 2 {
            Some(bytes.read_with::<UInt256>(&mut offset, byte::LE)?)
        } else {
            None
        };
        let locked_amount = if coinbase_transaction_version >= 3 {
            bytes.read_with::<u64>(&mut offset, byte::LE)?
        } else {
            u64::MAX
        };
        base.payload_offset = offset;
        let mut tx = Self {
            base,
            coinbase_transaction_version,
            height,
            merkle_root_mn_list,
            merkle_root_llmq_list,
            locked_amount
        };
        tx.base.tx_hash = UInt256::sha256d(tx.to_data());
        Ok((tx, offset))
    }
}

impl CoinbaseTransaction {

    pub fn has_found_coinbase(&mut self, hashes: &[UInt256]) -> bool {
        if self.base.tx_hash.is_zero() {
            self.base.tx_hash = UInt256::sha256d(self.to_data());
        }
        self.has_found_coinbase_internal(self.base.tx_hash, hashes)
    }

    fn has_found_coinbase_internal(&self, coinbase_hash: UInt256, hashes: &[UInt256]) -> bool {
        hashes
            .iter()
            .filter(|&h| coinbase_hash.cmp(h).is_eq())
            .count()
            > 0
    }

    pub fn init_with_coinbase_message(message: &String, payment_addresses: Vec<String>, height: u32, chain_type: ChainType, chain: Shared<Chain>) -> Self {
        let mut base = Transaction::init_on_chain(chain_type, chain);
        base.add_input_hash_with_signature(UInt256::MIN, u32::MAX, None, Some(Vec::<u8>::from_coinbase_message(message, height)), u32::MAX);
        let new_outputs = payment_addresses.into_iter().map(|address| TransactionOutput {
            amount: chain_type.base_reward() / address.len() as u64,
            script: Some(Vec::<u8>::script_pub_key_for_address(&address, &chain_type.script_map())),
            address: Some(address)
        });
        base.outputs.extend(new_outputs);
        let mut tx = Self {
            base,
            coinbase_transaction_version: 0,
            height,
            merkle_root_mn_list: Default::default(),
            merkle_root_llmq_list: None,
            locked_amount: u64::MAX
        };
        tx.base.tx_hash = UInt256::sha256d(tx.to_data());
        tx
    }
}

impl ITransaction for CoinbaseTransaction {
    fn chain(&self) -> Shared<Chain> {
        self.base.chain()
    }

    fn chain_type(&self) -> ChainType {
        self.base.chain_type()
    }

    fn r#type(&self) -> TransactionType {
        TransactionType::Coinbase
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
        self.base.size() + VarInt(self.payload_data().len() as u64).len()
    }

    fn payload_data(&self) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        self.coinbase_transaction_version.enc(&mut writer);
        self.height.enc(&mut writer);
        self.merkle_root_mn_list.enc(&mut writer);
        if self.coinbase_transaction_version >= 2 {
            self.merkle_root_llmq_list.unwrap().enc(&mut writer);
        }
        if self.coinbase_transaction_version >= 3 {
            self.locked_amount.enc(&mut writer);
        }
        writer
    }

    fn set_instant_send_received_with_instant_send_lock(&mut self, instant_send_lock: Option<Shared<InstantSendLock>>) {
        self.base.set_instant_send_received_with_instant_send_lock(instant_send_lock);
    }

    fn to_data_with_subscript_index(&self, subscript_index: Option<u64>) -> Vec<u8> {
        let mut writer = Transaction::data_with_subscript_index_static(
            subscript_index,
            self.base.version,
            self.base.tx_type,
            &self.base.inputs,
            &self.base.outputs,
            self.base.lock_time,
        );
        self.payload_data().enc(&mut writer);
        writer
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
    //     let mut base = self.base.to_entity_with_chain_entity(chain_entity);
    //     base.special_transaction_version = Some(self.coinbase_transaction_version as i16);
    //     base.height = Some(self.height as i32);
    //     base.mn_list_merkle_root = Some(self.merkle_root_mn_list);
    //     base
    // }
    //
    // fn load_blockchain_identities_from_derivation_paths(&mut self, derivation_paths: Vec<&dyn IDerivationPath>) {
    //     self.base.load_blockchain_identities_from_derivation_paths(derivation_paths)
    // }

    fn is_coinbase_classic_transaction(&self) -> bool {
        self.base.is_coinbase_classic_transaction()
    }
}
