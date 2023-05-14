use std::net::SocketAddr;
use std::{io, thread};
use std::sync::mpsc;
use futures::executor::{ThreadPool, ThreadPoolBuilder};
use crate::chain::common::{ChainType, IHaveChainSettings};
use crate::network::p2p::{P2PNotificationDispatcher, P2PNotificationReceiver};

pub mod dispatcher;
pub mod headers;
pub mod manager;
pub mod ping;
pub mod sporks;

pub use self::dispatcher::PipelineDispatcher;
pub use self::headers::Headers;
pub use self::ping::Ping;

pub trait Pipeline where Self: Send + Sync + Sized + 'static {
    fn setup(self, back_pressure: usize, chain_type: ChainType) -> P2PNotificationDispatcher;
    fn setup_with(mut self, thread_name: ThreadName, back_pressure: usize, chain_type: ChainType) -> P2PNotificationDispatcher {
        let (sender, receiver) = mpsc::sync_channel(back_pressure);
        thread_name
            .thread(chain_type)
            .spawn(move || self.run(receiver))
            .unwrap();
        P2PNotificationDispatcher::new(sender)
    }

    fn run(&mut self, receiver: P2PNotificationReceiver);
}

pub enum ThreadName {
    PipelineManager,
    PipelineDispatcher,
    PipelineHeaders,
    PipelinePing,
    PipelineSporks,
    P2PConnect,
    P2PManager,
    Peer(SocketAddr)
}

impl ThreadName {

    fn name(&self, chain_type: ChainType) -> String {
        let postfix = chain_type.name();
        match self {
            Self::PipelineManager => format!("pipeline_manager_{}", postfix),
            Self::PipelineDispatcher => format!("pipeline_dispatcher_{}", postfix),
            Self::PipelineHeaders => format!("pipeline_headers_{}", postfix),
            Self::PipelinePing => format!("pipeline_ping_{}", postfix),
            Self::PipelineSporks => format!("pipeline_sporks_{}", postfix),
            Self::P2PConnect => format!("p2p_connect_{}", postfix),
            Self::P2PManager => format!("p2p_manager_{}", postfix),
            Self::Peer(addr) => format!("peer.{}_{}", addr, postfix)
        }
    }

    pub fn thread(&self, chain_type: ChainType) -> thread::Builder {
        thread::Builder::new().name(self.name(chain_type))
    }

    pub fn pool(&self, chain_type: ChainType, size: usize) -> Result<ThreadPool, io::Error> {
        ThreadPoolBuilder::new()
            .name_prefix(self.name(chain_type))
            .pool_size(2)
            .create()
    }
}

