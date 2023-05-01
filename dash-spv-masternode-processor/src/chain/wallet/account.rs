use std::collections::HashMap;
use crate::chain::common::ChainType;
use crate::chain::derivation::derivation_path::DerivationPath;
use crate::chain::derivation::derivation_path_kind::DerivationPathKind;
use crate::chain::derivation::derivation_path_reference::DerivationPathReference;
use crate::chain::derivation::funds_derivation_path::FundsDerivationPath;
use crate::chain::derivation::incoming_funds_derivation_path::IncomingFundsDerivationPath;
use crate::chain::derivation::protocol::IDerivationPath;
use crate::chain::tx::ITransaction;
use crate::chain::wallet::seed::Seed;
use crate::crypto::UInt256;
use crate::default_shared;
use crate::util::shared::Shared;

#[derive(Clone, Debug)]
pub struct Account {
    /// BIP 43 derivation paths
    fund_derivation_paths: Vec<DerivationPathKind>,
    pub default_derivation_path: Option<FundsDerivationPath>,
    pub bip44_derivation_path: Option<FundsDerivationPath>,
    pub bip32_derivation_path: Option<FundsDerivationPath>,
    pub master_contacts_derivation_path: Option<DerivationPath>,
    // pub wallet: Weak<RwLock<Wallet>>,
    pub account_number: u32,
    /// current wallet balance excluding transactions known to be invalid
    // pub balance: u64,
    // pub balance_history: Vec<u64>,
    //
    // pub spent_outputs: HashSet<UTXO>,
    // pub invalid_transaction_hashes: HashSet<UInt256>,
    // pub pending_transaction_hashes: HashSet<UInt256>,
    // pub pending_coinbase_locked_transaction_hashes: HashMap<u32, HashSet<UInt256>>,
    // pub utxos: HashSet<UTXO>,
    // // pub unspent_outputs: Vec<UTXO>,
    // pub unspent_outputs: HashSet<UTXO>,
    // /// latest 100 transactions sorted by date, most recent first
    // pub recent_transactions: Vec<Box<dyn ITransaction>>,
    // /// latest 100 transactions sorted by date, most recent first
    // pub recent_transactions_with_internal_output: Vec<Box<dyn ITransaction>>,
    // /// all wallet transactions sorted by date, most recent first
    // pub all_transactions: Vec<&'static dyn ITransaction>,
    // /// all wallet transactions sorted by date, most recent first
    // pub coinbase_transactions: Vec<CoinbaseTransaction>,
    // /// Does this account have any coinbase rewards
    // pub has_coinbase_transaction: bool,
    // /// returns the first unused external address
    // pub receive_address: Option<String>,
    // /// returns the first unused internal address
    // pub change_address: Option<String>,
    // // /// all previously generated external addresses
    // // pub external_addresses: Vec<String>,
    // // /// all previously generated internal addresses
    // // pub internal_addresses: Vec<String>,
    // /// all the contacts for an account
    // pub contacts: Vec<PotentialOneWayFriendship>,
    //
    // all_tx: HashMap<UInt256, Box<dyn ITransaction>>,
    // transactions: Vec<Box<dyn ITransaction>>,
    // transactions_to_save: Vec<Box<dyn ITransaction>>,
    // transactions_to_save_in_block_save: HashMap<u32, Vec<Box<dyn ITransaction>>>,
    contact_incoming_fund_derivation_paths_dictionary: HashMap<UInt256, IncomingFundsDerivationPath>,
    contact_outgoing_fund_derivation_paths_dictionary: HashMap<UInt256, IncomingFundsDerivationPath>,
    is_view_only_account: bool,
    // // the total amount spent from the account (excluding change)
    // total_sent: u64,
    // // the total amount received to the account (excluding change)
    // total_received: u64,
    // first_transaction_hash: Option<UInt256>,
    // @property (nonatomic, readonly) NSString *uniqueID;
    // context: Weak<ManagedContext>,
}
default_shared!(Account);

impl Default for Account {
    fn default() -> Self {
        Self {
            fund_derivation_paths: vec![],
            default_derivation_path: None,
            bip44_derivation_path: None,
            bip32_derivation_path: None,
            master_contacts_derivation_path: None,
            // wallet: Weak::new(),
            account_number: 0,
            contact_incoming_fund_derivation_paths_dictionary: Default::default(),
            contact_outgoing_fund_derivation_paths_dictionary: Default::default(),
            is_view_only_account: false
        }
    }
}

