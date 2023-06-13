#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LLMQTypedHash {
    pub llmq_type: u8,
    pub llmq_hash: *mut [u8; 32],
}
