use crate::crypto::UInt256;

#[derive(Clone, Debug, Default, Eq)]
pub struct Identity {
    /// This is the unique identifier representing the blockchain identity.
    /// It is derived from the credit funding transaction credit burn UTXO (as of dpp v10).
    /// Returned as a 256 bit number
    pub unique_id: UInt256,

}

impl PartialEq for Identity {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id == other.unique_id
    }
}
