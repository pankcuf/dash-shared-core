#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum SyncPhase {
    #[default]
    Offline = 0,
    InitialTerminalBlocks,
    ChainSync,
    Synced
}

// impl Default for SyncPhase {
//     fn default() -> Self {
//         Self::Offline
//     }
// }
