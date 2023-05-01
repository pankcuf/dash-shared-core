use std::sync::Arc;
use crate::chain::block::MerkleBlock;
use crate::chain::{Chain, ChainLock, tx, Wallet};
use crate::chain::network::Peer;
use crate::chain::tx::{InstantSendLock, ITransaction, TransactionDirection};
use crate::crypto::UInt256;
use crate::util::Shared;

pub trait PeerTransactionDelegate {
    /// called when the peer relays either a merkleblock or a block header, headers will have 0 totalTransactions
    fn peer_relayed_header(&self, peer: &Peer, block: &MerkleBlock);
    fn peer_relayed_block(&self, peer: &Peer, block: &MerkleBlock);
    fn peer_relayed_chain_lock(&self, peer: &Peer, chain_lock: ChainLock);
    fn peer_relayed_too_many_orphan_blocks(&self, peer: &Peer, orphan_block_count: usize);
    fn peer_relayed_not_found_messages_with_transaction_hashes(&self, peer: &Peer, tx_hashes: Vec<UInt256>, block_hashes: Vec<UInt256>);
    fn peer_relayed_transaction(&self, peer: &Peer, transaction: &tx::Kind, block: &MerkleBlock);
    fn peer_relayed_instant_send_transaction_lock(&self, peer: &Peer, transaction_lock: InstantSendLock);
    fn peer_requested_transaction(&self, peer: &Peer, tx_hash: &UInt256) -> Option<tx::Kind>;
    fn peer_has_transaction_with_hash(&self, peer: &Peer, tx_hash: &UInt256);
    fn peer_rejected_transaction(&self, peer: &Peer, tx_hash: UInt256, code: u8);
    fn peer_has_instant_send_lock_hashes(&self, peer: &Peer, hashes: Vec<UInt256>);
    fn peer_has_instant_send_deterministic_lock_hashes(&self, peer: &Peer, hashes: Vec<UInt256>);
    fn peer_has_chain_lock_hashes(&self, peer: &Peer, hashes: Vec<UInt256>);
    fn peer_set_fee_per_byte(&self, peer: &Peer, fee_per_kb: u64);

}


impl PeerTransactionDelegate for Shared<Chain> {
    fn peer_relayed_header(&self, peer: &Peer, block: &MerkleBlock) {
        todo!()
    }

    fn peer_relayed_block(&self, peer: &Peer, block: &MerkleBlock) {
        todo!()
    }

    fn peer_relayed_chain_lock(&self, peer: &Peer, chain_lock: ChainLock) {
        todo!()
    }

    fn peer_relayed_too_many_orphan_blocks(&self, peer: &Peer, orphan_block_count: usize) {
        todo!()
    }

    fn peer_relayed_not_found_messages_with_transaction_hashes(&self, peer: &Peer, tx_hashes: Vec<UInt256>, block_hashes: Vec<UInt256>) {
        todo!()
    }

    fn peer_relayed_transaction(&self, peer: &Peer, transaction: &tx::Kind, block: &MerkleBlock) {
        todo!()
    }

    fn peer_relayed_instant_send_transaction_lock(&self, peer: &Peer, transaction_lock: InstantSendLock) {
        todo!()
    }

    fn peer_requested_transaction(&self, peer: &Peer, tx_hash: &UInt256) -> Option<tx::Kind> {
        todo!()
    }

    fn peer_has_transaction_with_hash(&self, peer: &Peer, tx_hash: &UInt256) {
        todo!()
    }

    fn peer_rejected_transaction(&self, peer: &Peer, tx_hash: UInt256, code: u8) {
        todo!()
    }

    fn peer_has_instant_send_lock_hashes(&self, peer: &Peer, hashes: Vec<UInt256>) {
        todo!()
    }

    fn peer_has_instant_send_deterministic_lock_hashes(&self, peer: &Peer, hashes: Vec<UInt256>) {
        todo!()
    }

    fn peer_has_chain_lock_hashes(&self, peer: &Peer, hashes: Vec<UInt256>) {
        todo!()
    }

    fn peer_set_fee_per_byte(&self, peer: &Peer, fee_per_kb: u64) {
        todo!()
    }
}


pub trait Transactions {
    fn transaction_for_hash(&self, hash: &UInt256) -> Option<tx::Kind>;
    fn transaction_and_wallet_for_hash(&self, hash: &UInt256) -> Option<(&Wallet, tx::Kind)>;
    fn all_transactions(&self) -> Vec<tx::Kind>;
    /// The amount sent globally by the transaction (total wallet outputs consumed, change and fee included)
    fn amount_received_from_transaction(&self, transaction: tx::Kind) -> u64;
    /// The amount sent globally by the transaction (total wallet outputs consumed, change and fee included)
    fn amount_sent_by_transaction(&self, transaction: tx::Kind) -> u64;

    fn direction_of_transaction(&self, transaction: tx::Kind) -> TransactionDirection;
    fn trigger_updates_for_local_references(&self, transaction: tx::Kind);

    // fn clear_transaction_relays_for_peer(&self, peer: &Peer);
}

impl Transactions for Arc<Chain> {
    fn transaction_for_hash(&self, hash: &UInt256) -> Option<tx::Kind> {
        self.transaction_and_wallet_for_hash(hash).map(|(_, tx)| tx)
    }

    fn transaction_and_wallet_for_hash(&self, hash: &UInt256) -> Option<(&Wallet, tx::Kind)> {
        todo!()
        // self.wallets.with(|wallets| wallets.iter().find_map(|wallet|
        //     wallet.accounts.values().find_map(|account| {
        //         if let Some(tx) = account.transaction_for_hash(hash) {
        //             Some((wallet, tx))
        //         } else {
        //             None
        //         }
        //     })))
    }

    fn all_transactions(&self) -> Vec<tx::Kind> {
        todo!()
        // self.wallets.with(|wallets| wallets.iter().fold(Vec::new(), |mut transactions, wallet| {
        //     transactions.extend(wallet.all_transactions());
        //     transactions
        // }))
    }

    fn amount_received_from_transaction(&self, transaction: tx::Kind) -> u64 {
        todo!()
        // self.wallets
        //     .with(|wallets| wallets
        //     .iter()
        //     .map(|wallet| wallet.amount_received_from_transaction(transaction))
        //     .sum())
    }

    fn amount_sent_by_transaction(&self, transaction: tx::Kind) -> u64 {
        todo!()
        // self.wallets
        //     .with(|wallets| wallets
        //     .iter()
        //     .map(|wallet| wallet.amount_sent_by_transaction(transaction))
        //     .sum())
    }

    fn direction_of_transaction(&self, transaction: tx::Kind) -> TransactionDirection {
        todo!()
        // let sent = self.amount_sent_by_transaction(transaction.clone());
        // let received = self.amount_received_from_transaction(transaction.clone());
        // let fee = if let Some(acc) = self.first_account_that_can_contain_transaction(transaction.clone()) {
        //     acc.fee_for_transaction(transaction)
        // } else {
        //     0
        // };
        // if sent > 0 && (received + fee) == sent {
        //     TransactionDirection::Moved
        // } else if sent > 0 {
        //     TransactionDirection::Sent
        // } else if received > 0 {
        //     TransactionDirection::Received
        // } else {
        //     TransactionDirection::NotAccountFunds
        // }
    }

    fn trigger_updates_for_local_references(&self, transaction: tx::Kind) {
        transaction.trigger_updates_for_local_references();
    }

    // fn clear_transaction_relays_for_peer(&self, peer: &Peer) {
    //     self.transaction_manager.clear_transaction_relays_for_peer(peer);
    // }
}
