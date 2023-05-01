use std::sync::Arc;
use crate::chain::Chain;
use crate::storage::context_type::ContextType;
use crate::storage::manager::managed_context::ManagedContext;

pub trait Storage {
    fn chain_context(&self) -> &Arc<ManagedContext>;
    fn masternodes_context(&self) -> &Arc<ManagedContext>;
    fn peer_context(&self) -> &Arc<ManagedContext>;
    fn platform_context(&self) -> &Arc<ManagedContext>;
    fn view_context(&self) -> &Arc<ManagedContext>;
}

impl Storage for Chain {
    fn chain_context(&self) -> &Arc<ManagedContext> {
        self.store_context.context_for(ContextType::Chain)
    }

    fn masternodes_context(&self) -> &Arc<ManagedContext> {
        self.store_context.context_for(ContextType::Masternodes)
    }

    fn peer_context(&self) -> &Arc<ManagedContext> {
        self.store_context.context_for(ContextType::Peer)
    }

    fn platform_context(&self) -> &Arc<ManagedContext> {
        self.store_context.context_for(ContextType::Platform)
    }

    fn view_context(&self) -> &Arc<ManagedContext> {
        self.store_context.context_for(ContextType::View)
    }
}
