#[derive(Clone, Debug, Default)]
pub enum PeerManagerDesiredState {
    #[default]
    Unknown = -1,
    Connected = 1,
    Disconnected
}
