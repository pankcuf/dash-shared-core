use crate::chain::common::LLMQType;
use crate::common::LLMQVersion;
use crate::consensus::{encode::VarInt, Encodable, WriteExt};
use crate::crypto::{byte_util::AsBytes, data_ops::Data, UInt256, UInt384, UInt768};
use crate::keys::BLSKey;
use crate::models;
use byte::{
    ctx::{Bytes, Endian},
    BytesExt, TryRead, LE,
};
use hashes::hex::ToHex;
#[cfg(feature = "generate-dashj-tests")]
use serde::ser::SerializeStruct;
#[cfg(feature = "generate-dashj-tests")]
use serde::{Serialize, Serializer};
use std::convert::Into;

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub struct LLMQEntry {
    pub version: crate::common::llmq_version::LLMQVersion,
    pub llmq_hash: UInt256,
    pub index: Option<u16>,
    pub public_key: UInt384,
    pub threshold_signature: UInt768,
    pub verification_vector_hash: UInt256,
    pub all_commitment_aggregated_signature: UInt768,
    pub llmq_type: crate::chain::common::llmq_type::LLMQType,
    pub signers_bitset: Vec<u8>,
    pub signers_count: crate::consensus::encode::VarInt,
    pub valid_members_bitset: Vec<u8>,
    pub valid_members_count: crate::consensus::encode::VarInt,
    pub entry_hash: UInt256,
    pub verified: bool,
    pub saved: bool,
    pub commitment_hash: Option<UInt256>,
}

#[cfg(feature = "generate-dashj-tests")]
impl Serialize for LLMQEntry {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state =
            serializer.serialize_struct("LLMQEntry", 10 + usize::from(self.index.is_some()))?;
        state.serialize_field("version", &self.version)?;
        if let Some(index) = self.index {
            state.serialize_field("index", &index)?;
        }
        state.serialize_field("public_key", &self.public_key)?;
        state.serialize_field("threshold_signature", &self.threshold_signature)?;
        state.serialize_field("verification_vector_hash", &self.verification_vector_hash)?;
        state.serialize_field(
            "all_commitment_aggregated_signature",
            &self.all_commitment_aggregated_signature,
        )?;
        state.serialize_field("llmq_type", &self.llmq_type)?;
        state.serialize_field("signers_bitset", &self.signers_bitset.to_hex())?;
        state.serialize_field("signers_count", &self.signers_count.0)?;
        state.serialize_field("valid_members_bitset", &self.valid_members_bitset.to_hex())?;
        state.serialize_field("valid_members_count", &self.valid_members_count.0)?;
        state.end()
    }
}

pub struct VecHex(pub Vec<u8>);
impl std::fmt::Debug for VecHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec::from_hex(\"{}\").unwrap()", self.0.to_hex())
    }
}

impl std::fmt::Debug for LLMQEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMQEntry")
            .field("version", &self.version)
            .field("llmq_hash", &self.llmq_hash)
            .field("index", &self.index)
            .field("public_key", &self.public_key)
            .field("threshold_signature", &self.threshold_signature)
            .field("verification_vector_hash", &self.verification_vector_hash)
            .field(
                "all_commitment_aggregated_signature",
                &self.all_commitment_aggregated_signature,
            )
            .field("llmq_type", &self.llmq_type)
            .field("signers_bitset", &VecHex(self.signers_bitset.clone()))
            .field("signers_count", &self.signers_count)
            .field(
                "valid_members_bitset",
                &VecHex(self.valid_members_bitset.clone()),
            )
            .field("valid_members_count", &self.valid_members_count)
            .field("entry_hash", &self.entry_hash)
            .field("verified", &self.verified)
            .field("saved", &self.saved)
            .field("commitment_hash", &self.commitment_hash)
            .finish()
    }
}

