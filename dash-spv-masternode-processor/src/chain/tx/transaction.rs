use std::cmp::Ordering;
use std::collections::HashSet;
use byte::{BytesExt, TryRead};
use secp256k1::rand::{Rng, thread_rng};
use crate::blockdata::opcodes::all::OP_RETURN;
use crate::chain::{Chain, ScriptMap};
use crate::chain::common::{ChainType, DevnetType};
use crate::chain::params::{TX_FEE_PER_B, TX_FEE_PER_INPUT, TX_INPUT_SIZE, TX_OUTPUT_SIZE};
use crate::chain::tx::instant_send_lock::InstantSendLock;
use crate::chain::tx::protocol::{ITransaction, ReadContext, SIGHASH_ALL, TX_LOCKTIME, TX_UNCONFIRMED, TX_VERSION, TXIN_SEQUENCE};
use crate::chain::tx::transaction_input::TransactionInput;
use crate::chain::tx::transaction_output::TransactionOutput;
use crate::chain::tx::transaction_persistence_status::TransactionPersistenceStatus;
use crate::chain::tx::TransactionType;
use crate::consensus::{Encodable, encode::VarInt};
use crate::crypto::{byte_util::Zeroable, UInt256};
use crate::keys::{ECDSAKey, IKey, Key};
use crate::platform::identity::identity::Identity;
use crate::util::address::address;
use crate::util::Shared;
use crate::util::data_append::DataAppend;

#[derive(Clone, Debug, Default)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub struct Transaction {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub lock_time: u32,
    pub version: u16,
    pub tx_hash: UInt256,
    pub tx_type: TransactionType,
    pub payload_offset: usize,
    pub block_height: u32,
    pub timestamp: u64,
    pub instant_send_lock_awaiting_processing: Option<Shared<InstantSendLock>>,
    pub instant_send_received: bool,
    pub has_unverified_instant_send_lock: bool,
    // associated_shapeshift: Option<ShapeshiftEntity>,
    persistence_status: TransactionPersistenceStatus,
    source_identities: HashSet<Identity>,
    destination_identities: HashSet<Identity>,
    confirmed: bool,

    chain_type: ChainType,
    chain: Shared<Chain>,
}

