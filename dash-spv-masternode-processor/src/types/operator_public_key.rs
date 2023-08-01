#[repr(C)]
#[derive(Clone, Copy, Debug)]
// #[dash_spv_macro_derive::impl_ffi_conv]
pub struct OperatorPublicKey {
    pub data: [u8; 48],
    pub version: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
// #[dash_spv_macro_derive::impl_ffi_conv]
pub struct BlockOperatorPublicKey {
    // 84 // 692
    pub block_hash: [u8; 32],
    pub block_height: u32,
    pub key: [u8; 48],
    pub version: u16,
}


// #[repr(C)]
// #[derive(Clone, Copy, Debug)]
// #[dash_spv_macro_derive::ffi_conversion(OperatorPublicKey)]
// pub struct OperatorPublicKeyFFI {
//     pub data: [u8; 48],
//     pub version: u16,
// }
//
// #[repr(C)]
// #[derive(Clone, Copy, Debug)]
// #[dash_spv_macro_derive::ffi_conversion(BlockOperatorPublicKey)]
// pub struct BlockOperatorPublicKeyFFI {
//     // 84 // 692
//     pub block_hash: [u8; 32],
//     pub block_height: u32,
//     pub key: [u8; 48],
//     pub version: u16,
// }
