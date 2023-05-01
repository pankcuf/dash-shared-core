use std::collections::BTreeMap;
use std::ptr::null_mut;
use crate::{common, models, tx, types};
use crate::chain::common::LLMQType;
use crate::crypto::UInt256;
use crate::ffi::boxer::{boxed, boxed_vec};
use crate::ffi::from::FromFFI;

pub trait ToFFI {
    type Item: FromFFI;
    fn encode(&self) -> Self::Item;
}
impl ToFFI for tx::TransactionInput {
    type Item = types::TransactionInput;

    fn encode(&self) -> Self::Item {
        let (script, script_length) = match &self.script {
            Some(data) => (boxed_vec(data.clone()), data.len()),
            None => (null_mut(), 0),
        };
        let (signature, signature_length) = match &self.signature {
            Some(data) => (boxed_vec(data.clone()), data.len()),
            None => (null_mut(), 0),
        };
        Self::Item {
            input_hash: boxed(self.input_hash.0),
            index: self.index,
            script,
            script_length,
            signature,
            signature_length,
            sequence: self.sequence,
        }
    }
}

impl ToFFI for tx::TransactionOutput {
    type Item = types::TransactionOutput;

    fn encode(&self) -> Self::Item {
        let (script, script_length) = match &self.script {
            Some(data) => (boxed_vec(data.clone()), data.len()),
            None => (null_mut(), 0),
        };
        let (address, address_length) = match &self.address {
            Some(data) => (boxed_vec(data.clone()), data.len()),
            None => (null_mut(), 0),
        };
        Self::Item {
            amount: self.amount,
            script,
            script_length,
            address,
            address_length,
        }
    }
}

impl ToFFI for tx::Transaction {
    type Item = types::Transaction;

    fn encode(&self) -> Self::Item {
        Self::Item {
            inputs: boxed_vec(
                self.inputs
                    .iter()
                    .map(|input| boxed(input.encode()))
                    .collect(),
            ),
            inputs_count: self.inputs.len(),
            outputs: boxed_vec(
                self.outputs
                    .iter()
                    .map(|output| boxed(output.encode()))
                    .collect(),
            ),
            outputs_count: self.outputs.len(),
            lock_time: self.lock_time,
            version: self.version,
            tx_hash: if self.tx_hash.is_none() {
                null_mut()
            } else {
                boxed(self.tx_hash.unwrap().0)
            },
            tx_type: self.tx_type,
            payload_offset: self.payload_offset,
            block_height: self.block_height,
        }
    }
}
impl ToFFI for tx::CoinbaseTransaction {
    type Item = types::CoinbaseTransaction;

    fn encode(&self) -> Self::Item {
        Self::Item {
            base: boxed(self.base.encode()),
            coinbase_transaction_version: self.coinbase_transaction_version,
            height: self.height,
            merkle_root_mn_list: boxed(self.merkle_root_mn_list.0),
            merkle_root_llmq_list: if self.merkle_root_llmq_list.is_none() {
                null_mut()
            } else {
                boxed(self.merkle_root_llmq_list.unwrap().0)
            },
            locked_amount: self.locked_amount
        }
    }
}

impl ToFFI for models::MasternodeList {
    type Item = types::MasternodeList;

    fn encode(&self) -> Self::Item {
        Self::Item {
            block_hash: boxed(self.block_hash.0),
            known_height: self.known_height,
            masternode_merkle_root: if self.masternode_merkle_root.is_none() {
                null_mut()
            } else {
                boxed(self.masternode_merkle_root.unwrap().0)
            },
            llmq_merkle_root: if self.llmq_merkle_root.is_none() {
                null_mut()
            } else {
                boxed(self.llmq_merkle_root.unwrap().0)
            },
            masternodes: encode_masternodes_map(&self.masternodes),
            masternodes_count: self.masternodes.len(),
            llmq_type_maps: encode_quorums_map(&self.quorums),
            llmq_type_maps_count: self.quorums.len(),
        }
    }
}

impl ToFFI for models::MasternodeEntry {
    type Item = types::MasternodeEntry;

