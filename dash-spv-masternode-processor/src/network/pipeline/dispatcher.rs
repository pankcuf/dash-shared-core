use std::sync::{Arc, Mutex};
use crate::chain::common::ChainType;
use crate::network::p2p::{P2PNotificationDispatcher, P2PNotificationReceiver};
use crate::network::pipeline::ThreadName;

pub struct PipelineDispatcher {
    dispatchers: Arc<Mutex<Vec<P2PNotificationDispatcher>>>
}

impl PipelineDispatcher {
    pub fn new(chain_type: ChainType, receiver: P2PNotificationReceiver, pipelines: Vec<P2PNotificationDispatcher>) -> PipelineDispatcher {
        let dispatchers = Arc::new(Mutex::new(pipelines));
        let cloned_dispatchers = dispatchers.clone();
        ThreadName::PipelineDispatcher
            .thread(chain_type)
            .spawn(move || Self::incoming_messages_loop(receiver, cloned_dispatchers))
            .unwrap();
        PipelineDispatcher { dispatchers }
    }

    pub fn add_pipeline(&mut self, dispatcher: P2PNotificationDispatcher) {
        let mut locked_dispatchers = self.dispatchers.lock().unwrap();
        locked_dispatchers.push(dispatcher);
    }

    pub fn add_pipelines(&mut self, dispatchers: Vec<P2PNotificationDispatcher>) {
        let mut locked_dispatchers = self.dispatchers.lock().unwrap();
        locked_dispatchers.extend(dispatchers);
    }

    fn incoming_messages_loop(receiver: P2PNotificationReceiver, dispatchers: Arc<Mutex<Vec<P2PNotificationDispatcher>>>) {
        while let Ok(notification) = receiver.recv() {
            let list = dispatchers.lock().unwrap();
            for dispatcher in list.iter() {
                dispatcher.send(notification.clone());
            }
        }
        panic!("dispatcher failed");
    }
}
