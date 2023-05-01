use byte::BytesExt;
use crate::chain::tx::{CoinbaseTransaction, CreditFundingTransaction, ProviderRegistrationTransaction, ProviderUpdateRegistrarTransaction, ProviderUpdateRevocationTransaction, ProviderUpdateServiceTransaction, QuorumCommitmentTransaction, Transaction, TransactionType};
use crate::chain::tx::protocol::ReadContext;
use crate::chain::tx;

#[derive(Debug, Default)]
pub struct Factory {}

impl Factory {

    pub fn ignore_messages_of_transaction_type(r#type: TransactionType) -> bool {
        match r#type {
            TransactionType::Classic |
            TransactionType::Coinbase |
            TransactionType::SubscriptionRegistration |
            TransactionType::SubscriptionTopUp |
            TransactionType::SubscriptionCloseAccount |
            TransactionType::SubscriptionResetKey |
            TransactionType::ProviderRegistration |
            TransactionType::ProviderUpdateService |
            TransactionType::ProviderUpdateRegistrar |
            TransactionType::ProviderUpdateRevocation => false,
            TransactionType::QuorumCommitment | _ => true
        }
    }

    pub fn should_ignore_transaction_message(message: &[u8]) -> bool {
        Self::ignore_messages_of_transaction_type(Self::transaction_type_of_message(message))
    }

    pub fn transaction_type_of_message(message: &[u8]) -> TransactionType {
        let version = message.read_with::<u16>(&mut 0, byte::LE).unwrap();
        if version < 3 {
            TransactionType::Classic
        } else {
            TransactionType::from(message.read_with::<u16>(&mut 2, byte::LE).unwrap())
        }
    }

    pub fn transaction_with_message(message: &[u8], context: ReadContext) -> Option<tx::Kind> {
        let version = message.read_with::<u16>(&mut 0, byte::LE).unwrap();
        let r#type = if version < 3 {
            TransactionType::Classic
        } else {
            TransactionType::from(message.read_with::<u16>(&mut 2, byte::LE).unwrap())
        };
        match r#type {
            TransactionType::Classic => match message.read_with::<Transaction>(&mut 0, context.clone()) {
                Ok(tx) if tx.is_credit_funding_transaction() => message.read_with::<CreditFundingTransaction>(&mut 0, context).ok().map(|tx| tx::Kind::CreditFunding(tx)),
                Ok(tx) => Some(tx::Kind::Classic(tx)),
                _ => None
            },
            TransactionType::Coinbase => message.read_with::<CoinbaseTransaction>(&mut 0, context).ok().map(|tx| tx::Kind::Coinbase(tx)),
            TransactionType::ProviderRegistration => message.read_with::<ProviderRegistrationTransaction>(&mut 0, context).ok().map(|tx| tx::Kind::ProviderRegistration(tx)),
            TransactionType::ProviderUpdateService => message.read_with::<ProviderUpdateServiceTransaction>(&mut 0, context).ok().map(|tx| tx::Kind::ProviderUpdateService(tx)),
            TransactionType::ProviderUpdateRegistrar => message.read_with::<ProviderUpdateRegistrarTransaction>(&mut 0, context).ok().map(|tx| tx::Kind::ProviderUpdateRegistrar(tx)),
            TransactionType::ProviderUpdateRevocation => message.read_with::<ProviderUpdateRevocationTransaction>(&mut 0, context).ok().map(|tx| tx::Kind::ProviderUpdateRevocation(tx)),
            TransactionType::QuorumCommitment => message.read_with::<QuorumCommitmentTransaction>(&mut 0, context).ok().map(|tx| tx::Kind::QuorumCommitment(tx)),
            // we won't be able to check the payload, but try best to support it.
            _ => message.read_with::<Transaction>(&mut 0, context).ok().map(|tx| tx::Kind::Classic(tx))
        }
    }

}
