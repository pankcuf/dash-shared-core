#[derive(Debug, Default, Eq, Hash, PartialEq)]
pub enum ContextType {
    View,
    Peer,
    #[default]
    Chain,
    Masternodes,
    Platform
}
