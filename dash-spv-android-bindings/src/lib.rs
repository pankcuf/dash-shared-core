use std::ffi::CStr;
use std::os::raw::c_char;
use crate::core::core::get_status;
use crate::ffi::boxer::boxed;
use crate::ffi::opaque_runtime::OpaqueRuntime;
use crate::ffi::ToFFI;
use crate::ffi::unboxer::unbox_any;

pub mod core;
pub mod ffi;

pub type GetStatusResponseCallback = extern "C" fn(*mut ffi::get_status_response::GetStatusResponse);

#[no_mangle]
pub extern "C" fn create_runtime() -> *mut OpaqueRuntime {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    boxed(OpaqueRuntime { runtime })
}

#[no_mangle]
pub unsafe extern "C" fn destroy_runtime(runtime_ptr: *mut OpaqueRuntime) {
    if !runtime_ptr.is_null() {
        unbox_any(runtime_ptr);
    }
}

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