impl Account {

//     - (instancetype)initAsViewOnlyWithAccountNumber:(uint32_t)accountNumber withDerivationPaths:(NSArray<DSFundsDerivationPath *> *)derivationPaths inContext:(NSManagedObjectContext *)context {
//     NSParameterAssert(derivationPaths);
//
//     if (!(self = [self initWithAccountNumber:accountNumber withDerivationPaths:derivationPaths inContext:context])) return nil;
//     self.isViewOnlyAccount = TRUE;
//     self.transactionsToSave = [NSMutableArray array];
//     self.transactionsToSaveInBlockSave = [NSMutableDictionary dictionary];
//
//     return self;
// }
    pub fn view_only_account_with_number(account_number: u32) -> Self {
        Self { account_number, is_view_only_account: true, ..Default::default() }
    }

    pub fn account_with_generated_extended_public_key_for_account_number(account_number: u32, master_contacts_derivation_path: DerivationPath, bip32_derivation_path: Option<FundsDerivationPath>, bip44_derivation_path: FundsDerivationPath, chain_type: ChainType, seed: &Seed) -> Self {
        Self::init_with(0, master_contacts_derivation_path, bip32_derivation_path, bip44_derivation_path, chain_type)
    }

    fn init_with(account_number: u32, master_contacts_derivation_path: DerivationPath, bip32_derivation_path: Option<FundsDerivationPath>, bip44_derivation_path: FundsDerivationPath, chain_type: ChainType) -> Self {
        let mut account = Self { account_number, ..Default::default() };
        account.bip32_derivation_path = bip32_derivation_path.clone();
        account.bip44_derivation_path = Some(bip44_derivation_path.clone());
        account.master_contacts_derivation_path = Some(master_contacts_derivation_path.clone());
        account.fund_derivation_paths = if let Some(bip32) = bip32_derivation_path {
            vec![
                DerivationPathKind::Funds(bip32),
                DerivationPathKind::Funds(bip44_derivation_path),
                DerivationPathKind::Default(master_contacts_derivation_path)
            ]
        } else {
            // don't include BIP32 derivation path on higher accounts
            vec![
                DerivationPathKind::Funds(bip44_derivation_path),
                DerivationPathKind::Default(master_contacts_derivation_path)
            ]
        };
        account
    }

    pub fn verify_and_assign_added_derivation_paths(&mut self, mut derivation_paths: Vec<DerivationPathKind>) {
        derivation_paths.dedup_by(|a, b| a.is_derivation_path_equal(b));
        derivation_paths.into_iter().enumerate().for_each(|(n, derivation_path)| {
            match derivation_path {
                DerivationPathKind::Funds(path) if path.reference() == DerivationPathReference::BIP32 => {
                    if self.bip32_derivation_path.is_some() {
                        assert!(true, "There should only be one BIP 32 derivation path");
                    }
                    self.bip32_derivation_path = Some(path);
                },
                DerivationPathKind::Funds(path) if path.reference() == DerivationPathReference::BIP44 => {
                    if self.bip44_derivation_path.is_some() {
                        assert!(true, "There should only be one BIP 32 derivation path");
                    }
                    self.bip44_derivation_path = Some(path);
                },
                DerivationPathKind::Default(path) if path.reference() == DerivationPathReference::ContactBasedFundsRoot => {
                    if self.master_contacts_derivation_path.is_some() {
                        assert!(true, "There should only be one master contacts derivation path");
                    }
                    self.master_contacts_derivation_path = Some(path);
                },
                _ => {}
            }
            //assert_eq!(derivation_paths.into_iter().skip(n).filter(|path| derivation_path.is_derivation_path_equal(path)).count(), 0, "Derivation paths should all be different");
        });
    }
    pub fn fund_derivation_paths(&self) -> Vec<DerivationPathKind> {
        self.fund_derivation_paths.clone()
    }

    pub fn incoming_fund_derivation_paths(&self) -> Vec<IncomingFundsDerivationPath> {
        self.contact_incoming_fund_derivation_paths_dictionary.clone().into_values().collect()
    }

    pub fn outgoing_fund_derivation_paths(&self) -> Vec<IncomingFundsDerivationPath> {
        self.contact_outgoing_fund_derivation_paths_dictionary.clone().into_values().collect()
    }

    pub fn add_derivation_path(&mut self, path: DerivationPathKind) {
        if !self.is_view_only_account {
            self.verify_and_assign_added_derivation_paths(vec![path.clone()]);
        }
        if self.verify_derivation_path_not_already_present(&path) {
            self.fund_derivation_paths.push(path);
        }
    }

