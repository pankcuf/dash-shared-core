use std::ffi::CStr;
use std::ptr::null_mut;
use std::os::raw::c_char;
use crate::core::core::get_status;
use crate::ffi::boxer::boxed;
use crate::ffi::opaque_runtime::OpaqueRuntime;
use crate::ffi::unboxer::unbox_any;

use dash_spv_ffi::FFIConversion;
use dash_spv_macro_derive::ffi_conversion;
use rs_dapi_client::rs_dapi_client::{get_status_response::{
    Chain, Masternode, Network, NetworkFee, Time, Version,
}, GetStatusResponse};
use tokio::runtime::Runtime;


pub mod core;
pub mod ffi;

// Define a callback type that represents a function that takes a pointer
// to a GetStatusResponse structure.
pub type GetStatusResponseCallback = extern "C" fn(*mut GetStatusResponseFFI);


/// Create a new Tokio runtime.
///
/// This function initializes a new Tokio runtime and returns a pointer to it.
/// The returned runtime is capable of executing asynchronous tasks.
///
/// # Returns
/// A pointer to the newly created Tokio runtime.
#[no_mangle]
pub extern "C" fn create_runtime() -> *mut OpaqueRuntime {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    boxed(OpaqueRuntime { runtime })
}

/// Destroy a Tokio runtime.
///
/// This function takes a pointer to a Tokio runtime and safely destroys it.
/// It should be called when the runtime is no longer needed to avoid memory leaks.
///
/// # Parameters
/// - `runtime_ptr`: A pointer to the Tokio runtime that should be destroyed.
#[no_mangle]
pub unsafe extern "C" fn destroy_runtime(runtime_ptr: *mut OpaqueRuntime) {
    if !runtime_ptr.is_null() {
        unbox_any(runtime_ptr);
    }
}

/// Fetch the status of a DAPI node asynchronously.
///
/// This function initiates an asynchronous network request to fetch the status of a DAPI node.
/// When the request is complete, it calls the provided callback function with the result.
///
/// # Parameters
/// - `runtime_ptr`: A pointer to the Tokio runtime used to execute async tasks.
/// - `address`: A pointer to a null-terminated string representing the DAPI node's address.
/// - `callback`: The callback function to be called with the result of the request.
#[no_mangle]
pub unsafe extern "C" fn dapi_core_get_status(runtime_ptr: *mut OpaqueRuntime, address: *const c_char, callback: GetStatusResponseCallback) {
    let c_str = unsafe { CStr::from_ptr(address) };
    let host = c_str.to_str().unwrap();
    let opaque_runtime = unsafe { &mut *runtime_ptr };
    opaque_runtime.runtime.spawn(async move {
        let result = get_status(host).await;
        let response: *mut GetStatusResponseFFI = FFIConversion::ffi_to(result.into_inner());
        callback(response);
    });
}

#[repr(C)]
pub struct OpaqueRuntime {
    pub(crate) runtime: Runtime,
}


#[repr(C)]
#[ffi_conversion(Version)]
pub struct VersionFFI {
    pub protocol: u32,
    pub software: u32,
    pub agent: *mut c_char,
}

#[repr(C)]
#[ffi_conversion(Time)]
pub struct TimeFFI {
    pub now: u32,
    pub offset: i32,
    pub median: u32,
}

#[repr(C)]
#[ffi_conversion(Chain)]
pub struct ChainFFI {
    pub name: *mut c_char,
    pub headers_count: u32,
    pub blocks_count: u32,
    pub best_block_hash: *mut [u8; 32],
    pub difficulty: f64,
    pub chain_work: *mut [u8; 32],
    pub is_synced: bool,
    pub sync_progress: f64,
}

#[repr(C)]
#[ffi_conversion(Masternode)]
pub struct MasternodeFFI {
    pub status: i32,
    pub pro_tx_hash: *mut [u8; 32],
    pub pose_penalty: u32,
    pub is_synced: bool,
    pub sync_progress: f64,
}

#[repr(C)]
#[ffi_conversion(Network)]
pub struct NetworkFFI {
    pub peers_count: u32,
    pub fee: *mut NetworkFeeFFI,
}

#[repr(C)]
#[ffi_conversion(NetworkFee)]
pub struct NetworkFeeFFI {
    pub relay: f64,
    pub incremental: f64,
}

#[repr(C)]
#[ffi_conversion(GetStatusResponse)]
pub struct GetStatusResponseFFI {
    pub version: *mut VersionFFI,
    pub time: *mut TimeFFI,
    pub status: i32,
    pub sync_progress: f64,
    pub chain: *mut ChainFFI,
    pub masternode: *mut MasternodeFFI,
    pub network: *mut NetworkFFI,
}

unsafe impl Send for GetStatusResponseFFI {}


