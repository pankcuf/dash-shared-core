use std::cmp::max;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use std::time::SystemTime;
use byte::BytesExt;
use crate::chain::wallet::Account;
use crate::chain::wallet::ext::constants::{BIP39_CREATION_TIME, BIP39_WALLET_UNKNOWN_CREATION_TIME, creation_guess_time_unique_id_for_unique_id, creation_time_unique_id_for_unique_id, did_verify_creation_time_unique_id_for_unique_id, mnemonic_unique_id_for_unique_id, REFERENCE_DATE_2001};
use crate::chain::wallet::seed::Seed;
use crate::platform::identity::identity::Identity;
use crate::storage::keychain::Keychain;
use crate::{default_shared, util};
use crate::chain::Chain;
use crate::chain::common::chain_type::IHaveChainSettings;
use crate::chain::common::ChainType;
use crate::chain::derivation::derivation_path::DerivationPath;
use crate::chain::derivation::funds_derivation_path::FundsDerivationPath;
use crate::chain::ext::auth::Authentication;
use crate::crypto::UInt256;
use crate::environment::Environment;
use crate::storage::manager::managed_context::ManagedContext;
use crate::util::shared::{Shareable, Shared};
use crate::util::TimeUtil;

pub type SeedCompletionBlock = fn(seed: Option<Vec<u8>>, cancelled: bool);

#[derive(Clone, Debug, Default)]
pub struct Wallet {
    pub chain: Arc<RwLock<Chain>>,

    pub accounts: HashMap<u32, Account>,
    pub identities: HashMap<UInt256, Identity>,

    is_transient: bool,
    unique_id_string: String,
    wallet_creation_time: u64,
    guessed_wallet_creation_time: u64,
    checked_wallet_creation_time: bool,
    checked_guessed_wallet_creation_time: bool,
}
impl Shareable for Wallet {}
default_shared!(Wallet);

impl PartialEq for Wallet {
    fn eq(&self, other: &Self) -> bool {
        /*self.chain == other.chain &&*/ self.unique_id_string.eq(other.unique_id_string())
        // self.chain.with(|chain| chain.eq(&other.chain.with(|other| other)) && self.unique_id_string.eq(other.unique_id_string()))
    }
}

//
impl Wallet {
    pub fn is_transient(&self) -> bool {
        self.is_transient
    }
    pub fn unique_id_string(&self) -> &String {
        &self.unique_id_string
    }
    pub fn unique_id_as_str(&self) -> &str {
        self.unique_id_string.as_str()
    }
    pub fn account_with_number(&self, account_number: u32) -> Option<&Account> {
        self.accounts.get(&account_number)
    }
    pub fn account_with_number_mut(&mut self, account_number: u32) -> Option<&mut Account> {
        self.accounts.get_mut(&account_number)
    }
}

impl Wallet {
    pub fn standard_wallet_with_seed(seed: Seed, account_number: u32, is_transient: bool, chain_type: ChainType, context: Weak<ManagedContext>, chain: &Arc<RwLock<Chain>>) {
        let mut account = Account::account_with_generated_extended_public_key_for_account_number(
            account_number,
            DerivationPath::master_identity_contacts_derivation_path_for_account_number(account_number, chain_type, context.clone()),
            if account_number == 0 { Some(FundsDerivationPath::bip32_derivation_path(account_number, chain_type, context.clone())) } else { None },
            FundsDerivationPath::bip44_derivation_path(account_number, chain_type, context.clone()),
            chain_type,
            &seed
        );
        account.generate_extended_public_keys_for_seed(&seed, chain_type.is_evolution_enabled());
        Self::init_with_unique_id_and_accounts(seed.unique_id.clone(), vec![account], is_transient, chain)
    }

    fn init_with_chain(chain: Arc<RwLock<Chain>>) -> Self {
        Self { chain, ..Default::default() }
    }

    pub fn init_with_chain_and_unique_id(unique_id: String, is_transient: bool, chain: Arc<RwLock<Chain>>) -> Self {
        let mut wallet = Self::init_with_chain(chain);
        wallet.unique_id_string = unique_id;
        wallet.is_transient = is_transient;
        // wallet.load_identities();
        wallet
    }

