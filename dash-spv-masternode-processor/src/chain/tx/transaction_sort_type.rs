pub enum TransactionSortType {
    /// No sorting
    None,
    /// Shuffle outputs
    Shuffle,
    /// Sorting inputs & outputs according to BIP-69
    BIP69,
}
