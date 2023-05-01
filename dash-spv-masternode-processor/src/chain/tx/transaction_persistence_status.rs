#[derive(Clone, Debug, Default)]
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
