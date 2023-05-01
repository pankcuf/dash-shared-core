#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DerivationPathReference {
    #[default]
    Unknown = 0,
    BIP32 = 1,
    BIP44 = 2,
    BlockchainIdentities = 3,
    ProviderFunds = 4,
    ProviderVotingKeys = 5,
    ProviderOperatorKeys = 6,
    ProviderOwnerKeys = 7,
    ContactBasedFunds = 8,
    ContactBasedFundsRoot = 9,
    ContactBasedFundsExternal = 10,
    BlockchainIdentityCreditRegistrationFunding = 11,
    BlockchainIdentityCreditTopupFunding = 12,
    BlockchainIdentityCreditInvitationFunding = 13,
    PlatformNodeKeys = 14,
    Root = 255,
}

impl DerivationPathReference {
    /// the reference of type of derivation path
    pub fn name(&self) -> &str {
        match self {
            DerivationPathReference::Root => "Root",
            DerivationPathReference::BIP32 => "BIP 32",
            DerivationPathReference::BIP44 => "BIP 44",
            DerivationPathReference::ProviderFunds => "Provider Holding Funds Keys",
            DerivationPathReference::ProviderOwnerKeys => "Provider Owner Keys",
            DerivationPathReference::ProviderOperatorKeys => "Provider Operator Keys",
            DerivationPathReference::ProviderVotingKeys => "Provider Voting Keys",
            DerivationPathReference::BlockchainIdentities => "Blockchain Identities",
            DerivationPathReference::ContactBasedFunds => "Contact Funds",
            DerivationPathReference::ContactBasedFundsExternal => "Contact Funds External",
            DerivationPathReference::ContactBasedFundsRoot => "Contact Funds Root",
            DerivationPathReference::BlockchainIdentityCreditRegistrationFunding => "BI Credit Registration Funding",
            DerivationPathReference::BlockchainIdentityCreditTopupFunding => "BI Credit Topup Funding",
            DerivationPathReference::BlockchainIdentityCreditInvitationFunding => "BI Credit Invitation Funding",
            _ => "Unknown"
        }
    }
}

impl From<u32> for DerivationPathReference {
    fn from(orig: u32) -> Self {
        match orig {
            0 => DerivationPathReference::Unknown,
            1 => DerivationPathReference::BIP32,
            2 => DerivationPathReference::BIP44,
            3 => DerivationPathReference::BlockchainIdentities,
            4 => DerivationPathReference::ProviderFunds,
            5 => DerivationPathReference::ProviderVotingKeys,
            6 => DerivationPathReference::ProviderOperatorKeys,
            7 => DerivationPathReference::ProviderOwnerKeys,
            8 => DerivationPathReference::ContactBasedFunds,
            9 => DerivationPathReference::ContactBasedFundsRoot,
            10 => DerivationPathReference::ContactBasedFundsExternal,
            11 => DerivationPathReference::BlockchainIdentityCreditRegistrationFunding,
            12 => DerivationPathReference::BlockchainIdentityCreditTopupFunding,
            13 => DerivationPathReference::BlockchainIdentityCreditInvitationFunding,
            255 => DerivationPathReference::Root,
            _ => DerivationPathReference::Unknown,
        }
    }
}

impl From<DerivationPathReference> for u32 {
    fn from(value: DerivationPathReference) -> Self {
        match value {
            DerivationPathReference::Unknown => 0,
            DerivationPathReference::BIP32 => 1,
            DerivationPathReference::BIP44 => 2,
            DerivationPathReference::BlockchainIdentities => 3,
            DerivationPathReference::ProviderFunds => 4,
            DerivationPathReference::ProviderVotingKeys => 5,
            DerivationPathReference::ProviderOperatorKeys => 6,
            DerivationPathReference::ProviderOwnerKeys => 7,
            DerivationPathReference::ContactBasedFunds => 8,
            DerivationPathReference::ContactBasedFundsRoot => 9,
            DerivationPathReference::ContactBasedFundsExternal => 10,
            DerivationPathReference::BlockchainIdentityCreditRegistrationFunding => 11,
            DerivationPathReference::BlockchainIdentityCreditTopupFunding => 12,
            DerivationPathReference::BlockchainIdentityCreditInvitationFunding => 13,
            DerivationPathReference::PlatformNodeKeys => 14,
            DerivationPathReference::Root => 255,
        }
    }
}
impl From<&DerivationPathReference> for u32 {
    fn from(value: &DerivationPathReference) -> Self {
        match value {
            DerivationPathReference::Unknown => 0,
            DerivationPathReference::BIP32 => 1,
            DerivationPathReference::BIP44 => 2,
            DerivationPathReference::BlockchainIdentities => 3,
            DerivationPathReference::ProviderFunds => 4,
            DerivationPathReference::ProviderVotingKeys => 5,
            DerivationPathReference::ProviderOperatorKeys => 6,
            DerivationPathReference::ProviderOwnerKeys => 7,
            DerivationPathReference::ContactBasedFunds => 8,
            DerivationPathReference::ContactBasedFundsRoot => 9,
            DerivationPathReference::ContactBasedFundsExternal => 10,
            DerivationPathReference::BlockchainIdentityCreditRegistrationFunding => 11,
            DerivationPathReference::BlockchainIdentityCreditTopupFunding => 12,
            DerivationPathReference::BlockchainIdentityCreditInvitationFunding => 13,
            DerivationPathReference::PlatformNodeKeys => 14,
            DerivationPathReference::Root => 255,
        }
    }
}
