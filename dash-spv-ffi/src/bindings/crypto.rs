use std::slice;
use crate::ffi::ByteArray;

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn processor_x11(data: *const u8, len: usize) -> ByteArray {
    let data = slice::from_raw_parts(data, len);
    rs_x11_hash::get_x11_hash(data).into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn processor_blake3(data: *const u8, len: usize) -> ByteArray {
    let data = slice::from_raw_parts(data, len);
    blake3::hash(data).into()
}
