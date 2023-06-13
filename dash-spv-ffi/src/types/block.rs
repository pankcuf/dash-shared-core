#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub height: u32,
    pub hash: *mut [u8; 32],
}