    pub fn verify_derivation_path_not_already_present(&self, path: &DerivationPathKind) -> bool {
        // Added derivation paths should be different from existing ones on account
        // self.fund_derivation_paths.iter().find(|other| path.is_derivation_path_equal(other)).is_none()
        self.fund_derivation_paths.iter().find(|other| path.is_derivation_path_equal(other)).is_none()
    }


    pub fn bind_to_wallet_with_unique_id(&mut self, unique_id: String, is_transient: bool/*, wallet: Weak<RwLock<Wallet>>*/) {
        if let Some(path) = self.bip32_derivation_path.as_mut() {
            path.set_is_transient(is_transient);
            path.set_wallet_unique_id(unique_id.clone());
        }
        if let Some(path) = self.bip44_derivation_path.as_mut() {
            path.set_is_transient(is_transient);
            path.set_wallet_unique_id(unique_id.clone());
        }
        if let Some(path) = self.master_contacts_derivation_path.as_mut() {
            path.set_is_transient(is_transient);
            path.set_wallet_unique_id(unique_id.clone());
        }
        if let Some(path) = self.default_derivation_path.as_mut() {
            path.set_is_transient(is_transient);
            path.set_wallet_unique_id(unique_id.clone());
        }
        self.fund_derivation_paths.iter_mut().for_each(|path| {
            path.set_is_transient(is_transient);
            path.set_wallet_unique_id(unique_id.clone());
        });
        self.contact_incoming_fund_derivation_paths_dictionary.values_mut().for_each(|path| {
            path.set_is_transient(is_transient);
            path.set_wallet_unique_id(unique_id.clone());
        });
        self.contact_outgoing_fund_derivation_paths_dictionary.values_mut().for_each(|path| {
            path.set_is_transient(is_transient);
            path.set_wallet_unique_id(unique_id.clone());
        });
        // self.wallet = wallet;
    }

    pub fn generate_extended_public_keys_for_seed(&mut self, seed: &Seed, is_evolution_enabled: bool) {
        // if let Some(path) = self.bip32_derivation_path.as_mut() {
        //     path.generate_extended_public_key_from_seed(seed)
        //         .expect("Can't generate extended public key from seed");
        // }
        self.fund_derivation_paths().iter_mut().for_each(|derivation_path| {
            derivation_path.generate_extended_public_key_from_seed(seed)
                .expect("Can't generate extended public key from seed");
        });
        if is_evolution_enabled {
            self.master_contacts_derivation_path.as_mut()
                .unwrap()
                .generate_extended_public_key_from_seed(seed);
        }
        // let wallet = self.wallet.borrow();
        // self.wallet.with(|w| {
        //     let chain = w.chain.borrow();
        //     chain.with(|c| {
        //         if c.is_evolution_enabled() {
        //             self.master_contacts_derivation_path.as_mut()
        //                 .unwrap()
        //                 .generate_extended_public_key_from_seed(seed);
        //         }
        //     })
        // })
        // if self.wallet.is_evolution_enabled() {
        //     self.master_contacts_derivation_path.as_mut()
        //         .unwrap()
        //         .generate_extended_public_key_from_seed(seed);
        // }
        // match self.wallet.upgrade() {
        //     Some(wallet_mutex) => match wallet_mutex.lock() {
        //         Ok(wallet) => match wallet.chain.upgrade() {
        //             Some(chain_mutex) => match chain_mutex.lock() {
        //                 Ok(chain) if chain.is_evolution_enabled() => {
        //                     self.master_contacts_derivation_path.as_mut()
        //                         .unwrap()
        //                         .generate_extended_public_key_from_seed(seed);
        //                 },
        //                 _ => {}
        //             },
        //             _ => {}
        //         },
        //         _ => {}
        //     },
        //     _ => {}
        // }
    }

    // Calculated Attributes
    // pub fn unique_id(&self) -> String {
    //     //0 is for type 0
    //     account_unique_id_from(self.wallet.unique_id_as_str(), self.account_number)
    // }

    /// Proposal Transaction Creation
    pub fn proposal_collateral_transaction_with_data(&mut self, data: Vec<u8>) -> Option<&dyn ITransaction> {
        todo!()
        // let script = Vec::<u8>::proposal_info(data);
        // self.transaction_for_amounts(vec![PROPOSAL_COST], vec![script], true)
    }

}
