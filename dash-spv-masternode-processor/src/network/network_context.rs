// use std::sync::Arc;
// use tokio::runtime::{Builder, Runtime};

#[derive(Debug)]
pub struct NetworkContext {
    // pub runtime: Arc<Runtime>,
}

impl Default for NetworkContext {
    fn default() -> Self {
        Self {
            // runtime: Arc::<Runtime>::new_uninit()
            // runtime: Arc::new(Builder::new_multi_thread()
            //     .worker_threads(1)
            //     .build()
            //     .unwrap())
        }
    }
}

impl NetworkContext {
    pub fn new() -> Self {
        Self {
            // runtime: Arc::new(Builder::new_multi_thread()
            //     .worker_threads(1)
            //     .build()
            //     .unwrap())
        }
    }
}

