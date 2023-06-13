use std::ffi::CStr;
use std::os::raw::c_char;
use crate::core::core::get_status;
use crate::ffi::boxer::boxed;
use crate::ffi::opaque_runtime::OpaqueRuntime;
use crate::ffi::ToFFI;
use crate::ffi::unboxer::unbox_any;

pub mod core;
pub mod ffi;

// Define a callback type that represents a function that takes a pointer
// to a GetStatusResponse structure.
pub type GetStatusResponseCallback = extern "C" fn(*mut ffi::get_status_response::GetStatusResponse);


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
pub extern "C" fn dapi_core_get_status(runtime_ptr: *mut OpaqueRuntime, address: *const c_char, callback: GetStatusResponseCallback) {
    let c_str = unsafe { CStr::from_ptr(address) };
    let host = c_str.to_str().unwrap();
    let opaque_runtime = unsafe { &mut *runtime_ptr };
    opaque_runtime.runtime.spawn(async move {
        let result = get_status(host).await;
        callback(boxed(result.into_inner().encode()));
    });
}

