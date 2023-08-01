use byte::{BytesExt, TryRead};
use std::collections::BTreeMap;
#[cfg(feature = "generate-dashj-tests")]
use serde::{Serialize, Serializer};
#[cfg(feature = "generate-dashj-tests")]
use serde::ser::SerializeStruct;
use crate::chain::constants::CORE_PROTO_19_2;
use crate::consensus::Encodable;
use crate::crypto::{UInt160, UInt256, byte_util::Zeroable};
use crate::util::data_ops::short_hex_string_from;

// (block height, list diff version (2: BLSBasic), protocol_version)
#[derive(Clone, Copy)]
pub struct MasternodeReadContext(pub u32, pub u16, pub u32);

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub struct MasternodeEntry {
    pub provider_registration_transaction_hash: UInt256,
    pub confirmed_hash: UInt256,
    pub confirmed_hash_hashed_with_provider_registration_transaction_hash: Option<UInt256>,
    // TODO: migrate on SocketAddr
    pub socket_address: crate::common::socket_address::SocketAddress,
    pub operator_public_key: crate::models::operator_public_key::OperatorPublicKey,
    pub previous_operator_public_keys: BTreeMap<crate::common::block::Block, crate::models::operator_public_key::OperatorPublicKey>,
    pub previous_entry_hashes: BTreeMap<crate::common::block::Block, UInt256>,
    pub previous_validity: BTreeMap<crate::common::block::Block, bool>,
    pub known_confirmed_at_height: Option<u32>,
    pub update_height: u32,
    pub key_id_voting: UInt160,
    pub is_valid: bool,
    pub mn_type: crate::common::masternode_type::MasternodeType,
    pub platform_http_port: u16,
    pub platform_node_id: UInt160,
    pub entry_hash: UInt256,
}

#[cfg(feature = "generate-dashj-tests")]
impl Serialize for MasternodeEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("MasternodeEntry", 10)?;
        state.serialize_field("provider_registration_transaction_hash", &self.provider_registration_transaction_hash)?;
        state.serialize_field("confirmed_hash", &self.confirmed_hash)?;
        state.serialize_field("ip_address", &self.socket_address.ip_address)?;
        state.serialize_field("port", &self.socket_address.port)?;
        state.serialize_field("operator_public_key", &self.operator_public_key.data)?;
        state.serialize_field("key_id_voting", &self.key_id_voting)?;
        state.serialize_field("is_valid", &self.is_valid)?;
        state.serialize_field("mn_type", &self.mn_type)?;
        state.serialize_field("platform_http_port", &self.platform_http_port)?;
        state.serialize_field("platform_node_id", &self.platform_node_id)?;
        state.serialize_field("version", &self.version)?;
        state.end()

    }
}

// Define a wrapper struct for the BTreeMap.
pub struct CustomDebugBTreeMap<K, V>(pub BTreeMap<K, V>);
impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for CustomDebugBTreeMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Customize the debug representation here.
        write!(f, "BTreeMap::from([")?;
        for (key, value) in &self.0 {
            write!(f, "({:?}, {:?}), ", key, value)?;
        }
        write!(f, "])")
    }
}

impl std::fmt::Debug for MasternodeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prev_e = CustomDebugBTreeMap(self.previous_entry_hashes.clone());
        let prev_v = CustomDebugBTreeMap(self.previous_validity.clone());
        let prev_o = CustomDebugBTreeMap(self.previous_operator_public_keys.clone());
        f.debug_struct("MasternodeEntry")
            .field("provider_registration_transaction_hash", &self.provider_registration_transaction_hash)
            .field("confirmed_hash", &self.confirmed_hash)
            .field("confirmed_hash_hashed_with_provider_registration_transaction_hash", &self.confirmed_hash_hashed_with_provider_registration_transaction_hash)
            .field("socket_address", &self.socket_address)
            .field("operator_public_key", &self.operator_public_key.data)
            .field("version", &self.operator_public_key.version)
            .field("previous_operator_public_keys", &prev_o)
            .field("previous_entry_hashes", &prev_e)
            .field("previous_validity", &prev_v)
            .field("known_confirmed_at_height", &self.known_confirmed_at_height)
            .field("update_height", &self.update_height)
            .field("key_id_voting", &self.key_id_voting)
            .field("is_valid", &self.is_valid)
            .field("mn_type", &self.mn_type)
            .field("platform_http_port", &self.platform_http_port)
            .field("platform_node_id", &self.platform_node_id)
            .field("entry_hash", &self.entry_hash)
            .finish()
    }
}

