use std::os::raw::c_void;
use tokio::runtime::Runtime;

pub type GetStatusResponseCallback = extern "C" fn(*mut c_void, rs_dapi_client::rs_dapi_client::);


#[repr(C)]
pub struct OpaqueRuntime {
    runtime: Runtime,
}

pub trait ToOpaque<T>: Sized {
    fn to_opaque_ptr(self) -> *mut T;
}



impl<T> ToOpaque<T> for Response<GetStatusResponse> {
    fn to_opaque_ptr(self) -> *mut T {
        todo!()
    }
}

impl<DTO, T> From<tonic::Response<DTO>> for dyn ToOpaque<T> {
    fn from(value: Response<DTO>) -> Self {
        todo!()
    }
}


#[no_mangle]
pub extern "C" fn create_runtime() -> *mut OpaqueRuntime {
    boxed(OpaqueRuntime { runtime: Runtime::new().unwrap() })
}

#[no_mangle]
pub unsafe extern "C" fn destroy_runtime(runtime_ptr: *mut OpaqueRuntime) {
    if !runtime_ptr.is_null() {
        unbox_any(runtime_ptr);
    }
}

#[no_mangle]
pub extern "C" fn dapi_core_get_status(runtime_ptr: *mut OpaqueRuntime, address: *const c_char, callback: GetStatusResponseCallback, context: *mut c_void) {
    let c_str = unsafe { CStr::from_ptr(address) };
    let host = c_str.to_str().unwrap();
    let wrapper = unsafe { &mut *runtime_ptr };
    opaque_runtime.runtime.spawn(async move {
        let result = core::get_status(host).await;
        callback(context, result.);
    });
}