impl ITransaction for Transaction {
    fn chain(&self) -> Shared<Chain> {
        self.chain.clone()
    }
    fn chain_type(&self) -> ChainType {
        self.chain_type
    }
    fn r#type(&self) -> TransactionType {
        //TransactionType::Classic
        self.tx_type
    }

    fn block_height(&self) -> u32 {
        self.block_height
    }

    fn tx_hash(&self) -> UInt256 {
        self.tx_hash
    }

    fn tx_lock_time(&self) -> u32 {
        self.lock_time
    }

    fn inputs(&self) -> Vec<TransactionInput> {
        self.inputs.clone()
    }

    fn outputs(&self) -> Vec<TransactionOutput> {
        self.outputs.clone()
    }

    fn input_addresses(&self) -> Vec<String> {
        // TODO: check may be it worth to keep index with Option<String>
        self.inputs.iter().filter_map(|input| {
            if let Some(script) = &input.script {
                address::with_script_pub_key(script, &self.chain_type.script_map())
            } else if let Some(signature) = &input.signature {
                address::with_script_sig(signature, &self.chain_type.script_map())
            } else {
                None
            }
        }).collect()
    }

    fn output_addresses(&self) -> Vec<String> {
        // TODO: check may be it worth to keep index with Option<String>
        self.outputs.iter().filter_map(|output| output.address.clone()).collect()
    }

    /// size in bytes if signed, or estimated size assuming compact pubkey sigs
    fn size(&self) -> usize {
        if !self.tx_hash.is_zero() {
            // todo: check size() is properly overriden according to 'their' to_data
            return self.to_data().len();
        }
        let input_count = self.inputs.len();
        let output_count = self.outputs.len();
        return 8 + VarInt(input_count as u64).len() + VarInt(output_count as u64).len() + TX_INPUT_SIZE as usize * input_count + TX_OUTPUT_SIZE as usize * output_count;
    }

    fn to_data_with_subscript_index(&self, subscript_index: Option<u64>) -> Vec<u8> {
        Self::data_with_subscript_index_static(
            subscript_index,
            self.version,
            self.tx_type,
            &self.inputs,
            &self.outputs,
            self.lock_time,
        )
    }

    fn set_instant_send_received_with_instant_send_lock(&mut self, instant_send_lock: Option<Shared<InstantSendLock>>) {
        todo!()
        // if let Some(lock) = instant_send_lock {
        //     let is_signature_verified = lock.signature_verified;
        //     self.instant_send_received = is_signature_verified;
        //     self.has_unverified_instant_send_lock = !is_signature_verified;
        //     if is_signature_verified {
        //         self.instant_send_lock_awaiting_processing = instant_send_lock;
        //     }
        //     if !lock.saved {
        //         lock.save_initial();
        //     }
        // }
    }

    fn is_coinbase_classic_transaction(&self) -> bool {
        if self.inputs.len() == 1 {
            let first_input = self.inputs.first().unwrap();
            if first_input.input_hash.is_zero() && first_input.index == u32::MAX {
                return true;
            }
        }
        return false;
    }

    // Info
    // fn has_non_dust_output_in_wallet(&self, wallet: &Wallet) -> bool {
    //     self.outputs.iter().find(|output| {
    //         if let Some(address) = &output.address {
    //             output.amount > TX_MIN_OUTPUT_AMOUNT && wallet.contains_address(address)
    //         } else {
    //             false
    //         }
    //     }).is_some()
    // }

    // fn set_initial_persistent_attributes_in_context(&mut self, context: &ManagedContext) -> bool {
    //     // TODO: impl prepare and commit changes in managed context (to delay insert) or use TransactionEntity::save_transaction_for
    //     match TransactionEntity::count_transactions_for_hash(&self.tx_hash(), context) {
    //         Ok(0) => match context.prepare(self.to_entity(), TransactionEntity::create) {
    //             Ok(1) => true,
    //             _ => false
    //         },
    //         _ => false
    //     }
    // }
    // fn to_entity_with_chain_entity(&self, chain_entity: ChainEntity) -> NewTransactionEntity {
    //     NewTransactionEntity {
    //         hash: self.tx_hash,
    //         block_height: self.block_height as i32,
    //         version: self.version as i16,
    //         lock_time: self.lock_time as i32,
    //         timestamp: NaiveDateTime::from_timestamp_opt(self.timestamp as i64, 0).unwrap(),
    //         chain_id: chain_entity.id,
    //         associated_shapeshift_id: self.associated_shapeshift.and_then(|sh| Some(sh.id)).or(None),
    //         ..Default::default()
    //     }
    // }
    //
    fn trigger_updates_for_local_references(&self) {

    }
    //
    // fn load_blockchain_identities_from_derivation_paths(&mut self, derivation_paths: Vec<&dyn IDerivationPath>) {
    //     let mut destination_identities = HashSet::new();
    //     let mut source_identities = HashSet::new();
    //     for output in self.outputs {
    //         for derivation_path in derivation_paths {
    //             if derivation_path.kind() == DerivationPathKind::IncomingFunds {
    //                 if let Some(address) = &output.address {
    //                     if derivation_path.contains_address(address) {
    //                         let (source_identity, destination_identity) = derivation_path.load_identities();
    //                         // these need to be inverted since the derivation path is incoming
    //                         if let Some(value) = source_identity {
    //                             destination_identities.insert(value);
    //                         }
    //                         if let Some(value) = destination_identity {
    //                             source_identities.insert(value);
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     self.source_identities.extend(source_identities);
    //     self.destination_identities.extend(destination_identities);
    // }
}

