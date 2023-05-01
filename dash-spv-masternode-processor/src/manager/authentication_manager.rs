use crate::chain::Wallet;
use crate::{default_shared, util};
use crate::util::Shared;

#[derive(Clone, Debug, Default)]
pub struct AuthenticationManager {
    // pub environment: &'static Environment,

    pub biometric_spending_limit: u64,
    // true if the app should use authentication once it is set up
    pub should_use_authentication: bool,
    // true if the app uses authentication and it is set up
    pub uses_authentication: bool,
    // true if the user authenticated after this was last set to false
    pub did_authenticate: bool,
    // last known time from an ssl server connection
    pub secure_time: u64,
    // Secure time was updated by HTTP response since app starts
    pub secure_time_updated: bool,
    pub lockout_wait_time: u64,
}
default_shared!(AuthenticationManager);

pub enum AuthenticationError {
    CannotCreateWallet,
    CannotRetrieveSeedFromKeychain,
    NotAuthenticated
}

impl From<AuthenticationError> for util::Error {
    fn from(value: AuthenticationError) -> Self {
        match value {
            AuthenticationError::CannotCreateWallet => util::Error::Default(format!("Cannot create wallet")),
            AuthenticationError::CannotRetrieveSeedFromKeychain => util::Error::Default(format!("Cannot retrieve seed from keychain")),
            AuthenticationError::NotAuthenticated => util::Error::Default(format!("Not Authenticated")),
        }
    }
}

impl AuthenticationManager {
    // pub fn new(environment: &Environment) -> Self {
    //     Self { environment, ..Default::default() }
    // }

    pub(crate) fn can_use_biometric_authentication_for_amount(&self, amount: u64) -> bool {
        todo!()
    }

    pub(crate) fn update_biometrics_amount_left_after_spending_amount(&self, amount: u64) -> bool {
        todo!()
    }

    pub async fn authenticate_with_prompt(&self, prompt: Option<String>, using_biometric_authentication: bool, alert_if_lockout: bool) -> Result<(bool, bool, bool), AuthenticationError> {
        todo!()
    }

    pub async fn seed_with_prompt(&self, prompt: String, wallet: &Wallet, amount: u64, force_authentication: bool) -> Result<(Option<Vec<u8>>, bool), AuthenticationError> {
        todo!()
    }

}
