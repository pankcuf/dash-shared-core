use crate::crypto::UInt256;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DerivationPathFeaturePurpose {
    Default = 9,
    Identities = 5,
    IdentitiesSubfeatureAuthentication = 0,
    IdentitiesSubfeatureRegistration = 1,
    IdentitiesSubfeatureTopup = 2,
    IdentitiesSubfeatureInvitations = 3,
    DashPay = 15,
}

impl From<u32> for DerivationPathFeaturePurpose {
    fn from(orig: u32) -> Self {
        match orig {
            0 => Self::IdentitiesSubfeatureAuthentication,
            1 => Self::IdentitiesSubfeatureRegistration,
            2 => Self::IdentitiesSubfeatureTopup,
            3 => Self::IdentitiesSubfeatureInvitations,
            5 => Self::Identities,
            9 => Self::Default,
            15 => Self::DashPay,
            _ => Self::Default,
        }
    }
}

impl From<DerivationPathFeaturePurpose> for u32 {
    fn from(value: DerivationPathFeaturePurpose) -> u32 {
        value as u32
    }
}

impl From<DerivationPathFeaturePurpose> for UInt256 {
    fn from(value: DerivationPathFeaturePurpose) -> UInt256 {
        UInt256::from(u32::from(value))
    }
}

impl DerivationPathFeaturePurpose {
    pub fn into_u32(self) -> u32 {
        self.into()
    }
    pub fn into_u256(self) -> UInt256 {
        self.into()
    }
}
