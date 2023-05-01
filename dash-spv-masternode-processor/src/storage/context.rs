use std::sync::Arc;
use crate::storage::context_type::ContextType;
use crate::storage::manager::managed_context::ManagedContext;

#[derive(Clone, Debug)]
pub struct StoreContext {
    chain_context: Arc<ManagedContext>,
    masternodes_context: Arc<ManagedContext>,
    peer_context: Arc<ManagedContext>,
    platform_context: Arc<ManagedContext>,
    view_context: Arc<ManagedContext>,
}
impl StoreContext {
    pub fn new_default() -> Self {
        Self {
            chain_context: Arc::new(ManagedContext::new_const_default()),
            masternodes_context: Arc::new(ManagedContext::new_const_default()),
            peer_context: Arc::new(ManagedContext::new_const_default()),
            platform_context: Arc::new(ManagedContext::new_const_default()),
            view_context: Arc::new(ManagedContext::new_const_default())
        }
    }
}
impl Default for StoreContext {
    fn default() -> Self {
        Self::new_default()
    }
}

impl StoreContext {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn context_for(&self, r#type: ContextType) -> &Arc<ManagedContext> {
        match r#type {
            ContextType::View => &self.view_context,
            ContextType::Peer => &self.peer_context,
            ContextType::Chain => &self.chain_context,
            ContextType::Masternodes => &self.masternodes_context,
            ContextType::Platform => &self.platform_context
        }
    }

}