    fn encode(&self) -> Self::Item {
        let previous_operator_public_keys_count = self.previous_operator_public_keys.len();
        let previous_entry_hashes_count = self.previous_entry_hashes.len();
        let previous_validity_count = self.previous_validity.len();
        let confirmed_hash = boxed(self.confirmed_hash.0);
        let confirmed_hash_hashed_with_provider_registration_transaction_hash = if self
            .confirmed_hash_hashed_with_provider_registration_transaction_hash
            .is_none()
        {
            null_mut()
        } else {
            boxed(
                self.confirmed_hash_hashed_with_provider_registration_transaction_hash
                    .unwrap()
                    .0,
            )
        };
        let key_id_voting = boxed(self.key_id_voting.0);
        let known_confirmed_at_height = self.known_confirmed_at_height.unwrap_or(0);
        let entry_hash = boxed(self.entry_hash.0);
        let operator_public_key = boxed(self.operator_public_key.encode());
        let previous_operator_public_keys = boxed_vec(
            self.previous_operator_public_keys
                .iter()
                .map(
                    |(
                        &common::Block {
                            hash,
                            height: block_height,
                        },
                        &key,
                    )| types::BlockOperatorPublicKey {
                        block_hash: hash.0,
                        block_height,
                        key: key.data.0,
                        version: key.version
                    },
                )
                .collect(),
        );
        let previous_entry_hashes = boxed_vec(
            self.previous_entry_hashes
                .iter()
                .map(|(&common::Block { hash: block_hash, height: block_height }, &hash)| types::MasternodeEntryHash { block_hash: block_hash.0, block_height, hash: hash.0 })
                .collect(),
        );
        let previous_validity = boxed_vec(
            self.previous_validity
                .iter()
                .map(
                    |(
                        &common::Block {
                            hash,
                            height: block_height,
                        },
                        &is_valid,
                    )| types::Validity {
                        block_hash: hash.0,
                        block_height,
                        is_valid,
                    },
                )
                .collect(),
        );
        let provider_registration_transaction_hash =
            boxed(self.provider_registration_transaction_hash.0);
        let ip_address = boxed(self.socket_address.ip_address.0);
        let port = self.socket_address.port;
        let is_valid = self.is_valid;
        let update_height = self.update_height;
        let mn_type: u16 = self.mn_type.into();
        let platform_http_port = self.platform_http_port;
        let platform_node_id = boxed(self.platform_node_id.0);
        Self::Item {
            confirmed_hash,
            confirmed_hash_hashed_with_provider_registration_transaction_hash,
            is_valid,
            key_id_voting,
            known_confirmed_at_height,
            entry_hash,
            operator_public_key,
            previous_operator_public_keys,
            previous_operator_public_keys_count,
            previous_entry_hashes,
            previous_entry_hashes_count,
            previous_validity,
            previous_validity_count,
            provider_registration_transaction_hash,
            ip_address,
            port,
            update_height,
            mn_type,
            platform_http_port,
            platform_node_id
        }
    }
}

impl ToFFI for models::LLMQEntry {
    type Item = types::LLMQEntry;

    fn encode(&self) -> Self::Item {
        let all_commitment_aggregated_signature = boxed(self.all_commitment_aggregated_signature.0);
        let commitment_hash = if self.commitment_hash.is_none() {
            null_mut()
        } else {
            boxed(self.commitment_hash.unwrap().0)
        };
        let llmq_type = self.llmq_type;
        let entry_hash = boxed(self.entry_hash.0);
        let llmq_hash = boxed(self.llmq_hash.0);
        let public_key = boxed(self.public_key.0);
        let threshold_signature = boxed(self.threshold_signature.0);
        let verification_vector_hash = boxed(self.verification_vector_hash.0);
        let index = self.index.unwrap_or(0);
        let saved = self.saved;
        let verified = self.verified;
        let version = self.version;
        let signers_count = self.signers_count.0;
        let valid_members_count = self.valid_members_count.0;
        let signers_bitset = boxed_vec(self.signers_bitset.clone());
        let signers_bitset_length = self.signers_bitset.len();
        let valid_members_bitset = boxed_vec(self.valid_members_bitset.clone());
        let valid_members_bitset_length = self.valid_members_bitset.len();
        Self::Item {
            all_commitment_aggregated_signature,
            commitment_hash,
            llmq_type,
            entry_hash,
            llmq_hash,
            index,
            public_key,
            threshold_signature,
            verification_vector_hash,
            saved,
            signers_bitset,
            signers_bitset_length,
            signers_count,
            valid_members_bitset,
            valid_members_bitset_length,
            valid_members_count,
            verified,
            version,
        }
    }
}

impl ToFFI for models::LLMQSnapshot {
    type Item = types::LLMQSnapshot;

    fn encode(&self) -> Self::Item {
        Self::Item {
            member_list_length: self.member_list.len(),
            member_list: boxed_vec(self.member_list.clone()),
            skip_list_length: self.skip_list.len(),
            skip_list: boxed_vec(self.skip_list.clone()),
            skip_list_mode: self.skip_list_mode,
        }
    }
}

impl ToFFI for models::OperatorPublicKey {
    type Item = types::OperatorPublicKey;

    fn encode(&self) -> Self::Item {
        Self::Item { data: self.data.0, version: self.version }
    }
}

impl ToFFI for common::Block {
    type Item = types::Block;

    fn encode(&self) -> Self::Item {
        Self::Item {
            height: self.height,
            hash: boxed(self.hash.0),
        }
    }
}

pub fn encode_quorums_map(
    quorums: &BTreeMap<LLMQType, BTreeMap<UInt256, models::LLMQEntry>>,
) -> *mut *mut types::LLMQMap {
    boxed_vec(
        quorums
            .iter()
            .map(|(&llmq_type, map)| {
                boxed(types::LLMQMap {
                    llmq_type: llmq_type.into(),
                    values: boxed_vec(
                        (*map)
                            .iter()
                            .map(|(_, entry)| boxed(entry.encode()))
                            .collect(),
                    ),
                    count: (*map).len(),
                })
            })
            .collect(),
    )
}

pub fn encode_masternodes_map(
    masternodes: &BTreeMap<UInt256, models::MasternodeEntry>,
) -> *mut *mut types::MasternodeEntry {
    boxed_vec(
        masternodes
            .iter()
            .map(|(_, entry)| boxed(entry.encode()))
            .collect(),
    )
}
