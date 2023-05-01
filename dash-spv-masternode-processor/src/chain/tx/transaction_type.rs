#[repr(C)]
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum TransactionType {
    #[default]
    Classic = 0,
    ProviderRegistration = 1,
    ProviderUpdateService = 2,
    ProviderUpdateRegistrar = 3,
    ProviderUpdateRevocation = 4,
    Coinbase = 5,
    QuorumCommitment = 6,
    SubscriptionRegistration = 8,
    SubscriptionTopUp = 9,
    SubscriptionResetKey = 10,
    SubscriptionCloseAccount = 11,
    Transition = 12,
    // tmp

    /// TODO: find actual value for this type
    CreditFunding = 255,
}

impl From<u16> for TransactionType {
    fn from(orig: u16) -> Self {
        match orig {
            0x0000 => TransactionType::Classic,
            0x0001 => TransactionType::ProviderRegistration,
            0x0002 => TransactionType::ProviderUpdateService,
            0x0003 => TransactionType::ProviderUpdateRegistrar,
            0x0004 => TransactionType::ProviderUpdateRevocation,
            0x0005 => TransactionType::Coinbase,
            0x0006 => TransactionType::QuorumCommitment,
            0x0008 => TransactionType::SubscriptionRegistration,
            0x0009 => TransactionType::SubscriptionTopUp,
            0x000A => TransactionType::SubscriptionResetKey,
            0x000B => TransactionType::SubscriptionCloseAccount,
            0x000C => TransactionType::Transition,
            _ => TransactionType::Classic,
        }
    }
}

impl From<TransactionType> for u16 {
    fn from(value: TransactionType) -> Self {
        value as u16
    }
}

impl TransactionType {
    pub(crate) fn raw_value(&self) -> u16 {
        *self as u16
    }
    pub fn requires_inputs(&self) -> bool {
        true
    }
}