impl<'a> TryRead<'a, Endian> for LLMQEntry {
    fn try_read(bytes: &'a [u8], _ctx: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let version = bytes.read_with::<LLMQVersion>(offset, LE)?;
        let llmq_type = bytes.read_with::<LLMQType>(offset, LE)?;
        let llmq_hash = bytes.read_with::<UInt256>(offset, LE)?;
        let index = if version.use_rotated_quorums() {
            Some(bytes.read_with::<u16>(offset, LE)?)
        } else {
            None
        };
        let signers_count = bytes.read_with::<VarInt>(offset, LE)?;
        let signers_buffer_length: usize = ((signers_count.0 as usize) + 7) / 8;
        let signers_bitset: &[u8] = bytes.read_with(offset, Bytes::Len(signers_buffer_length))?;
        let valid_members_count = bytes.read_with::<VarInt>(offset, LE)?;
        let valid_members_count_buffer_length: usize = ((valid_members_count.0 as usize) + 7) / 8;
        let valid_members_bitset: &[u8] =
            bytes.read_with(offset, Bytes::Len(valid_members_count_buffer_length))?;
        let public_key = bytes.read_with::<UInt384>(offset, LE)?;
        let verification_vector_hash = bytes.read_with::<UInt256>(offset, LE)?;
        let threshold_signature = bytes.read_with::<UInt768>(offset, LE)?;
        let all_commitment_aggregated_signature = bytes.read_with::<UInt768>(offset, LE)?;
        let entry = LLMQEntry::new(
            version,
            llmq_type,
            llmq_hash,
            index,
            signers_count,
            valid_members_count,
            signers_bitset.to_vec(),
            valid_members_bitset.to_vec(),
            public_key,
            verification_vector_hash,
            threshold_signature,
            all_commitment_aggregated_signature,
        );
        Ok((entry, *offset))
    }
}

