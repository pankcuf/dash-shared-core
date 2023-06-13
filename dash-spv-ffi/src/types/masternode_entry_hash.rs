#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MasternodeEntryHash {
    pub block_hash: [u8; 32],
    pub block_height: u32,
    pub hash: [u8; 32],
}
