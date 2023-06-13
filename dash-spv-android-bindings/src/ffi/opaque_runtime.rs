use tokio::runtime::Runtime;

#[repr(C)]
pub struct OpaqueRuntime {
    pub(crate) runtime: Runtime,
}

