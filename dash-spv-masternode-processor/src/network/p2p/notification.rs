use std::net::SocketAddr;
use crate::chain::network::message::{Message, Response};
use crate::network::p2p::PeerId;

/// A message from network to downstream
#[derive(Clone)]
pub enum P2PNotification {
    Outgoing(Message),
    Incoming(PeerId, Response),
    Connected(PeerId, Option<SocketAddr>),
    Disconnected(PeerId, bool/*banned*/)
}
