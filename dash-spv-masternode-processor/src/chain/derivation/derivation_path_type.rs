// pub enum DerivationPathType {
//     Unknown = 0,
//     ClearFunds = 1,
//     AnonymousFunds = 1 << 1,
//     ViewOnlyFunds = 1 << 2,
//     SingleUserAuthentication = 1 << 3,
//     MultipleUserAuthentication = 1 << 4,
//     PartialPath = 1 << 5,
//     ProtectedFunds = 1 << 6,
//     CreditFunding = 1 << 7,
//     IsForAuthentication = DerivationPathType::SingleUserAuthentication | DerivationPathType::MultipleUserAuthentication,
//     IsForFunds = DerivationPathType::ClearFunds | DerivationPathType::AnonymousFunds | DerivationPathType::ViewOnlyFunds | DerivationPathType::ProtectedFunds
// }

// pub mod DerivationPathType {
//     pub const Unknown: u8 = 0;
//     pub const ClearFunds: u8 = 1;
//     pub const AnonymousFunds: u8 = 1 << 1;
//     pub const ViewOnlyFunds: u8 = 1 << 2;
//     pub const SingleUserAuthentication: u8 = 1 << 3;
//     pub const MultipleUserAuthentication: u8 = 1 << 4;
//     pub const PartialPath: u8 = 1 << 5;
//     pub const ProtectedFunds: u8 = 1 << 6;
//     pub const CreditFunding: u8 = 1 << 7;
//     pub const IsForAuthentication: u8 = SingleUserAuthentication | MultipleUserAuthentication;
//     pub const IsForFunds: u8 = ClearFunds | AnonymousFunds | ViewOnlyFunds | ProtectedFunds;
// }

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct DerivationPathType: u8 {
        const Unknown = 0;
        const ClearFunds = 1;
        const AnonymousFunds = 1 << 1;
        const ViewOnlyFunds = 1 << 2;
        const SingleUserAuthentication = 1 << 3;
        const MultipleUserAuthentication = 1 << 4;
        const PartialPath = 1 << 5;
        const ProtectedFunds = 1 << 6;
        const CreditFunding = 1 << 7;
        const IsForAuthentication = Self::SingleUserAuthentication.bits() | Self::MultipleUserAuthentication.bits();
        const IsForFunds = Self::ClearFunds.bits() | Self::AnonymousFunds.bits() | Self::ViewOnlyFunds.bits() | Self::ProtectedFunds.bits();
    }
}