    fn init_with_unique_id_and_accounts(unique_id: String, accounts: Vec<Account>, is_transient: bool, chain: &Arc<RwLock<Chain>>) {
        let mut wallet = Self::init_with_chain_and_unique_id(unique_id.clone(), is_transient, chain.clone());
        // let mut shared_wallet = Shared::<Wallet>::Owned(Arc::new(Mutex::new(wallet)));
        // let shared_wallet = wallet.to_shared();
        // this must be last, as adding the account queries the wallet unique ID
        // wallet.add_accounts(accounts, shared_wallet.borrow());
        for mut account in accounts {
            account.bind_to_wallet_with_unique_id(unique_id.clone(), is_transient);
            wallet.accounts.insert(account.account_number, account);
            let last_account_number = wallet.last_account_number();
            if last_account_number > Keychain::last_account_number(unique_id.as_str()).unwrap_or(0) {
                Keychain::save_last_account_number(last_account_number, unique_id.as_str())
                    .expect("Can't save last_account_number in keychain");
            }
        }
        // for mut account in accounts {
            // account.bind_to_wallet_with_unique_id(unique_id.clone(), is_transient, Arc::downgrade(&shared_wallet));
        // }

        if !is_transient {
            // if let Some(lock) = chain.upgrade() {
                if let Ok(mut chain) = chain.try_write() {
                    // if !chain.wallets.contains(&shared_wallet) {
                    // let w = shared_wallet.read(|w| *w);
                    chain.wallets.push(wallet);
                    // }
                }
                // chain.register_wallet(shared_wallet.clone());
            // }
        }
        // let shared_wallet = Shared::RwLock(Arc::new(RwLock::new(wallet)));

        // chain.loaded_specialized_derivation_paths_for_wallet(&wallet);
        // wallet.special_transactions_holder = SpecialTransactionWalletHolder::init_with_wallet(&wallet, chain.chain_context());
        //
        // wallet.identities.clear();
        // wallet.invitations.clear();
        // wallet.identities();
        // wallet.invitations();
        // blockchain users are loaded
        // add blockchain user derivation paths to account
        // shared_wallet.borrow()
        // shared_wallet
    }

    fn add_account(&mut self, mut account: Account, is_transient: bool/*, wallet: Weak<RwLock<Wallet>>*/) {
        account.bind_to_wallet_with_unique_id(self.unique_id_string.clone(), is_transient/*,  wallet*/);
        self.accounts.insert(account.account_number, account);
        let last_account_number = self.last_account_number();
        if last_account_number > self.accounts_known() {
            Keychain::save_last_account_number(last_account_number, self.unique_id_as_str())
                .expect("Can't save last_account_number in keychain");
        }
    }

    fn add_accounts(&mut self, accounts: Vec<Account>, is_transient: bool/*, wallet: Weak<RwLock<Wallet>>*/) {
        accounts.into_iter().for_each(|account| self.add_account(account, is_transient/*, wallet.clone()*/))
    }


    pub fn accounts_known_for_unique_id(unique_id: &str) -> u32 {
        Keychain::last_account_number(unique_id)
            .unwrap_or(0)
    }

    pub fn accounts_known(&self) -> u32 {
        Self::accounts_known_for_unique_id(self.unique_id_as_str())
    }

    pub fn load_identities(&self) {
        // TODO: impl storage
    }

    pub fn last_account_number(&self) -> u32 {
        assert!(!self.accounts.is_empty(), "There should always be at least one account");
        self.accounts.keys()
            .max()
            .unwrap_or(&u32::MAX)
            .clone()
    }

    /// Unique Identifiers

    pub fn mnemonic_unique_id(&self) -> String {
        mnemonic_unique_id_for_unique_id(self.unique_id_as_str())
    }
}

// async
impl Wallet {
    // authenticates user and returns seed
    pub fn seed_with_prompt(&self, authprompt: Option<String>, amount: u64) -> Result<(Option<Vec<u8>>, bool), util::Error> {
        // if let Some(chain) = self.chain.upgrade() {
        self.chain.seed_with_prompt(authprompt, amount, self.unique_id_as_str())
        // } else {
        //     Err(util::Error::Default(format!("Can't upgrade chain, probably it's destroyed")))
        // }
    }
}


impl Wallet {

    /// Wallet Creation Time
//     - (void)migrateWalletCreationTime {
//     NSData *d = getKeychainData(self.creationTimeUniqueID, nil);
//
//     if (d.length == sizeof(NSTimeInterval)) {
//     NSTimeInterval potentialWalletCreationTime = *(const NSTimeInterval *)d.bytes;
//     if (potentialWalletCreationTime < BIP39_CREATION_TIME) { //it was from reference date for sure
//     NSDate *realWalletCreationDate = [NSDate dateWithTimeIntervalSinceReferenceDate:potentialWalletCreationTime];
//     NSTimeInterval realWalletCreationTime = [realWalletCreationDate timeIntervalSince1970];
//     if (realWalletCreationTime && (realWalletCreationTime != REFERENCE_DATE_2001)) {
//     _walletCreationTime = MAX(realWalletCreationTime, BIP39_CREATION_TIME); //safeguard
//     #if DEBUG
//     DSLogPrivate(@"real wallet creation set to %@", realWalletCreationDate);
//     #else
//     DSLog(@"real wallet creation set to %@", @"<REDACTED>");
//     #endif
//     setKeychainData([NSData dataWithBytes:&realWalletCreationTime length:sizeof(realWalletCreationTime)], self.creationTimeUniqueID, NO);
//     } else if (realWalletCreationTime == REFERENCE_DATE_2001) {
//     realWalletCreationTime = 0;
//     setKeychainData([NSData dataWithBytes:&realWalletCreationTime length:sizeof(realWalletCreationTime)], self.creationTimeUniqueID, NO);
//     }
//     }
//     }
// }


    pub fn creation_time_unique_id(&self) -> String {
        creation_time_unique_id_for_unique_id(self.unique_id_as_str())
    }

