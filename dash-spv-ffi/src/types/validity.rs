#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Validity {
    // 37 // 296
    pub block_hash: [u8; 32],
    pub block_height: u32,
    pub is_valid: bool,
}