impl<'a> TryRead<'a, MasternodeReadContext> for MasternodeEntry {
    fn try_read(bytes: &'a [u8], context: MasternodeReadContext) -> byte::Result<(Self, usize)> {
        let MasternodeReadContext (block_height, diff_version, protocol_version) = context;
        let offset = &mut 0;
        let version = if protocol_version >= CORE_PROTO_19_2 {
            bytes.read_with::<u16>(offset, byte::LE)?
        } else {
            1 // legacy
        };
        let provider_registration_transaction_hash =
            bytes.read_with::<UInt256>(offset, byte::LE)?;
        let confirmed_hash = bytes.read_with::<UInt256>(offset, byte::LE)?;
        let socket_address = bytes.read_with::<crate::common::SocketAddress>(offset, ())?;
        let operator_public_key = bytes.read_with::<crate::models::OperatorPublicKey>(offset, (version, protocol_version))?;
        let key_id_voting = bytes.read_with::<UInt160>(offset, byte::LE)?;
        let is_valid = bytes.read_with::<u8>(offset, byte::LE)
            .unwrap_or(0);
         let mn_type = if version >= 2 {
            bytes.read_with::<crate::common::MasternodeType>(offset, byte::LE)?
        } else {
             crate::common::masternode_type::MasternodeType::Regular
        };
        let (platform_http_port, platform_node_id) = if mn_type == crate::common::MasternodeType::HighPerformance {
            (bytes.read_with::<u16>(offset, byte::BE)?,
             bytes.read_with::<UInt160>(offset, byte::LE)?)
        } else {
            (0u16, UInt160::MIN)
        };
        let mut entry = Self::new(
            version,
            provider_registration_transaction_hash,
            confirmed_hash,
            socket_address,
            key_id_voting,
            operator_public_key,
            is_valid,
            mn_type,
            platform_http_port,
            platform_node_id,
            block_height,
            protocol_version
        );
        if !entry.confirmed_hash.is_zero() && block_height != u32::MAX {
            entry.known_confirmed_at_height = Some(block_height);
        }
        Ok((entry, *offset))
    }
}

impl MasternodeEntry {
    pub fn new(
        version: u16,
        provider_registration_transaction_hash: UInt256,
        confirmed_hash: UInt256,
        socket_address: crate::common::SocketAddress,
        key_id_voting: UInt160,
        operator_public_key: crate::models::OperatorPublicKey,
        is_valid: u8,
        mn_type: crate::common::MasternodeType,
        platform_http_port: u16,
        platform_node_id: UInt160,
        update_height: u32,
        protocol_version: u32,
    ) -> Self {
        let entry_hash = Self::calculate_entry_hash(
            version,
            provider_registration_transaction_hash,
            confirmed_hash,
            socket_address,
            operator_public_key,
            key_id_voting,
            is_valid,
            mn_type,
            platform_http_port,
            platform_node_id,
            protocol_version,
        );
        Self {
            provider_registration_transaction_hash,
            confirmed_hash,
            confirmed_hash_hashed_with_provider_registration_transaction_hash: Some(
                Self::hash_confirmed_hash(confirmed_hash, provider_registration_transaction_hash),
            ),
            socket_address,
            operator_public_key,
            previous_operator_public_keys: Default::default(),
            previous_entry_hashes: Default::default(),
            previous_validity: Default::default(),
            known_confirmed_at_height: None,
            update_height,
            key_id_voting,
            is_valid: is_valid != 0,
            mn_type,
            platform_http_port,
            platform_node_id,
            entry_hash,
        }
    }

    pub fn calculate_entry_hash(
        version: u16,
        provider_registration_transaction_hash: UInt256,
        confirmed_hash: UInt256,
        socket_address: crate::common::SocketAddress,
        operator_public_key: crate::models::OperatorPublicKey,
        key_id_voting: UInt160,
        is_valid: u8,
        mn_type: crate::common::MasternodeType,
        platform_http_port: u16,
        platform_node_id: UInt160,
        protocol_version: u32,
    ) -> UInt256 {
        let mut writer = Vec::<u8>::new();
        provider_registration_transaction_hash.enc(&mut writer);
        confirmed_hash.enc(&mut writer);
        socket_address.enc(&mut writer);
        operator_public_key.enc(&mut writer);
        key_id_voting.enc(&mut writer);
        is_valid.enc(&mut writer);
        if version >= 2 {
            u16::from(mn_type).enc(&mut writer);
            if mn_type == crate::common::MasternodeType::HighPerformance {
                platform_http_port.swap_bytes().enc(&mut writer);
                platform_node_id.enc(&mut writer);
            }
        }
        UInt256::sha256d(writer)
    }

    pub fn confirmed_hash_at(&self, block_height: u32) -> Option<UInt256> {
        self.known_confirmed_at_height
            .and_then(|h| (h <= block_height)
                .then_some(self.confirmed_hash))
    }

    pub fn update_confirmed_hash(&mut self, hash: UInt256) {
        self.confirmed_hash = hash;
        if !self.provider_registration_transaction_hash.is_zero() {
            self.update_confirmed_hash_hashed_with_pro_reg_tx_hash();
        }
    }

