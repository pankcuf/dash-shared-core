#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VarInt {
    // 9 // 72
    pub value: u64,
    pub length: usize,
}