impl<'a> TryRead<'a, ReadContext> for Transaction {
    fn try_read(bytes: &'a [u8], context: ReadContext) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let version = bytes.read_with::<u16>(offset, byte::LE)?;
        let tx_type_uint = bytes.read_with::<u16>(offset, byte::LE)?;
        let tx_type = TransactionType::from(tx_type_uint);
        let count_var = bytes.read_with::<VarInt>(offset, byte::LE)?;
        let count = count_var.0;
        // at least one input is required
        if count == 0 && tx_type.requires_inputs() {
            return Err(byte::Error::Incomplete);
        }
        let mut inputs: Vec<TransactionInput> = Vec::new();
        for _i in 0..count {
            inputs.push(bytes.read_with::<TransactionInput>(offset, byte::LE)?);
        }
        let mut outputs: Vec<TransactionOutput> = Vec::new();
        let count_var = bytes.read_with::<VarInt>(offset, byte::LE)?;
        let count = count_var.0;
        for _i in 0..count {
            outputs.push(bytes.read_with::<TransactionOutput>(offset, byte::LE)?);
        }
        let lock_time = bytes.read_with::<u32>(offset, byte::LE)?;
        let tx = Self {
            inputs,
            outputs,
            version,
            tx_type,
            lock_time,
            payload_offset: *offset,
            block_height: TX_UNCONFIRMED as u32,
            chain_type: context.0,
            chain: context.1,
            ..Default::default()
        };
        /*
        if tx_type != TransactionType::Classic {
            return Ok((tx, *offset));
        } else {
            // only classic transactions are shapeshifted
            tx.tx_hash = UInt256::sha256d(&tx.to_data());
            if let Some(outbound_shapeshift_address) = tx.shapeshift_outbound_address() {
                if let Ok(mut entity) = ShapeshiftEntity::having_withdrawal_address(&outbound_shapeshift_address, tx.chain().chain_context()) {
                    if entity.shapeshift_status == ShapeshiftAddressStatus::Unused.into() {
                        entity.shapeshift_status = ShapeshiftAddressStatus::NoDeposits.into();
                        // save later
                    }
                    tx.associated_shapeshift = Some(entity);
                } else if let Some(possibleOutboundShapeshiftAddress) = tx.shapeshift_outbound_address_force_script() {
                    if let Ok(mut entity) = ShapeshiftEntity::having_withdrawal_address(&possibleOutboundShapeshiftAddress, tx.chain().chain_context()) {
                        if entity.shapeshift_status == ShapeshiftAddressStatus::Unused.into() {
                            entity.shapeshift_status = ShapeshiftAddressStatus::NoDeposits.into();
                            // save later
                        }
                        tx.associated_shapeshift = Some(entity);
                    }
                }
                if tx.associated_shapeshift.is_none() && !tx.outputs().is_empty() {
                    let mut all_addresses = Vec::<String>::new();
                    match AddressEntity::all(tx.chain().chain_context()) {
                        Ok(entities) => {
                            all_addresses = entities.iter().map(|e| e.address).collect();
                        },
                        Err(err) => panic!("Can't read all address entities")
                    }
                    if let Some(main_output_address) = tx.outputs().iter().find_map(|output| {
                        if let Some(addr) = &output.address {
                            if all_addresses.contains(addr) {
                                return Some(addr);
                            }
                        }
                        None
                    }) {
                        if let Ok(entity) = ShapeshiftEntity::register_shapeshift_with_addess(main_output_address, &outbound_shapeshift_address, ShapeshiftAddressStatus::NoDeposits, tx.chain().chain_context()) {
                            tx.associated_shapeshift = Some(entity);
                        }
                    }
                }
            } else {
                return Ok((tx, *offset));
            }
        }*/
        Ok((tx, *offset))
    }
}

impl Transaction {

    pub fn standard_fee(&self) -> u64 {
        TX_FEE_PER_B * self.size() as u64
    }

    pub fn standard_instant_fee(&self) -> u64 {
        TX_FEE_PER_INPUT * self.inputs.len() as u64
    }

    pub fn is_credit_funding_transaction(&self) -> bool {
        self.outputs.iter().filter(|o| {
            if let Some(s) = &o.script {
                if s[0] == OP_RETURN.into_u8() && s.len() == 22 {
                    return true;
                }
            }
            false
        }).count() > 0
    }

    pub fn shapeshift_outbound_address(&self) -> Option<String> {
        if self.chain_type.is_mainnet() {
            self.outputs.iter()
                .find_map(|output| output.script.as_ref()
                    .and_then(|script| address::shapeshift_outbound_for_script(script.clone())))
        } else {
            None
        }
    }

