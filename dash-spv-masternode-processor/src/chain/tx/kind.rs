use byte::TryRead;
use crate::chain::{Chain, tx};
use crate::chain::common::ChainType;
use crate::chain::tx::coinbase_transaction::CoinbaseTransaction;
use crate::chain::tx::credit_funding_transaction::CreditFundingTransaction;
use crate::chain::tx::{InstantSendLock, ITransaction, TransactionInput, TransactionOutput, TransactionType};
use crate::chain::tx::provider_registration_transaction::ProviderRegistrationTransaction;
use crate::chain::tx::provider_update_registrar_transaction::ProviderUpdateRegistrarTransaction;
use crate::chain::tx::provider_update_revocation_transaction::ProviderUpdateRevocationTransaction;
use crate::chain::tx::provider_update_service_transaction::ProviderUpdateServiceTransaction;
use crate::chain::tx::quorum_commitment_transaction::QuorumCommitmentTransaction;
use crate::chain::tx::transaction::Transaction;
use crate::crypto::UInt256;
use crate::network::p2p::state::PeerState;
use crate::network::p2p::state_flags::PeerStateFlags;
use crate::util::Shared;

#[derive(Clone, Debug)]
pub enum Kind {
    Classic(Transaction),
    ProviderRegistration(ProviderRegistrationTransaction),
    ProviderUpdateService(ProviderUpdateServiceTransaction),
    ProviderUpdateRegistrar(ProviderUpdateRegistrarTransaction),
    ProviderUpdateRevocation(ProviderUpdateRevocationTransaction),
    Coinbase(CoinbaseTransaction),
    QuorumCommitment(QuorumCommitmentTransaction),
    // SubscriptionRegistration,
    // SubscriptionTopUp,
    // SubscriptionResetKey,
    // SubscriptionCloseAccount,
    // Transition,
    CreditFunding(CreditFundingTransaction),
}

impl Kind {
    pub fn tx_mut(&mut self) -> &mut dyn ITransaction {
        match self {
            Self::Classic(tx) => tx,
            Self::ProviderRegistration(tx) => tx,
            Self::ProviderUpdateService(tx) => tx,
            Self::ProviderUpdateRegistrar(tx) => tx,
            Self::ProviderUpdateRevocation(tx) => tx,
            Self::Coinbase(tx) => tx,
            Self::QuorumCommitment(tx) => tx,
            Self::CreditFunding(tx) => tx,
        }
    }
    pub fn tx(&self) -> &dyn ITransaction {
        match self {
            Self::Classic(tx) => tx,
            Self::ProviderRegistration(tx) => tx,
            Self::ProviderUpdateService(tx) => tx,
            Self::ProviderUpdateRegistrar(tx) => tx,
            Self::ProviderUpdateRevocation(tx) => tx,
            Self::Coinbase(tx) => tx,
            Self::QuorumCommitment(tx) => tx,
            Self::CreditFunding(tx) => tx,
        }
    }
}

impl ITransaction for Kind {
    fn chain(&self) -> Shared<Chain> {
        self.tx().chain()
    }

    fn chain_type(&self) -> ChainType {
        self.tx().chain_type()
    }

    fn r#type(&self) -> TransactionType {
        self.tx().r#type()
    }

    fn block_height(&self) -> u32 {
        self.tx().block_height()
    }

    fn tx_hash(&self) -> UInt256 {
        self.tx().tx_hash()
    }

    fn tx_lock_time(&self) -> u32 {
        self.tx().tx_lock_time()
    }

    fn inputs(&self) -> Vec<TransactionInput> {
        self.tx().inputs()
    }

    fn outputs(&self) -> Vec<TransactionOutput> {
        self.tx().outputs()
    }

    fn input_addresses(&self) -> Vec<String> {
        self.tx().input_addresses()
    }

    fn output_addresses(&self) -> Vec<String> {
        self.tx().output_addresses()
    }

    fn size(&self) -> usize {
        self.tx().size()
    }

    fn to_data_with_subscript_index(&self, subscript_index: Option<u64>) -> Vec<u8> {
        self.tx().to_data_with_subscript_index(subscript_index)
    }

    fn set_instant_send_received_with_instant_send_lock(&mut self, instant_send_lock: Option<Shared<InstantSendLock>>) {
        self.tx_mut().set_instant_send_received_with_instant_send_lock(instant_send_lock);
    }

    fn is_coinbase_classic_transaction(&self) -> bool {
        self.tx().is_coinbase_classic_transaction()
    }
}

impl<'a, T: PeerState> TryRead<'a, &T> for Kind {
    fn try_read(bytes: &'a [u8], state: &T) -> byte::Result<(Self, usize)> {
        let tx = tx::Factory::transaction_with_message(bytes, tx::ReadContext(state.chain_type(), state.chain()));
        if tx.is_none() && !tx::Factory::should_ignore_transaction_message(bytes) {
            Err(byte::Error::BadInput { err: "malformed tx message" })
        } else if !state.flags().intersects(PeerStateFlags::SENT_FILTER | PeerStateFlags::SENT_GETDATATXBLOCKS) {
            Err(byte::Error::BadInput { err: "got tx message before loading a filter" })
        } else if let Some(tx) = tx {
            Ok((tx, bytes.len()))
        } else {
            Err(byte::Error::Incomplete)
        }
    }
}
