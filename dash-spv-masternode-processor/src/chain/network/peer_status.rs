#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PeerStatus {
    #[default]
    Unknown = -1,
    Disconnected = 0,
    Connecting,
    Connected,
    Banned
}