    pub fn update_confirmed_hash_hashed_with_pro_reg_tx_hash(&mut self) {
        let hash = Self::hash_confirmed_hash(self.confirmed_hash, self.provider_registration_transaction_hash);
        self.confirmed_hash_hashed_with_provider_registration_transaction_hash = Some(hash)
    }

    pub fn confirmed_hash_hashed_with_pro_reg_tx_hash_at(
        &self,
        block_height: u32,
    ) -> Option<UInt256> {
        if self.known_confirmed_at_height.is_none() || self.known_confirmed_at_height? <= block_height {
            self.confirmed_hash_hashed_with_provider_registration_transaction_hash
        } else {
            Some(Self::hash_confirmed_hash(
                UInt256::default(),
                self.provider_registration_transaction_hash,
            ))
        }
    }

    pub fn host(&self) -> String {
        format!("{}", self.socket_address)
    }

    /*pub fn payload_data(&self) -> UInt256 {
        Self::calculate_entry_hash(
            self.provider_registration_transaction_hash,
            self.confirmed_hash,
            self.socket_address,
            self.operator_public_key,
            self.key_id_voting,
            u8::from(self.is_valid),
            self.mn_type,
            self.platform_http_port,
            self.platform_node_id,
        )
    }*/

    pub fn hash_confirmed_hash(confirmed_hash: UInt256, provider_registration_transaction_hash: UInt256) -> UInt256 {
        UInt256::sha256(&[provider_registration_transaction_hash.0, confirmed_hash.0].concat())
    }

    pub fn is_valid_at(&self, block_height: u32) -> bool {
        if self.previous_validity.is_empty() || block_height == u32::MAX {
            return self.is_valid;
        }
        let mut min_distance = u32::MAX;
        let mut is_valid = self.is_valid;
        for (&crate::common::Block { height, .. }, &validity) in &self.previous_validity {
            if height <= block_height {
                continue;
            }
            let distance = height - block_height;
            if distance < min_distance {
                min_distance = distance;
                is_valid = validity;
            }
        }
        is_valid
    }

    pub fn operator_public_key_at(&self, block_height: u32) -> crate::models::OperatorPublicKey {
        if self.previous_operator_public_keys.is_empty() {
            return self.operator_public_key;
        }
        let mut min_distance = u32::MAX;
        let mut used_previous_operator_public_key_at_block_hash = self.operator_public_key;
        for (&crate::common::Block { height, .. }, &key) in &self.previous_operator_public_keys {
            if height <= block_height {
                continue;
            }
            let distance = height - block_height;
            if distance < min_distance {
                min_distance = distance;
                info!("SME operator public key for proTxHash {:?} : Using {:?} instead of {:?} for list at block height {block_height}", key, used_previous_operator_public_key_at_block_hash, self.provider_registration_transaction_hash);
                used_previous_operator_public_key_at_block_hash = key;
            }
        }
        used_previous_operator_public_key_at_block_hash
    }

    pub fn entry_hash_at(&self, block_height: u32) -> UInt256 {
        if self.previous_entry_hashes.is_empty() || block_height == u32::MAX {
            return self.entry_hash;
        }
        let mut min_distance = u32::MAX;
        let mut used_hash = self.entry_hash;
        for (&crate::common::Block { height, .. }, &hash) in &self.previous_entry_hashes {
            if height <= block_height {
                continue;
            }
            let distance = height - block_height;
            if distance < min_distance {
                min_distance = distance;
                info!("SME Hash for proTxHash {:?} : Using {hash} instead of {used_hash} for list at block height {block_height}", self.provider_registration_transaction_hash);
                used_hash = hash;
            }
        }
        used_hash
    }

    pub fn unique_id(&self) -> String {
        short_hex_string_from(&self.provider_registration_transaction_hash.0)
    }

    pub fn update_with_previous_entry(&mut self, entry: &mut MasternodeEntry, block_height: u32, block_hash: UInt256) {
        let block = crate::common::Block::new(block_height, block_hash);
        self.previous_validity = entry
            .previous_validity
            .clone()
            .into_iter()
            .filter(|(block, _)| block.height < self.update_height)
            .collect();
        if entry.is_valid_at(self.update_height) != self.is_valid {
            self.previous_validity.insert(block, entry.is_valid);
        }
        self.previous_operator_public_keys = entry
            .previous_operator_public_keys
            .clone()
            .into_iter()
            .filter(|(block, _)| block.height < self.update_height)
            .collect();
        if entry.operator_public_key_at(self.update_height) != self.operator_public_key {
            self.previous_operator_public_keys.insert(block, entry.operator_public_key);
        }
        let old_prev_mn_entry_hashes = entry
            .previous_entry_hashes
            .clone()
            .into_iter()
            .filter(|(block, _)| block.height < self.update_height)
            .collect();
        self.previous_entry_hashes = old_prev_mn_entry_hashes;
        if entry.entry_hash_at(self.update_height) != self.entry_hash {
            self.previous_entry_hashes.insert(block, entry.entry_hash);
        }
    }
}
