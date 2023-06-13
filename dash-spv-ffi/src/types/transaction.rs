use byte::ctx::Endian;
use byte::{BytesExt, TryRead, LE};
use std::ptr::null_mut;
use crate::consensus;
use crate::ffi::boxer::{boxed, boxed_vec};
use crate::tx::{TransactionType, TX_UNCONFIRMED};
use crate::types::{TransactionInput, TransactionOutput};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Transaction {
    pub inputs: *mut *mut TransactionInput,
    pub inputs_count: usize,
    pub outputs: *mut *mut TransactionOutput,
    pub outputs_count: usize,
    pub lock_time: u32,
    pub version: u16,
    pub tx_hash: *mut [u8; 32],
    pub tx_type: TransactionType,
    pub payload_offset: usize,
    pub block_height: u32,
}
impl<'a> TryRead<'a, Endian> for Transaction {
    fn try_read(bytes: &'a [u8], _endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let version = bytes.read_with::<u16>(offset, LE)?;
        let tx_type = TransactionType::from(bytes.read_with::<u16>(offset, LE)?);
        let inputs_count = bytes
            .read_with::<consensus::encode::VarInt>(offset, LE)?
            .0 as usize;
        if inputs_count == 0 && tx_type.requires_inputs() {
            return Err(byte::Error::BadOffset(*offset));
        }
        let mut inputs_vec: Vec<*mut TransactionInput> = Vec::with_capacity(inputs_count);
        for _i in 0..inputs_count {
            inputs_vec.push(boxed(bytes.read_with::<TransactionInput>(offset, LE)?));
        }
        let outputs_count = bytes
            .read_with::<consensus::encode::VarInt>(offset, LE)?
            .0 as usize;
        let mut outputs_vec: Vec<*mut TransactionOutput> = Vec::new();
        for _i in 0..outputs_count {
            outputs_vec.push(boxed(bytes.read_with::<TransactionOutput>(offset, LE)?));
        }
        let lock_time = bytes.read_with::<u32>(offset, LE)?;
        Ok((
            Self {
                inputs: boxed_vec(inputs_vec),
                inputs_count,
                outputs: boxed_vec(outputs_vec),
                outputs_count,
                tx_hash: null_mut(),
                version,
                tx_type,
                lock_time,
                payload_offset: *offset,
                block_height: TX_UNCONFIRMED as u32,
            },
            *offset,
        ))
    }
}