impl LLMQEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: LLMQVersion,
        llmq_type: LLMQType,
        llmq_hash: UInt256,
        index: Option<u16>,
        signers_count: VarInt,
        valid_members_count: VarInt,
        signers_bitset: Vec<u8>,
        valid_members_bitset: Vec<u8>,
        public_key: UInt384,
        verification_vector_hash: UInt256,
        threshold_signature: UInt768,
        all_commitment_aggregated_signature: UInt768,
    ) -> Self {
        let q_data = Self::generate_data(
            version,
            llmq_type,
            llmq_hash,
            index,
            signers_count,
            signers_bitset.as_slice(),
            valid_members_count,
            valid_members_bitset.as_slice(),
            public_key,
            verification_vector_hash,
            threshold_signature,
            all_commitment_aggregated_signature,
        );
        let entry_hash = UInt256::sha256d(q_data);
        //println!("LLMQEntry::new({}, {:?}, {}, {:?}, {}, {}, {}, {}, {}, {}, {}, {}) = {}", version, llmq_type, llmq_hash, index, signers_count, signers_bitset.to_hex(), valid_members_count, valid_members_bitset.to_hex(), public_key, verification_vector_hash, threshold_signature, all_commitment_aggregated_signature, entry_hash);
        Self {
            version,
            llmq_hash,
            index,
            public_key,
            threshold_signature,
            verification_vector_hash,
            all_commitment_aggregated_signature,
            llmq_type,
            signers_bitset,
            signers_count,
            valid_members_bitset,
            valid_members_count,
            entry_hash,
            verified: false,
            saved: false,
            commitment_hash: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn generate_data(
        version: LLMQVersion,
        llmq_type: LLMQType,
        llmq_hash: UInt256,
        llmq_index: Option<u16>,
        signers_count: VarInt,
        signers_bitset: &[u8],
        valid_members_count: VarInt,
        valid_members_bitset: &[u8],
        public_key: UInt384,
        verification_vector_hash: UInt256,
        threshold_signature: UInt768,
        all_commitment_aggregated_signature: UInt768,
    ) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let offset: &mut usize = &mut 0;
        let llmq_u8: u8 = llmq_type.into();
        let llmq_v: u16 = version.into();
        *offset += llmq_v.enc(&mut buffer);
        *offset += llmq_u8.enc(&mut buffer);
        *offset += llmq_hash.enc(&mut buffer);
        if let Some(index) = llmq_index {
            *offset += index.enc(&mut buffer);
        }
        *offset += signers_count.enc(&mut buffer);
        buffer.emit_slice(signers_bitset).unwrap();
        *offset += signers_bitset.len();
        *offset += valid_members_count.enc(&mut buffer);
        buffer.emit_slice(valid_members_bitset).unwrap();
        *offset += valid_members_bitset.len();
        *offset += public_key.enc(&mut buffer);
        *offset += verification_vector_hash.enc(&mut buffer);
        *offset += threshold_signature.enc(&mut buffer);
        *offset += all_commitment_aggregated_signature.enc(&mut buffer);
        buffer
    }

    pub fn to_data(&self) -> Vec<u8> {
        Self::generate_data(
            self.version,
            self.llmq_type,
            self.llmq_hash,
            self.index,
            self.signers_count,
            &self.signers_bitset,
            self.valid_members_count,
            &self.valid_members_bitset,
            self.public_key,
            self.verification_vector_hash,
            self.threshold_signature,
            self.all_commitment_aggregated_signature,
        )
    }

    pub fn build_llmq_quorum_hash(llmq_type: LLMQType, llmq_hash: UInt256) -> UInt256 {
        let mut writer: Vec<u8> = Vec::with_capacity(33);
        VarInt(llmq_type as u64).enc(&mut writer);
        llmq_hash.enc(&mut writer);
        UInt256::sha256d(writer)
    }

    pub fn llmq_quorum_hash(&self) -> UInt256 {
        Self::build_llmq_quorum_hash(self.llmq_type, self.llmq_hash)
    }

    pub fn commitment_data(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let offset: &mut usize = &mut 0;
        let llmq_type = VarInt(self.llmq_type as u64);
        *offset += llmq_type.enc(&mut buffer);
        *offset += self.llmq_hash.enc(&mut buffer);
        *offset += self.valid_members_count.enc(&mut buffer);
        buffer.emit_slice(&self.valid_members_bitset).unwrap();
        *offset += self.valid_members_bitset.len();
        *offset += self.public_key.enc(&mut buffer);
        *offset += self.verification_vector_hash.enc(&mut buffer);
        buffer
    }

    pub fn ordering_hash_for_request_id(
        &self,
        request_id: UInt256,
        llmq_type: LLMQType,
    ) -> UInt256 {
        let llmq_type = VarInt(llmq_type as u64);
        let mut buffer: Vec<u8> = Vec::with_capacity(llmq_type.len() + 64);
        llmq_type.enc(&mut buffer);
        self.llmq_hash.enc(&mut buffer);
        request_id.enc(&mut buffer);
        UInt256::sha256d(buffer)
    }

    pub fn generate_commitment_hash(&mut self) -> UInt256 {
        if self.commitment_hash.is_none() {
            self.commitment_hash = Some(UInt256::sha256d(self.commitment_data()));
        }
        self.commitment_hash.unwrap()
    }

    fn validate_bitset(bitset: Vec<u8>, count: VarInt) -> bool {
        if bitset.len() != (count.0 as usize + 7) / 8 {
            warn!(
                "Error: The byte size of the bitvectors ({}) must match “(quorumSize + 7) / 8 ({})",
                bitset.len(),
                (count.0 + 7) / 8
            );
            return false;
        }
        let len = (bitset.len() * 8) as i32;
        let size = count.0 as i32;
        if len != size {
            let rem = len - size;
            let mask = !(0xff >> rem);
            let last_byte = match bitset.last() {
                Some(&last) => last as i32,
                None => 0,
            };
            if last_byte & mask != 0 {
                warn!("Error: No out-of-range bits should be set in byte representation of the bitvector");
                return false;
            }
        }
        true
    }

    pub fn validate_payload(&self) -> bool {
        // The quorumHash must match the current DKG session
        // todo
        let is_valid_signers =
            Self::validate_bitset(self.signers_bitset.clone(), self.signers_count);
        if !is_valid_signers {
            warn!(
                "Error: signers_bitset is invalid ({:?} {})",
                self.signers_bitset, self.signers_count
            );
            return false;
        }
        let is_valid_members =
            Self::validate_bitset(self.valid_members_bitset.clone(), self.valid_members_count);
        if !is_valid_members {
            warn!(
                "Error: valid_members_bitset is invalid ({:?} {})",
                self.valid_members_bitset, self.valid_members_count
            );
            return false;
        }
        let quorum_threshold = self.llmq_type.threshold() as u64;
        // The number of set bits in the signers and validMembers bitvectors must be at least >= quorumThreshold
        let signers_bitset_true_bits_count = self.signers_bitset.as_slice().true_bits_count();
        if signers_bitset_true_bits_count < quorum_threshold {
            warn!("Error: The number of set bits in the signers bitvector {} must be at least >= quorumThreshold {}", signers_bitset_true_bits_count, quorum_threshold);
            return false;
        }
        let valid_members_bitset_true_bits_count =
            self.valid_members_bitset.as_slice().true_bits_count();
        if valid_members_bitset_true_bits_count < quorum_threshold {
            warn!("Error: The number of set bits in the validMembers bitvector {} must be at least >= quorumThreshold {}", valid_members_bitset_true_bits_count, quorum_threshold);
            return false;
        }
        true
    }
}

