use byte::ctx::Endian;
use byte::{BytesExt, TryRead, LE};
use std::ptr::null_mut;
use dash_spv_masternode_processor::crypto::VarBytes;
use crate::ffi::boxer::boxed_vec;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TransactionOutput {
    pub amount: u64,
    pub script: *mut u8,
    pub script_length: usize,
    pub address: *mut u8,
    pub address_length: usize,
}
impl<'a> TryRead<'a, Endian> for TransactionOutput {
    fn try_read(bytes: &'a [u8], _endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let amount = bytes.read_with::<u64>(offset, LE)?;
        let script = bytes.read_with::<VarBytes>(offset, LE)?;
        Ok((
            Self {
                amount,
                script: boxed_vec(script.1.to_vec()),
                script_length: script.1.len(),
                address: null_mut(),
                address_length: 0,
            },
            *offset,
        ))
    }
}
