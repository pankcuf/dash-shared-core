#[derive(Clone, Debug, Default)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub enum TransactionPersistenceStatus {
    #[default]
    NotSaved,
    Saving,
    Saved
}

// impl Default for TransactionPersistenceStatus {
//     fn default() -> Self {
//         TransactionPersistenceStatus::NotSaved
//     }
// }
