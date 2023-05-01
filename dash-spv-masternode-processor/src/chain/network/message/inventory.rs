use std::collections::HashMap;
use byte::{BytesExt, TryRead};
use crate::chain::network::InvType;
use crate::chain::network::message::inv_hash::InvHash;
use crate::chain::network::peer::MAX_GETDATA_HASHES;
use crate::consensus::encode::VarInt;
use crate::crypto::{byte_util::Zeroable, UInt256};
use crate::network::p2p::state::PeerState;
use crate::network::p2p::state_flags::PeerStateFlags;

#[derive(Clone, Debug)]
pub struct Inventory {
    pub map: HashMap<InvType, Vec<UInt256>>
}

impl<'a, T: PeerState> TryRead<'a, &T> for Inventory {
    fn try_read(bytes: &'a [u8], state: &T) -> byte::Result<(Self, usize)> {
        let mut offset = 0usize;
        let count = bytes.read_with::<VarInt>(&mut offset, byte::LE)?;
        if count.len() == 0 || bytes.len() < count.len() + count.0 as usize * 36 {
            println!("malformed inv message, length {} is too short", bytes.len());
            return Err(byte::Error::Incomplete);
        } else if count.0 > MAX_GETDATA_HASHES as u64 {
            println!("dropping inv message, {} is too many items, max is {}", count.0, MAX_GETDATA_HASHES);
            return Err(byte::Error::BadInput { err: "too many inv items" });
        }
        let mut only_private_send_transactions = false;
        let mut map = HashMap::<InvType, Vec<UInt256>>::new();
        while let Some(InvHash { r#type, hash }) = bytes.read_iter::<InvHash>(&mut offset, byte::LE).next() {
            if hash.is_zero() { continue }
            if r#type != InvType::Tx {
                only_private_send_transactions = false;
            } else if offset == count.len() {
                only_private_send_transactions = true;
            }
            match r#type {
                InvType::Tx | InvType::TxLockRequest |
                InvType::Block | InvType::Merkleblock |
                InvType::InstantSendLock |
                InvType::InstantSendDeterministicLock |
                InvType::Spork |
                InvType::GovernanceObject |
                InvType::ChainLockSignature =>
                    map.entry(r#type)
                        .or_insert_with(Vec::new)
                        .push(hash),
                InvType::DSTx |
                InvType::TxLockVote |
                InvType::MasternodePing |
                InvType::MasternodeVerify |
                InvType::MasternodeBroadcast |
                InvType::QuorumFinalCommitment |
                InvType::DummyCommitment |
                InvType::QuorumContribution |
                InvType::CompactBlock |
                InvType::QuorumPrematureCommitment |
                InvType::GovernanceObjectVote |
                InvType::MasternodePaymentVote => {},
                _ => panic!("inventory type not dealt with: {:?}", r#type)
            }
        }
        let block_hashes_num = map.get(&InvType::Block).map_or(0, |d| d.len()) +
            map.get(&InvType::Merkleblock).map_or(0, |d| d.len());
        if state.chain_type().syncs_blockchain() &&
            !state.flags().intersects(PeerStateFlags::SENT_GETADDR | PeerStateFlags::SENT_MEMPOOL | PeerStateFlags::SENT_GETBLOCKS) &&
            (map.contains_key(&InvType::Tx) || map.contains_key(&InvType::TxLockRequest)) &&
            !only_private_send_transactions {
            return Err(byte::Error::BadInput { err: "got tx inv message before loading a filter" });
        } else if map.get(&InvType::Tx).map_or(0, |d| d.len()) +
            map.get(&InvType::TxLockRequest).map_or(0, |d| d.len()) +
            map.get(&InvType::InstantSendLock).map_or(0, |d| d.len()) +
            map.get(&InvType::InstantSendDeterministicLock).map_or(0, |d| d.len()) > 10000 {
            return Err(byte::Error::BadInput { err: "too many transactions, disconnecting" });
        } else if (3..500).contains(&block_hashes_num) &&
            (0..state.get_height() - (state.known_block_hashes().len() + block_hashes_num) as u32)
                .contains(&state.get_local_height()) {
            return Err(byte::Error::BadInput { err: "non-standard inv (fewer block hashes than expected)" });
        }
        let last_block_hash = state.get_local_hash();
        if block_hashes_num == 1 &&
            (last_block_hash.eq(&map[&InvType::Block][0]) || last_block_hash.eq(&map[&InvType::Merkleblock][0])) {
            map.remove(&InvType::Block);
            map.remove(&InvType::Merkleblock);
        }
        // if inv_map.get(&InvType::Block).is_some() && inv_map.get(&InvType::Block).unwrap().len() == 1 && inv_map.get(&InvType::Merkleblock).is_none() {
        //     state.set_last_block_hash(inv_map.get(&InvType::Block).unwrap()[0]);
        // } else if inv_map.get(&InvType::Merkleblock).is_some() && inv_map.get(&InvType::Merkleblock).unwrap().len() == 1 && inv_map.get(&InvType::Block).is_none() {
        //     state.set_last_block_hash(inv_map.get(&InvType::Merkleblock).unwrap()[0]);
        // }

        Ok((Self { map }, offset))
    }
}
