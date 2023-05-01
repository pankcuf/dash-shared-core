#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum GovernanceRequestState {
    #[default]
    None,
    GovernanceObjectHashes,
    GovernanceObjectHashesCountReceived,
    GovernanceObjectHashesReceived,
    GovernanceObjects,
    GovernanceObjectVoteHashes,
    GovernanceObjectVoteHashesCountReceived,
    GovernanceObjectVoteHashesReceived,
    GovernanceObjectVotes,
}
