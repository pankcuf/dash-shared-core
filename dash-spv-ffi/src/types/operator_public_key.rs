#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OperatorPublicKey {
    pub data: [u8; 48],
    pub version: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BlockOperatorPublicKey {
    // 84 // 692
    pub block_hash: [u8; 32],
    pub block_height: u32,
    pub key: [u8; 48],
    pub version: u16,
}
