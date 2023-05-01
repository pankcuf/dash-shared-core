bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct SyncType: u32 {
        const NONE = 0;
        const BaseSPV = 1;
        const FullBlocks = 1 << 1;
        const Mempools = 1 << 2;
        const SPV = Self::BaseSPV.bits() | Self::Mempools.bits();
        const MasternodeList = 1 << 3;
        const VerifiedMasternodeList = Self::MasternodeList.bits() | Self::SPV.bits();
        const Governance = 1 << 4;
        const GovernanceVotes = 1 << 5;
        const GovernanceVoting = Self::Governance.bits() | Self::MasternodeList.bits();
        const Sporks = 1 << 6;
        const BlockchainIdentities = 1 << 7;
        const DPNS = 1 << 8;
        const Dashpay = 1 << 9;
        const MultiAccountAutoDiscovery = 1 << 10;
        const Default = Self::SPV.bits() | Self::Mempools.bits() | Self::VerifiedMasternodeList.bits() | Self::Sporks.bits() | Self::BlockchainIdentities.bits() | Self::DPNS.bits() | Self::Dashpay.bits() | Self::MultiAccountAutoDiscovery.bits();
        const NeedsWalletSyncType = Self::BaseSPV.bits() | Self::FullBlocks.bits();
        const GetsNewBlocks = Self::SPV.bits() | Self::FullBlocks.bits();
    }
}