    pub fn shapeshift_outbound_address_force_script(&self) -> Option<String> {
        self.outputs.iter()
            .find_map(|output| output.script.as_ref()
                .and_then(|script| address::shapeshift_outbound_force_script(script.clone())))
    }

    fn amount_sent(&self) -> u64 {
        todo!()
        /*self.inputs.iter().map(|input| {
            if let Some(tx) = self.chain.transaction_for_hash(&input.input_hash) {
                let n = input.index as usize;
                let outputs = tx.outputs();
                if n < outputs.len() {
                    if let Some(output) = outputs.get(n) {
                        if let Some(acc) = self.chain.first_account_that_can_contain_transaction(&tx) {
                            if let Some(address) = &output.address {
                                if acc.contains_address(address) {
                                    return Some(output.amount);
                                }
                            }
                        }
                    }
                }
            }
            None
        }).sum()*/
    }

    // checks if all signatures exist, but does not verify them
    pub fn is_signed(&self) -> bool {
        self.inputs.iter().rfind(|input| input.signature.is_none()).is_none()
    }
}

impl Transaction {
    pub fn add_input_hash(&mut self, input_hash: UInt256, index: u32, script: Option<Vec<u8>>) {
        self.add_input_hash_with_signature(input_hash, index, script, None, TXIN_SEQUENCE)
    }

    pub fn add_input_hash_with_signature(&mut self, input_hash: UInt256, index: u32, script: Option<Vec<u8>>, signature: Option<Vec<u8>>, sequence: u32) {
        self.inputs.push(TransactionInput {
            input_hash,
            index,
            script,
            signature,
            sequence
        });
    }

    pub fn add_output_address(&mut self, address: String, amount: u64) {
        // todo: check this is equivalent and no need to recalculate address with addressWithScriptPubKey

        self.outputs.push(TransactionOutput {
            amount,
            script: Some(Vec::<u8>::script_pub_key_for_address(&address, &self.chain_type.script_map())),
            address: Some(address)
        });
    }

    pub fn add_output_credit_address(&mut self, address: String, amount: u64) {
        let script_map = self.chain_type.script_map();
        self.outputs.push(
            TransactionOutput::from_script(
                amount,
                Vec::<u8>::script_pub_key_for_address(&address, &script_map),
                &script_map))
    }

    pub fn add_output_shapeshift_address(&mut self, address: String) {
        self.outputs.push(
            TransactionOutput::from_script(
                0,
                Vec::<u8>::shapeshift_memo_for_address(address),
                &self.chain_type.script_map()))
    }

    pub fn add_output_burn_amount(&mut self, amount: u64) {
        self.outputs.push(
            TransactionOutput::from_script(
                amount,
                vec![OP_RETURN.into_u8()],
                &self.chain_type.script_map()));
    }
    pub fn add_output_script(&mut self, script: &Vec<u8>, amount: u64) {
        self.add_output_script_with_address(Some(script.clone()), address::with_script_pub_key(script, &self.chain_type.script_map()), amount)
    }

    pub fn add_output_script_with_address(&mut self, script: Option<Vec<u8>>, address: Option<String>, amount: u64) {
        self.outputs.push(
            TransactionOutput::new(
                amount,
                script.clone(),
                address.or_else(||
                    script.and_then(|script|
                        address::with_script_pub_key(&script, &self.chain_type.script_map())))));
    }

    pub fn set_input_address(&mut self, address: String, index: usize) {
        match self.inputs.get_mut(index) {
            Some(input) =>
                input.script = Some(Vec::<u8>::script_pub_key_for_address(&address, &self.chain_type.script_map())),
            _ => {}
        }
        // self.inputs[index].script = Some(Vec::<u8>::script_pub_key_for_address(&address, &self.chain_type.script_map()));
    }
}

impl Transaction {
    /// fischer-yates shuffle
    pub fn shuffle_output_order(&mut self) {
        // fischer-yates shuffle
        for i in 0..self.outputs.len() {
            let j = thread_rng().gen_range(i..self.outputs.len() - i);
            if i == j {
                continue;
            }
            self.outputs.swap(i, j);
        }
    }

