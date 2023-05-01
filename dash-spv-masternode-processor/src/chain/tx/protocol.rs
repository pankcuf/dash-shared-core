use std::fmt::Debug;
use std::hash::Hasher;
use crate::chain::Chain;
use crate::chain::common::ChainType;
use crate::chain::tx::{InstantSendLock, TransactionInput, TransactionOutput, TransactionType};
use crate::chain::wallet::Account;
use crate::crypto::{byte_util::AsBytes, UInt256};
use crate::keys::ECDSAKey;
use crate::util::Shared;

// block height indicating transaction is unconfirmed
pub const TX_UNCONFIRMED: i32 = i32::MAX;

pub static SIGHASH_ALL: u32 = 1;
pub static TX_VERSION: u32 = 0x00000001;
pub static SPECIAL_TX_VERSION: u32 = 0x00000003;
pub static TX_LOCKTIME: u32 = 0x00000000;
pub static TXIN_SEQUENCE: u32 = u32::MAX;
// a lockTime below this value is a block height, otherwise a timestamp
pub const TX_MAX_LOCK_HEIGHT: u32 = 500000000;

pub const MAX_ECDSA_SIGNATURE_SIZE: usize = 75;

#[derive(Clone)]
pub struct ReadContext(pub ChainType, pub Shared<Chain>);


pub trait ITransaction: Debug + Send + Sync {
    fn chain(&self) -> Shared<Chain>;
    fn chain_type(&self) -> ChainType;
    fn accounts(&self) -> Vec<&Account> where Self: Sized {
        todo!()
        // let vec = self.chain().accounts_that_can_contain_transaction(self);
        // vec
    }
    fn first_account(&self) -> Option<&Account>  where Self: Sized {
        todo!()
        // let acc = self.chain().first_account_that_can_contain_transaction(self);
        // acc
    }

    fn r#type(&self) -> TransactionType;
    fn block_height(&self) -> u32;
    fn tx_hash(&self) -> UInt256;
    fn tx_lock_time(&self) -> u32;
    fn inputs(&self) -> Vec<TransactionInput>;
    fn outputs(&self) -> Vec<TransactionOutput>;
    fn input_addresses(&self) -> Vec<String>;
    fn output_addresses(&self) -> Vec<String>;

    fn size(&self) -> usize;

    fn payload_data(&self) -> Vec<u8> {
        vec![]
    }
    fn payload_data_for_hash(&self) -> Vec<u8> {
        vec![]
    }

    fn to_data(&self) -> Vec<u8> {
        self.to_data_with_subscript_index(None)
    }
    fn to_data_with_subscript_index(&self, subscript_index: Option<u64>) -> Vec<u8>;

    fn set_instant_send_received_with_instant_send_lock(&mut self, instant_send_lock: Option<Shared<InstantSendLock>>);
    fn is_coinbase_classic_transaction(&self) -> bool;
    fn has_set_inputs_and_outputs(&mut self) {}
    // fn has_non_dust_output_in_wallet(&self, wallet: &Wallet) -> bool;

    fn transaction_type_requires_inputs(&self) -> bool {
        self.r#type().requires_inputs()
    }

    // fn set_initial_persistent_attributes_in_context(&mut self, context: &ManagedContext) -> bool;
    // fn to_entity_with_chain_entity(&self, chain_entity: ChainEntity) -> NewTransactionEntity;

    fn sign_with_private_keys(&mut self, keys: Vec<ECDSAKey>) -> bool {
        todo!()
        // self.sign_with_private_keys_using_addresses(
        //     keys,
        //     keys.iter()
        //         .map(|mut key| Address::with_public_key_data(&key.public_key_data(), &self.chain_type().script_map()))
        //         .collect())
    }
    // fn sign_with_private_keys_using_addresses(&mut self, keys: Vec<&dyn IKey>, addresses: Vec<String>) -> bool;

    fn trigger_updates_for_local_references(&self) {}
    // fn associate_with_accepted_invitation(&self, invitation: &Invitation, index: u32, dashpay_username: String, wallet: &Wallet) -> Identity {
    //     panic!("Only for CreditFunding")
    // }
    // fn load_blockchain_identities_from_derivation_paths(&mut self, derivation_paths: Vec<&dyn IDerivationPath>);
}

impl PartialEq<Self> for dyn ITransaction {
    fn eq(&self, other: &Self) -> bool {
        self == other || self.tx_hash() == other.tx_hash()
    }
}

impl std::hash::Hash for dyn ITransaction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.tx_hash().as_bytes())
    }
}