    pub fn creation_guess_time_unique_id(&self) -> String {
        creation_guess_time_unique_id_for_unique_id(self.unique_id_as_str())
    }


    pub fn guessed_wallet_creation_time(&mut self) -> u64 {
        if self.guessed_wallet_creation_time != 0 {
            return self.guessed_wallet_creation_time;
        }
        if !self.checked_guessed_wallet_creation_time {
            if let Ok(d) = Keychain::get_data(self.creation_guess_time_unique_id()) {
                let size = std::mem::size_of::<u64>();
                if d.len() == size {
                    let time = u64::from_le_bytes(d.try_into().unwrap());
                    self.guessed_wallet_creation_time = time;
                    return time;
                }
            }
            self.checked_guessed_wallet_creation_time = true;
        }
        BIP39_WALLET_UNKNOWN_CREATION_TIME
    }

    pub fn set_guessed_wallet_creation_time(&mut self, time: u64) {
        if self.wallet_creation_time == 0 || self.guessed_wallet_creation_time() > 0 {
            return;
        }
        assert!(Keychain::set_data(self.creation_guess_time_unique_id(), Some(time.to_le_bytes().to_vec()), false).is_ok(), "error setting wallet guessed creation time");
        self.guessed_wallet_creation_time = time;
    }


    pub fn migrate_wallet_creation_time(&mut self) {
        if let Ok(data) = Keychain::get_data(self.creation_time_unique_id()) {
            if let Ok(potential_wallet_creation_time) = data.read_with::<u64>(&mut 0, byte::LE) {
                if potential_wallet_creation_time < BIP39_CREATION_TIME {
                    // it was from reference date for sure
                    // todo: check correct date
                    // NSDate *realWalletCreationDate = [NSDate dateWithTimeIntervalSinceReferenceDate:potentialWalletCreationTime];
                    // NSTimeInterval realWalletCreationTime = [realWalletCreationDate timeIntervalSince1970];

                    // let n = (potential_wallet_creation_time * 1_000_000_000) as u32;
                    // let t = potential_wallet_creation_time.checked_add(REFERENCE_DATE_2001)?;
                    // NaiveDateTime::from_timestamp_opt(t, n)


                    let real_wallet_creation_time = SystemTime::seconds_since_1970() - REFERENCE_DATE_2001 + potential_wallet_creation_time;
                    if real_wallet_creation_time != 0 && real_wallet_creation_time != REFERENCE_DATE_2001 {
                        self.wallet_creation_time = max(real_wallet_creation_time, BIP39_CREATION_TIME); //safeguard
                        Keychain::set_data(self.creation_time_unique_id(), Some(real_wallet_creation_time.to_le_bytes().to_vec()), false).expect("Can't save wallet creation time");
                    } else if real_wallet_creation_time == REFERENCE_DATE_2001 {
                        Keychain::set_int(0, self.creation_time_unique_id(), false)
                            .expect("Can't save wallet creation time");
                    }
                }
            }
        }
    }

    pub fn verify_wallet_creation_time(&mut self) {
        if !self.checked_wallet_creation_time {
            let key = did_verify_creation_time_unique_id_for_unique_id(self.unique_id_as_str());
            match Keychain::get_int(key.clone()) {
                Ok(status) if status == 1 => {},
                _ => {
                    self.migrate_wallet_creation_time();
                    Keychain::set_int(1, key, false).expect("Can't save in keychain");
                }
            }
            self.checked_wallet_creation_time = true;
        }
        // - (void)verifyWalletCreationTime {
        //     if (!self.checkedVerifyWalletCreationTime) {
        //     NSError *error = nil;
        //     BOOL didVerifyAlready = hasKeychainData(self.didVerifyCreationTimeUniqueID, &error);
        //     if (!didVerifyAlready) {
        //     [self migrateWalletCreationTime];
        //     setKeychainInt(1, self.didVerifyCreationTimeUniqueID, NO);
        //     }
        //     self.checkedVerifyWalletCreationTime = YES;
        //     }
        // }

    }

    pub fn wallet_creation_time(&mut self) -> u64 {
        self.verify_wallet_creation_time();
        if self.wallet_creation_time != 0 {
            return self.wallet_creation_time;
        }
        if !self.checked_wallet_creation_time {
            match Keychain::get_data(self.creation_time_unique_id()) {
                Ok(data) => {
                    if data.len() == std::mem::size_of::<u64>() {
                        let potential_wallet_creation_time = data.read_with::<u64>(&mut 0, byte::LE).unwrap();
                        if potential_wallet_creation_time > BIP39_CREATION_TIME {
                            self.wallet_creation_time = potential_wallet_creation_time;
                            return potential_wallet_creation_time;
                        }
                    }
                    self.checked_wallet_creation_time = true;
                },
                Err(err) => {}
            }
        }
        if Environment::watch_only() {
            return BIP39_WALLET_UNKNOWN_CREATION_TIME;
        }
        let guessed = self.guessed_wallet_creation_time();
        if guessed != 0 {
            guessed
        } else {
            BIP39_CREATION_TIME
        }
    }

}

