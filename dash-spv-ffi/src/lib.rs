use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::c_char;

pub trait FFIConversion<T> {
    unsafe fn ffi_from(ffi: *mut Self) -> T;
    unsafe fn ffi_to(obj: T) -> *mut Self;
}

impl FFIConversion<String> for c_char {
    unsafe fn ffi_from(ffi: *mut Self) -> String {
        let c_str = CStr::from_ptr(ffi);
        let s = c_str.to_str().unwrap();
        s.to_string()
    }

    unsafe fn ffi_to(obj: String) -> *mut Self {
        CString::new(obj).unwrap().into_raw()
    }
}

impl FFIConversion<Vec<u8>> for [u8; 32] {
    unsafe fn ffi_from(ffi: *mut Self) -> Vec<u8> {
        (*ffi).to_vec()
    }

    unsafe fn ffi_to(obj: Vec<u8>) -> *mut Self {
        convert_vec_to_fixed_array(&obj)
    }
}

pub fn boxed<T>(obj: T) -> *mut T {
    Box::into_raw(Box::new(obj))
}

pub fn boxed_vec<T>(vec: Vec<T>) -> *mut T {
    let mut slice = vec.into_boxed_slice();
    let ptr = slice.as_mut_ptr();
    mem::forget(slice);
    ptr
}

/// # Safety
pub unsafe fn unbox_any<T: ?Sized>(any: *mut T) -> Box<T> {
    Box::from_raw(any)
}

/// # Safety
pub unsafe fn unbox_vec<T>(vec: Vec<*mut T>) -> Vec<Box<T>> {
    vec.iter().map(|&x| unbox_any(x)).collect()
}

/// # Safety
pub unsafe fn unbox_vec_ptr<T>(ptr: *mut T, count: usize) -> Vec<T> {
    Vec::from_raw_parts(ptr, count, count)
}

pub fn convert_vec_to_fixed_array<const N: usize>(data: &Vec<u8>) -> *mut [u8; N] {
    let mut fixed_array = [0u8; N];
    fixed_array.copy_from_slice(data);
    boxed(fixed_array)
}