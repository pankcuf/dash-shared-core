use byte::ctx::{Bytes, Endian};
use byte::{BytesExt, TryRead, LE};
use dash_spv_ffi::{boxed, boxed_vec};
use std::ptr::null_mut;
use crate::chain::common::llmq_type::LLMQType;
use crate::common::llmq_version::LLMQVersion;
use crate::consensus;
use crate::crypto::byte_util::{UInt256, UInt384, UInt768};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LLMQEntry {
    // 144 bytes
    pub all_commitment_aggregated_signature: *mut [u8; 96],
    pub commitment_hash: *mut [u8; 32], // nullable
    pub llmq_type: LLMQType,
    pub entry_hash: *mut [u8; 32],
    pub llmq_hash: *mut [u8; 32],
    pub index: u16,
    pub public_key: *mut [u8; 48],
    pub threshold_signature: *mut [u8; 96],
    pub verification_vector_hash: *mut [u8; 32],
    pub saved: bool,
    pub signers_bitset: *mut u8,
    pub signers_bitset_length: usize,
    pub signers_count: u64,
    pub valid_members_bitset: *mut u8,
    pub valid_members_bitset_length: usize,
    pub valid_members_count: u64,
    pub verified: bool,
    pub version: LLMQVersion,
}
impl<'a> TryRead<'a, Endian> for LLMQEntry {
    fn try_read(bytes: &'a [u8], _endian: Endian) -> byte::Result<(Self, usize)> {
        let length = bytes.len();
        let offset = &mut 0;
        let version = bytes.read_with::<LLMQVersion>(offset, LE)?;
        let llmq_type = bytes.read_with::<u8>(offset, LE)?;
        let llmq_hash = boxed(bytes.read_with::<UInt256>(offset, LE)?.0);
        let index = if version.use_rotated_quorums() {
            bytes.read_with::<u16>(offset, LE)?
        } else {
            0
        };
        let signers_count =
            bytes.read_with::<consensus::encode::VarInt>(offset, LE)?;
        let signers_buffer_length: usize = ((signers_count.0 as usize) + 7) / 8;
        if length - *offset < signers_buffer_length {
            return Err(byte::Error::BadOffset(*offset));
        }
        let signers_bitset: &[u8] = bytes.read_with(offset, Bytes::Len(signers_buffer_length))?;
        let valid_members_count =
            bytes.read_with::<consensus::encode::VarInt>(offset, LE)?;
        let valid_members_count_buffer_length: usize = ((valid_members_count.0 as usize) + 7) / 8;
        if length - *offset < valid_members_count_buffer_length {
            return Err(byte::Error::BadOffset(*offset));
        }
        let valid_members_bitset: &[u8] =
            bytes.read_with(offset, Bytes::Len(valid_members_count_buffer_length))?;
        let public_key = boxed(bytes.read_with::<UInt384>(offset, LE)?.0);
        let verification_vector_hash = boxed(bytes.read_with::<UInt256>(offset, LE)?.0);
        let threshold_signature = boxed(bytes.read_with::<UInt768>(offset, LE)?.0);
        let all_commitment_aggregated_signature = boxed(bytes.read_with::<UInt768>(offset, LE)?.0);
        let llmq_type: LLMQType = llmq_type.into();

        Ok((
            Self {
                all_commitment_aggregated_signature,
                commitment_hash: null_mut(),
                llmq_type,
                entry_hash: null_mut(),
                llmq_hash,
                index,
                public_key,
                threshold_signature,
                verification_vector_hash,
                saved: false,
                signers_bitset: boxed_vec(signers_bitset.to_vec()),
                signers_bitset_length: signers_bitset.len(),
                signers_count: signers_count.0,
                valid_members_bitset: boxed_vec(valid_members_bitset.to_vec()),
                valid_members_bitset_length: valid_members_bitset.len(),
                valid_members_count: valid_members_count.0,
                verified: false,
                version,
            },
            *offset,
        ))
    }
}