impl LLMQEntry {
    pub fn verify(
        &mut self,
        valid_masternodes: Vec<models::MasternodeEntry>,
        block_height: u32,
    ) -> bool {
        if !self.validate_payload() {
            return false;
        }
        self.verified = self.validate(valid_masternodes, block_height);
        self.verified
    }

    pub fn validate(
        &mut self,
        valid_masternodes: Vec<models::MasternodeEntry>,
        block_height: u32,
    ) -> bool {
        let commitment_hash = self.generate_commitment_hash();
        let use_legacy = self.version.use_bls_legacy();

        let operator_keys = valid_masternodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| {
                self.signers_bitset
                    .as_slice()
                    .bit_is_true_at_le_index(i as u32)
                    .then_some(node.operator_public_key_at(block_height))
            })
            .collect::<Vec<_>>();
        // info!("let operator_keys = vec![{:?}];", operator_keys);
        let all_commitment_aggregated_signature_validated = BLSKey::verify_secure_aggregated(
            commitment_hash,
            self.all_commitment_aggregated_signature,
            operator_keys,
            use_legacy,
        );
        if !all_commitment_aggregated_signature_validated {
            // warn!("••• Issue with all_commitment_aggregated_signature_validated: {}", self.all_commitment_aggregated_signature);
            println!(
                "••• INVALID AGGREGATED SIGNATURE {}: {:?} ({})",
                block_height, self.llmq_type, self.all_commitment_aggregated_signature
            );
            return false;
        }
        // The sig must validate against the commitmentHash and all public keys determined by the signers bitvector.
        // This is an aggregated BLS signature verification.
        let quorum_signature_validated = BLSKey::verify_quorum_signature(
            commitment_hash.as_bytes(),
            self.threshold_signature.as_bytes(),
            self.public_key.as_bytes(),
            use_legacy,
        );
        if !quorum_signature_validated {
            println!(
                "••• INVALID QUORUM SIGNATURE {}: {:?} ({})",
                block_height, self.llmq_type, self.threshold_signature
            );
            // warn!("••• Issue with quorum_signature_validated");
            return false;
        }
        println!(
            "••• quorum {:?} validated at {}",
            self.llmq_type, block_height
        );
        true
    }
}
