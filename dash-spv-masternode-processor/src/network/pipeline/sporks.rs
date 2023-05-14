use crate::chain::common::ChainType;
use crate::network::p2p::{ExpectedReply, P2PControlDispatcher, P2PNotificationDispatcher, P2PNotificationReceiver, SharedTimeout};
use crate::network::pipeline::{Pipeline, ThreadName};

pub struct Sporks {
    p2p: P2PControlDispatcher,
    timeout: SharedTimeout<ExpectedReply>,
}

impl Pipeline for Sporks {
    fn setup(self, back_pressure: usize, chain_type: ChainType) -> P2PNotificationDispatcher {
        self.setup_with(ThreadName::PipelineSporks, back_pressure, chain_type)
    }

    fn run(&mut self, receiver: P2PNotificationReceiver) {

    }
}
