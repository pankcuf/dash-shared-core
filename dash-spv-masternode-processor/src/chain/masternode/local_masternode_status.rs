#[derive(Clone, Debug, Default, PartialEq)]
pub enum LocalMasternodeStatus {
    #[default]
    New = 0,
    Created = 1,
    Registered = 2,
}
