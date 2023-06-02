#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct QuorumsCLSigsObject {
    pub signature: *mut [u8; 96],
    pub index_set_count: usize,
    pub index_set: *mut u16,
}
