use std::net::SocketAddr;
use crate::chain::network::message::Message;
use crate::network::p2p::PeerId;

#[derive(Debug)]
pub enum P2PControlNotification {
    Send(PeerId, Message),
    Broadcast(Message),
    Ban(PeerId, u32),
    Disconnect(PeerId),
    Height(u32),
    Bind(SocketAddr)
}
