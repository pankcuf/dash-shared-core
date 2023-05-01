#![allow(dead_code)]
#![allow(unused_variables)]
#[macro_use]
pub mod processing;

pub extern crate bitcoin_hashes as hashes;
pub extern crate secp256k1;
#[macro_use] extern crate bitflags;
extern crate core;
#[macro_use] extern crate log;
extern crate simplelog;

#[cfg(test)]
mod lib_tests;
#[cfg(test)]
mod tests;


#[cfg(feature = "std")]
use std::io;
#[cfg(not(feature = "std"))]
use core2::io;

#[macro_use]
pub mod internal_macros;
#[macro_use]
pub mod macros;
pub mod bindings;
pub mod blockdata;
pub mod chain;
pub mod chains_manager;
pub mod common;
pub mod consensus;
pub mod crypto;
pub mod environment;
pub mod ffi;
pub mod hash_types;
pub mod keys;
pub mod manager;
pub mod models;
pub mod network;
pub mod platform;
pub mod storage;
pub mod tx;
pub mod types;
pub mod util;

// Don't remove, it's for testing purposes (look at ${project_dir}/c/main.c)
// #[no_mangle]
// pub extern "C" fn test_func(get_masternode_list_by_block_hash: MasternodeListLookup, destroy_masternode_list: MasternodeListDestroy, opaque_context: *const std::ffi::c_void) {
//     let block_hash = UInt256::MIN;
//     dash_spv_ffi::ffi::callbacks::lookup_masternode_list(
//         block_hash,
//         |h: UInt256| unsafe { (get_masternode_list_by_block_hash)(boxed(h.0), opaque_context) },
//         |list: *mut types::MasternodeList| unsafe { (destroy_masternode_list)(list) });
//
// }
// #[no_mangle]
// pub extern "C" fn test_snapshot_func(
//     get_llmq_snapshot_by_block_hash: GetLLMQSnapshotByBlockHash,
//     save_llmq_snapshot: SaveLLMQSnapshot,
//     destroy_snapshot: LLMQSnapshotDestroy,
//     opaque_context: *const std::ffi::c_void) {
//     let block_hash = UInt256::MIN;
//     let lookup_result = unsafe { (get_llmq_snapshot_by_block_hash)(boxed(block_hash.0), opaque_context) };
//     if !lookup_result.is_null() {
//         let data = unsafe { (*lookup_result).decode() };
//         unsafe { (destroy_snapshot)(lookup_result) };
//         println!("test_snapshot_func: ({:?})", data);
//     } else {
//         println!("test_snapshot_func: (None)");
//
//     }
// }


