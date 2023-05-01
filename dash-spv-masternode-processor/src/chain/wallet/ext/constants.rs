pub const WALLET_CREATION_TIME_KEY: &str = "WALLET_CREATION_TIME_KEY";
pub const WALLET_CREATION_GUESS_TIME_KEY: &str = "WALLET_CREATION_GUESS_TIME_KEY";
pub const AUTH_PRIVKEY_KEY: &str = "authprivkey";
pub const WALLET_MNEMONIC_KEY: &str = "WALLET_MNEMONIC_KEY";
pub const WALLET_MASTER_PUBLIC_KEY: &str = "WALLET_MASTER_PUBLIC_KEY";
pub const WALLET_BLOCKCHAIN_USERS_KEY: &str = "WALLET_BLOCKCHAIN_USERS_KEY";
pub const WALLET_BLOCKCHAIN_INVITATIONS_KEY: &str = "WALLET_BLOCKCHAIN_INVITATIONS_KEY";

pub const WALLET_ACCOUNTS_KNOWN_KEY: &str = "WALLET_ACCOUNTS_KNOWN_KEY";

pub const WALLET_MASTERNODE_VOTERS_KEY: &str = "WALLET_MASTERNODE_VOTERS_KEY";
pub const WALLET_MASTERNODE_OWNERS_KEY: &str = "WALLET_MASTERNODE_OWNERS_KEY";
pub const WALLET_MASTERNODE_OPERATORS_KEY: &str = "WALLET_MASTERNODE_OPERATORS_KEY";

pub const VERIFIED_WALLET_CREATION_TIME_KEY: &str = "VERIFIED_WALLET_CREATION_TIME";
pub const REFERENCE_DATE_2001: u64 = 978307200;

pub const BIP39_CREATION_TIME: u64 = 1425492298;
// pub const BIP39_CREATION_TIME: u32 = 1425492298;
//1546810296.0 <- that would be block 1M
pub const BIP39_WALLET_UNKNOWN_CREATION_TIME: u64 = 0;

pub fn accounts_known_key_for_wallet_unique_id(unique_id: &str) -> String {
    format!("{}_{}", WALLET_ACCOUNTS_KNOWN_KEY, unique_id)
}

pub fn account_unique_id_from<S: AsRef<str>>(unique_id: S, account_number: u32) -> String {
    format!("{}-0-{}", unique_id.as_ref(), account_number)
}

pub fn wallet_identities_key(unique_id: &str) -> String {
    format!("{}_{}", WALLET_BLOCKCHAIN_USERS_KEY, unique_id)
}

pub fn wallet_identities_default_index_key(unique_id: &str) -> String {
    format!("{}_{}_DEFAULT_INDEX", WALLET_BLOCKCHAIN_USERS_KEY, unique_id)
}

pub fn wallet_invitations_key(unique_id: &str) -> String {
    format!("{}_{}", WALLET_BLOCKCHAIN_INVITATIONS_KEY, unique_id)
}

pub fn wallet_masternode_voters_key(unique_id: &str) -> String {
    format!("{}_{}", WALLET_MASTERNODE_VOTERS_KEY, unique_id)
}

pub fn wallet_masternode_owners_key(unique_id: &str) -> String {
    format!("{}_{}", WALLET_MASTERNODE_OWNERS_KEY, unique_id)
}

pub fn wallet_masternode_operators_key(unique_id: &str) -> String {
    format!("{}_{}", WALLET_MASTERNODE_OPERATORS_KEY, unique_id)
}

/// Unique Identifiers
pub fn mnemonic_unique_id_for_unique_id(unique_id: &str) -> String {
    format!("{}_{}", WALLET_MNEMONIC_KEY, unique_id)
}

pub fn creation_time_unique_id_for_unique_id(unique_id: &str) -> String {
    format!("{}_{}", WALLET_CREATION_GUESS_TIME_KEY, unique_id)
}

pub fn creation_guess_time_unique_id_for_unique_id(unique_id: &str) -> String {
    format!("{}_{}", WALLET_CREATION_GUESS_TIME_KEY, unique_id)
}

pub(crate) fn did_verify_creation_time_unique_id_for_unique_id(unique_id: &str) -> String {
    format!("{}_{}", VERIFIED_WALLET_CREATION_TIME_KEY, unique_id)
}