    /**
     * Hashes (in reversed byte-order) are to be sorted in ASC order, lexicographically.
     * If they're match -> the respective indices will be compared, in ASC.
     */
    pub fn sort_inputs_according_to_bip69(&mut self) {
        self.inputs.sort_by(|i1, i2| match i1.input_hash.cmp(&i2.input_hash) {
            Ordering::Equal => match i1.index.cmp(&i2.index) {
                Ordering::Greater => Ordering::Less,
                Ordering::Less => Ordering::Greater,
                Ordering::Equal => Ordering::Equal
            },
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less
        })
    }

    /**
     * Amounts are to be sorted in ASC.
     * If they're equal -> respective outScripts will be compared lexicographically, in ASC.
     */
    pub fn sort_outputs_according_to_bip69(&mut self) {
        self.outputs.sort_by(|o1, o2| match o1.amount.cmp(&o2.amount) {
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => match (&o1.script, &o2.script) {
                (Some(script1), Some(script2)) => match script1.cmp(script2) {
                    Ordering::Equal => match script1.len().cmp(&script2.len()) {
                        Ordering::Equal => Ordering::Equal,
                        Ordering::Less => Ordering::Greater,
                        Ordering::Greater => Ordering::Less
                    },
                    Ordering::Less => Ordering::Greater,
                    Ordering::Greater => Ordering::Less
                },
                (None, None) => Ordering::Equal,
                (Some(..), None) => Ordering::Greater,
                (None, Some(..)) => Ordering::Less
            }
        });
    }
}

impl Transaction {

    /// Signing

    pub fn sign_with_serialized_private_keys(&mut self, keys: Vec<&String>) -> bool {
        self.sign_with_private_keys(
            keys.iter()
                .filter_map(|&private_key_string|
                    ECDSAKey::key_with_private_key(private_key_string, self.chain_type()))
                .collect())
    }


    pub fn sign_with_preordered_private_keys(&mut self, keys: Vec<Key>) -> bool {
        todo!()
        // for (i, input) in self.inputs.iter_mut().enumerate() {
        //     let mut sig = Vec::<u8>::new();
        //     let data = self.to_data_with_subscript_index(Some(i as u64));
        //     let hash = UInt256::sha256d(&data);
        //     if let Some(key) = keys.get(i) {
        //         let mut s = key.sign(&hash.0.to_vec());
        //         if let Some(input_script) = &input.script {
        //             let elem = input_script.script_elements();
        //             (SIGHASH_ALL as u8).enc(&mut s);
        //             sig.append_script_push_data(s);
        //             // sig.append_script_push_data(&mut s);
        //             if elem.len() >= 2 {
        //                 if let ScriptElement::Data([0x88 /*OP_EQUALVERIFY*/, ..], ..) = elem[elem.len() - 2] {
        //                     // pay-to-pubkey-hash scriptSig
        //                     sig.append_script_push_data(key.public_key_data());
        //                 }
        //             }
        //             input.signature = Some(sig);
        //         }
        //     }
        // }
        // for (NSUInteger i = 0; i < self.mInputs.count; i++) {
        //     DSTransactionInput *transactionInput = self.mInputs[i];
        //     NSMutableData *sig = [NSMutableData data];
        //     NSData *data = [self toDataWithSubscriptIndex:i];
        //     UInt256 hash = data.SHA256_2;
        //     NSMutableData *s = [NSMutableData dataWithData:[keys[i] sign:hash]];
        //     NSArray *elem = [transactionInput.inScript scriptElements];
        //
        //     [s appendUInt8:SIGHASH_ALL];
        //     [sig appendScriptPushData:s];
        //
        //     if (elem.count >= 2 && [elem[elem.count - 2] intValue] == OP_EQUALVERIFY) { // pay-to-pubkey-hash scriptSig
        //         [sig appendScriptPushData:[keys[i] publicKeyData]];
        //     }
        //
        //     transactionInput.signature = sig;
        // }
        //
        // if !self.is_signed() {
        //     return false;
        // }
        // self.tx_hash = UInt256::sha256d(&self.to_data());
        // true
    }

