#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LLMQIndexedHash {
    pub index: u32,
    pub hash: *mut [u8; 32],
}
