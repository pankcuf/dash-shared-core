use std::net::SocketAddr;
use std::time::SystemTime;
use hashes::hex::FromHex;
use crate::chain::common::ChainType;
use crate::consensus::Encodable;
use crate::consensus::encode::VarInt;
use crate::crypto::UInt256;
use crate::crypto::byte_util::Zeroable;
use crate::chain::network::message::inv_type::InvType;
use crate::chain::network::message::message::{MessageType, Payload};
use crate::chain::network::net_address::NetAddress;
use crate::chain::network::peer::{ENABLED_SERVICES, LOCAL_HOST};
use crate::crypto::UTXO;
use crate::util::TimeUtil;

// pub trait IRequest {
//     fn r#type(&self) -> MessageType;
//     fn to_data(self) -> Vec<u8>;
// }

// #[derive(Debug, Default)]
// pub enum GovernanceSyncRequest {
//     Objects(Request),
//     Votes(Request)
// }

// impl GovernanceSyncRequest {
    // pub fn state(&self) -> GovernanceRequestState {
    //     match self {
    //         GovernanceSyncRequest::Objects(_, state) => state,
    //         GovernanceSyncRequest::Votes(_, state) => state,
    //         // _ => panic!("wrong governance sync request")
    //     }
    // }
    //
    // pub fn request(&self) -> Request {
    //     match self {
    //         GovernanceSyncRequest::Objects(request, _) => request,
    //         GovernanceSyncRequest::Votes(request, _) => request,
    //         // _ => panic!("wrong governance sync request")
    //     }
    // }
// }

// pub enum GovernanceHashesRequest {
//     Object(Request, GovernanceRequestState),
//     Vote(Request, GovernanceRequestState)
// }
//
// impl GovernanceHashesRequest {
//     pub fn request(&self) -> Request {
//         match self {
//             GovernanceHashesRequest::Objects(&request, _) |
//             GovernanceHashesRequest::Votes(&request, _) => request
//         }
//     }
// }
fn array_of_hashes_enc(hashes: &Vec<UInt256>, inv_type: InvType, s: &mut Vec<u8>) {
    let inv = u32::from(inv_type);
    let mut writer = Vec::<u8>::new();
    hashes.iter().for_each(|hash| {
        inv.enc(&mut writer);
        hash.enc(&mut writer);
    });
    s.extend(writer);
}


#[derive(Clone, Debug)]
pub enum Request {
    Addr,
    FilterLoad(Vec<u8>),
    GetBlocks(Vec<UInt256>, UInt256, u32),
    GetHeaders(Vec<UInt256>, UInt256, u32),
    Inv(InvType, Vec<UInt256>),
    NotFound(Vec<u8>),
    Ping(u64),
    Pong(u64),
    Version(SocketAddr, u64, u64, ChainType),
    GovernanceHashes(InvType, Vec<UInt256>),
    GovernanceSync(UInt256, Vec<u8>),
    GovernanceSyncObjects,
    GovernanceSyncVotes,
    DSeg(UTXO),
    GetMNListDiff(UInt256, UInt256),
    GetQRInfo(Vec<UInt256>, UInt256, bool),
    GetDataForTransactionHash(UInt256),
    GetDataForTransactionHashes(Option<Vec<UInt256>>, Option<Vec<UInt256>>, Option<Vec<UInt256>>, Option<Vec<UInt256>>, Option<Vec<UInt256>>),
    TransactionInv(Vec<UInt256>, Vec<UInt256>),
    Default(MessageType),
}

impl Payload for Request {
    // type Item = Self;

    fn r#type(&self) -> MessageType {
        match self {
            Request::Addr => MessageType::Addr,
            Request::FilterLoad(..) => MessageType::Filterload,
            Request::GetBlocks(..) => MessageType::Getblocks,
            Request::GetHeaders(..) => MessageType::Getheaders,
            Request::Inv(..) |
            Request::TransactionInv(..) => MessageType::Inv,
            Request::NotFound(..) => MessageType::NotFound,
            Request::Ping(..) => MessageType::Ping,
            Request::Pong(..) => MessageType::Pong,
            Request::Version(..) => MessageType::Version,
            Request::GovernanceHashes(..) |
            Request::GetDataForTransactionHash(..) |
            Request::GetDataForTransactionHashes(..) => MessageType::Getdata,
            Request::GovernanceSync(..) => MessageType::Govsync,
            Request::DSeg(..) => MessageType::Dseg,
            Request::GetMNListDiff(..) => MessageType::Getmnlistd,
            Request::GetQRInfo(..) => MessageType::Getqrinfo,
            Request::Default(r#type) => r#type.clone(),
            Request::GovernanceSyncObjects => MessageType::Govsync,
            Request::GovernanceSyncVotes => MessageType::Govobjvote,
        }
    }
}

impl Request {