    pub fn sign_with_private_keys_using_addresses(&mut self, keys: Vec<Key>, addresses: Vec<String>) -> bool {
        // let script_map = self.chain_type().script_map();
        // let version = self.version;
        // let tx_type = self.tx_type;
        // let inputs = &mut self.inputs;
        // let outputs = self.outputs.as_ref();
        // let lock_time = self.lock_time;
        // for (i, input) in inputs.iter_mut().enumerate() {
        //     if let Some(tx_input_script) = &input.script {
        //         if let Some(addr) = Address::with_script_pub_key(tx_input_script, &script_map) {
        //             if let Some(key_idx) = addresses.iter().position(|a| *a == addr) {
        //                 if let Some(key) = keys.get(key_idx) {
        //                     let data = Self::data_with_subscript_index_static(Some(key_idx as u64), version, tx_type, inputs, outputs, lock_time);
        //                     input.signature = Some(key.create_signature(tx_input_script, &data));
        //                 }
        //             }
        //         }
        //     }
        // }
        let script_map = self.chain_type().script_map();
        let version = self.version;
        let tx_type = self.tx_type;
        let inputs = self.inputs.clone();
        let outputs = self.outputs.as_ref();
        let lock_time = self.lock_time;
        for (i, input) in self.inputs.iter_mut().enumerate() {
            if let Some(tx_input_script) = &input.script {
                if let Some(addr) = address::with_script_pub_key(tx_input_script, &script_map) {
                    if let Some(key_idx) = addresses.iter().position(|a| *a == addr) {
                        if let Some(key) = keys.get(key_idx) {
                            let data = Self::data_with_subscript_index_static(Some(key_idx as u64), version, tx_type, &inputs, outputs, lock_time);
                            input.signature = Some(key.create_signature(tx_input_script, &data));
                        }
                    }
                }
            }
        }
        if !self.is_signed() {
            return false;
        }
        self.tx_hash = UInt256::sha256d(self.to_data());
        true
    }

}

/// Priority (Deprecated)
impl Transaction {
    // priority = sum(input_amount_in_satoshis*input_age_in_blocks)/size_in_bytes
    pub fn priority_for_amounts(&self, amounts: Vec<u64>, ages: Vec<u64>) -> u64 {
        let mut p = 0u64;
        if amounts.len() != self.inputs.len() || ages.len() != self.inputs.len() || ages.contains(&0) {
            return 0;
        }
        for i in 0..amounts.len() {
            p += amounts[i] * ages[i];
        }
        p / self.size() as u64
    }

}

/// Fees
impl Transaction {
    // returns the fee for the given transaction if all its inputs are from wallet transactions, u64::MAX otherwise
    pub fn fee_used(&self) -> u64 {
        //TODO: This most likely does not work when sending from multiple accounts
        todo!()
        // self.first_account().unwrap().fee_for_transaction(self)
    }

    pub fn rounded_fee_cost_per_byte(&self) -> u64 {
        let fee_used = self.fee_used();
        if fee_used == u64::MAX {
            return u64::MAX;
        }
        (fee_used as f64 / self.size() as f64).round() as u64
    }

}
/// Confirmation
impl Transaction {
    pub fn confirmations(&mut self) -> u32 {
        todo!()
        // if self.block_height == TX_UNCONFIRMED as u32 {
        //     return 0;
        // }
        // self.chain.last_terminal_block_height() - self.block_height
    }

    pub fn confirmed(&mut self) -> bool {
        todo!()
        // if self.confirmed {
        //     // because it can't be unconfirmed
        //     return true;
        // }
        // if self.block_height == TX_UNCONFIRMED as u32 {
        //     return false;
        // }
        // let last_height = self.chain().last_sync_block_height;
        // if self.block_height > last_height {
        //     // this should only be possible if and only if we have migrated and kept old transactions.
        //     return true;
        // }
        // if last_height - self.block_height > 6 {
        //     return true;
        // }
        // self.confirmed = self.chain().block_height_chain_locked(self.block_height);
        // self.confirmed
    }
}

