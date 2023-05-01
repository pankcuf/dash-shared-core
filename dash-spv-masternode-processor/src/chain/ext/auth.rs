use std::sync::{Arc, RwLock};
use crate::chain::Chain;
use crate::storage::keychain::Keychain;
use crate::util;

pub trait Authentication {
    fn seed_with_prompt(&self, prompt: Option<String>, amount: u64, wallet_unique_id: &str) -> Result<(Option<Vec<u8>>, bool), util::Error>;
}


impl Authentication for Arc<RwLock<Chain>> {
    fn seed_with_prompt(&self, prompt: Option<String>, amount: u64, wallet_unique_id: &str) -> Result<(Option<Vec<u8>>, bool), util::Error> {
        match self.try_write() {
            Ok(rw_lock) => {
                rw_lock.authentication_manager.with(|auth| {
                    let parse = |cancelled: bool|
                        bip39::Mnemonic::parse_normalized(Keychain::mnemonic(wallet_unique_id).expect("Can't retrieve mnemonic").as_str())
                            .map_or((None, cancelled), |mnemonic| (Some(mnemonic.to_seed_normalized("").to_vec()), cancelled));
                    if prompt.is_none() && auth.did_authenticate {
                        Ok(parse(false))
                    } else {
                        let using_biometric_authentication = amount != 0 && auth.can_use_biometric_authentication_for_amount(amount);
                        futures::executor::block_on(auth.authenticate_with_prompt(prompt, using_biometric_authentication, true))
                            .map_err(util::Error::from)
                            .map(|(authenticated, used_biometrics, cancelled)| if authenticated && (!used_biometrics || auth.update_biometrics_amount_left_after_spending_amount(amount)) {
                                parse(cancelled)
                            } else {
                                (None, cancelled)
                            })
                    }

                })
            },
            Err(err) => Err(util::Error::Default(format!("RW lock is expired?")))
        }
    }
}
