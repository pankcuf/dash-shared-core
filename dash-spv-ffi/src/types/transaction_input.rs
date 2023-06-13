use byte::ctx::Endian;
use byte::{BytesExt, TryRead, LE};
use std::ptr::null_mut;
use dash_spv_masternode_processor::crypto::{UInt256, VarBytes};
use crate::ffi::boxer::{boxed, boxed_vec};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TransactionInput {
    pub input_hash: *mut [u8; 32],
    pub index: u32,
    pub script: *mut u8,
    pub script_length: usize,
    pub signature: *mut u8,
    pub signature_length: usize,
    pub sequence: u32,
}
impl<'a> TryRead<'a, Endian> for TransactionInput {
    fn try_read(bytes: &'a [u8], _endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let input_hash = bytes.read_with::<UInt256>(offset, LE)?;
        let index = bytes.read_with::<u32>(offset, LE)?;
        let (signature, signature_length) = match bytes.read_with::<VarBytes>(offset, LE) {
            Ok(bytes) => (boxed_vec(bytes.1.to_vec()), bytes.1.len()),
            Err(_err) => (null_mut(), 0),
        };
        let sequence = bytes.read_with::<u32>(offset, LE)?;
        Ok((
            Self {
                input_hash: boxed(input_hash.0),
                index,
                script: null_mut(),
                script_length: 0,
                signature,
                signature_length,
                sequence,
            },
            *offset,
        ))
    }
}
