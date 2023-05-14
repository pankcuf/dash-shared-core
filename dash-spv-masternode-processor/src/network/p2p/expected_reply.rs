#[derive(Eq, PartialEq, Hash, Debug)]
pub enum ExpectedReply {
    Block,
    Headers,
    Pong,
    FilterHeader,
    FilterCheckpoints,
    Filter
}
