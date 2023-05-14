use std::net::SocketAddr;
use std::sync::Arc;
use mio::net::TcpListener;

#[derive(Clone, Debug)]
pub enum PeerSource {
    Outgoing(SocketAddr),
    Incoming(Arc<TcpListener>)
}