impl Transaction {
    pub fn devnet_genesis_coinbase_with_identifier(devnet_type: DevnetType, protocol_version: u32, amount: u64, script_map: &ScriptMap) -> UInt256 {
        let script = OP_RETURN.into_u8().to_le_bytes().to_vec();
        UInt256::sha256d(Self::data_with_subscript_index_static(
            None,
            TX_VERSION as u16,
            TransactionType::Classic,
            &[TransactionInput::coinbase(devnet_type, protocol_version)],
            &[TransactionOutput::new(amount, Some(script.clone()), address::with_script_pub_key(&script, script_map))],
            TX_LOCKTIME,
        ))
    }
}


impl Transaction {
    pub fn init_on_chain(chain_type: ChainType, chain: Shared<Chain>) -> Self {
        Self {
            chain_type,
            chain,
            version: TX_VERSION as u16,
            lock_time: TX_LOCKTIME,
            block_height: TX_UNCONFIRMED as u32,
            ..Default::default()
        }
    }

    pub fn init_with(
        version: u16,
        lock_time: u32,
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        tx_hash: UInt256,
        block_height: u32,
        timestamp: u64,
        // associated_shapeshift: Option<ShapeshiftEntity>,
        instant_send_lock: Option<Shared<InstantSendLock>>) -> Self {
        let s = Self {
            inputs,
            outputs,
            lock_time,
            version,
            timestamp,
            tx_hash,
            block_height,
            instant_send_lock_awaiting_processing: instant_send_lock,
            // associated_shapeshift,
            persistence_status: TransactionPersistenceStatus::Saved,
            ..Default::default()
        };
        // s.set_instant_send_received_with_instant_send_lock(instant_send_lock);
        // if let Some(lock) = instant_send_lock {
        //     let is_signature_verified = lock.signature_verified;
        //     self.instant_send_received = is_signature_verified;
        //     self.has_unverified_instant_send_lock = !is_signature_verified;
        //     if is_signature_verified {
        //         self.instant_send_lock_awaiting_processing = instant_send_lock;
        //     }
        //     if !lock.saved {
        //         lock.save_initial();
        //     }
        // }

        s
    }


    pub fn data_with_subscript_index_static(
        subscript_index: Option<u64>,
        version: u16,
        tx_type: TransactionType,
        inputs: &[TransactionInput],
        outputs: &[TransactionOutput],
        lock_time: u32,
    ) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let offset: &mut usize = &mut 0;
        let inputs_len = inputs.len();
        let outputs_len = outputs.len();
        let for_sig_hash = (tx_type == TransactionType::Classic || tx_type == TransactionType::CreditFunding) && subscript_index.is_some();
        *offset += version.enc(&mut buffer);
        *offset += tx_type.raw_value().enc(&mut buffer);
        *offset += VarInt(inputs_len as u64).enc(&mut buffer);
        (0..inputs_len).into_iter().for_each(|i| {
            let input = &inputs[i];
            *offset += input.input_hash.enc(&mut buffer);
            *offset += input.index.enc(&mut buffer);
            if subscript_index.is_none() && input.signature.is_some() {
                *offset += input
                    .signature
                    .as_ref()
                    .unwrap()
                    .enc(&mut buffer)
            } else if subscript_index.is_some() && subscript_index.unwrap() == i as u64 && input.script.is_some() {
                *offset += input
                    .script
                    .as_ref()
                    .unwrap()
                    .enc(&mut buffer)
            } else {
                *offset += VarInt(0_u64).enc(&mut buffer);
            }
            *offset += input.sequence.enc(&mut buffer);
        });
        *offset += VarInt(outputs_len as u64)
            .enc(&mut buffer);
        (0..outputs_len).into_iter().for_each(|i| {
            let output = &outputs[i];
            *offset += output.amount.enc(&mut buffer);
            if let Some(script) = &output.script {
                *offset += script.enc(&mut buffer)
            }
        });
        *offset += lock_time.enc(&mut buffer);
        if for_sig_hash {
            *offset += SIGHASH_ALL.enc(&mut buffer);
        }
        buffer
    }
}
