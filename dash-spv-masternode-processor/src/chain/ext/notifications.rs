use crate::chain::Chain;
use crate::util::Shared;

pub trait ChainNotifications {
    // fn post(&self, )
}

impl ChainNotifications for Shared<Chain> {

}
