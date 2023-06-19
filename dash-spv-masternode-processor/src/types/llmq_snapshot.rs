use byte::ctx::{Bytes, Endian};
use byte::{BytesExt, TryRead, LE};
use dash_spv_ffi::boxed_vec;
use crate::common::LLMQSnapshotSkipMode;
use crate::consensus::encode;
use crate::crypto::byte_util::BytesDecodable;
use crate::impl_bytes_decodable;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LLMQSnapshot {
    pub member_list_length: usize,
    pub member_list: *mut u8,
    // Skip list at height n
    pub skip_list_length: usize,
    pub skip_list: *mut i32,
    //  Mode of the skip list
    pub skip_list_mode: LLMQSnapshotSkipMode,
}

impl_bytes_decodable!(LLMQSnapshot);

impl<'a> TryRead<'a, Endian> for LLMQSnapshot {
    fn try_read(bytes: &'a [u8], _endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let skip_list_mode = bytes.read_with::<LLMQSnapshotSkipMode>(offset, LE)?;
        let member_list_length = bytes.read_with::<encode::VarInt>(offset, LE)?.0 as usize;
        let member_list: &[u8] =
            bytes.read_with(offset, Bytes::Len((member_list_length + 7) / 8))?;
        let skip_list_length = bytes.read_with::<encode::VarInt>(offset, LE)?.0 as usize;
        let mut skip_list_vec = Vec::with_capacity(skip_list_length);
        for _i in 0..skip_list_length {
            skip_list_vec.push(bytes.read_with::<i32>(offset, LE)?);
        }
        Ok((
            Self {
                member_list_length,
                member_list: boxed_vec(member_list.to_vec()),
                skip_list_length,
                skip_list: boxed_vec(skip_list_vec),
                skip_list_mode,
            },
            *offset,
        ))
    }
}