    pub(crate) fn compile(&self) -> Vec<u8> {
        let mut writer: Vec<u8> = Vec::new();
        // let len = payload.len() as u32;
        // let mut writer = Vec::<u8>::new();
        // magic.enc(&mut writer);
        // self.r#type.enc(&mut writer);
        // len.enc(&mut writer);
        // sha256d::Hash::hash(&payload).enc(&mut writer);
        // writer.extend_from_slice(&payload);
        // writer.copy_from_slice(&payload);
        // writer
        match self {
            Request::Default(..) => {},
            Request::Addr => {
                // TODO: send peer addresses we know about
                0u8.enc(&mut writer);
            },
            Request::FilterLoad(data) => {
                data.enc(&mut writer);
            },
            Request::GetBlocks(locators, hash_stop, protocol_version) |
            Request::GetHeaders(locators, hash_stop, protocol_version) => {
                protocol_version.enc(&mut writer);
                VarInt(locators.len() as u64).enc(&mut writer);
                locators.iter().for_each(|locator| {
                    locator.enc(&mut writer);
                });
                hash_stop.enc(&mut writer);
            },
            Request::NotFound(data) => {
                VarInt((data.len() / 36) as u64).enc(&mut writer);
                data.enc(&mut writer);
            },
            Request::Ping(local_nonce) => {
                local_nonce.enc(&mut writer);
            },
            Request::Pong(local_nonce) => {
                local_nonce.enc(&mut writer);
            },
            Request::Version(socket_addr, services, local_nonce, chain_type) => {
                writer.extend_from_slice(&Vec::from_hex("5312010000000000000000004007606400000000050000000000000000000000000000000000ffff55d1f34b4e1f000000000000000000000000000000000000ffff7f0000014e1f4edca9469d06be4e192f6461736877616c6c65743a312e3028746573746e6574292f0000000000").unwrap());
                // chain_type.protocol_version().enc(&mut writer);
                // ENABLED_SERVICES.enc(&mut writer);
                // SystemTime::seconds_since_1970().enc(&mut writer);
                // services.enc(&mut writer);
                // socket_addr.enc(&mut writer);
                // NetAddress::new(LOCAL_HOST, chain_type.standard_port(), ENABLED_SERVICES).enc(&mut writer);
                // local_nonce.enc(&mut writer);
                // chain_type.user_agent().enc(&mut writer);
                // 0u32.enc(&mut writer); // last block received
                // 0u8.enc(&mut writer); // relay transactions (no for SPV bloom filter mode)
            },
            Request::GovernanceSync(parent_hash, filter_data) => {
                parent_hash.enc(&mut writer);
                filter_data.enc(&mut writer);
            },
            Request::Inv(inv_type, hashes) |
            Request::GovernanceHashes(inv_type, hashes) => {
                VarInt(hashes.len() as u64).enc(&mut writer);
                array_of_hashes_enc(&hashes, *inv_type, &mut writer);
            },
            Request::DSeg(utxo) => {
                utxo.hash.enc(&mut writer);
                if utxo.hash.is_zero() {
                    u32::MAX.enc(&mut writer);
                } else {
                    utxo.n.enc(&mut writer);
                }
                0u8.enc(&mut writer);
                u32::MAX.enc(&mut writer);
            },
            Request::GetMNListDiff(base_block_hash, block_hash) => {
                base_block_hash.enc(&mut writer);
                block_hash.enc(&mut writer);
            },
            Request::GetQRInfo(base_block_hashes, block_hash, extra_share) => {
                // Number of masternode lists the spv client knows
                VarInt(base_block_hashes.len() as u64).enc(&mut writer);
                // The base block hashes of the masternode lists the spv client knows
                base_block_hashes.iter().for_each(|hash| {
                    hash.enc(&mut writer);
                });
                // Hash of the height the client requests
                block_hash.enc(&mut writer);
                // Flag to indicate if an extra share is requested
                u8::from(*extra_share).enc(&mut writer);
            },
            Request::GetDataForTransactionHash(tx_hash) => {
                VarInt(1u64).enc(&mut writer);
                InvType::Tx.enc(&mut writer);
                tx_hash.enc(&mut writer);
            },
            Request::GetDataForTransactionHashes(tx_hashes, block_hashes, is_lock_hashes, isd_lock_hashes, c_lock_hashes) => {
                let size = tx_hashes.as_ref().map_or(0, |h| h.len()) +
                    block_hashes.as_ref().map_or(0, |h| h.len()) +
                    is_lock_hashes.as_ref().map_or(0, |h| h.len()) +
                    isd_lock_hashes.as_ref().map_or(0, |h| h.len()) +
                    c_lock_hashes.as_ref().map_or(0, |h| h.len());
                VarInt(size as u64).enc(&mut writer);
                if let Some(tx_hashes) = tx_hashes {
                    array_of_hashes_enc(tx_hashes, InvType::Tx, &mut writer);
                }
                if let Some(is_lock_hashes) = is_lock_hashes {
                    array_of_hashes_enc(is_lock_hashes, InvType::InstantSendLock, &mut writer);
                }
                if let Some(isd_lock_hashes) = isd_lock_hashes {
                    array_of_hashes_enc(isd_lock_hashes, InvType::InstantSendDeterministicLock, &mut writer);
                }
                if let Some(block_hashes) = block_hashes {
                    array_of_hashes_enc(block_hashes, InvType::Merkleblock, &mut writer);
                }
                if let Some(c_lock_hashes) = c_lock_hashes {
                    array_of_hashes_enc(c_lock_hashes, InvType::ChainLockSignature, &mut writer);
                }
            },
            Request::TransactionInv(tx_hashes, tx_lock_request_hashes) => {
                VarInt((tx_hashes.len() + tx_lock_request_hashes.len()) as u64).enc(&mut writer);
                array_of_hashes_enc(tx_hashes, InvType::Tx, &mut writer);
                array_of_hashes_enc(tx_lock_request_hashes, InvType::TxLockRequest, &mut writer);
            },
            Request::GovernanceSyncObjects |
            Request::GovernanceSyncVotes => {},
        }
        writer
    }
}
