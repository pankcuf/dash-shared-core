use core::slice;
use std::{ffi::c_void, ops::Deref};

extern "C" {
    pub fn SecAllocBytes(len: usize) -> *mut u8;
    pub fn SecFree(p: *mut ::std::os::raw::c_void);
}

pub struct SecureBox {
    c_sec_alloc: *mut u8,
    len: usize,
}

impl SecureBox {
    pub(crate) fn new(len: usize) -> Self {
        SecureBox {
            c_sec_alloc: unsafe { SecAllocBytes(len) },
            len,
        }
    }

    pub(crate) unsafe fn from_ptr(ptr: *mut u8, len: usize) -> Self {
        SecureBox {
            c_sec_alloc: ptr,
            len,
        }
    }

    // Somewhere it returns *mut c_void
    pub(crate) fn as_mut_ptr(&mut self) -> *mut c_void {
        self.c_sec_alloc as *mut c_void
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.c_sec_alloc, self.len) }
    }
}

impl Deref for SecureBox {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Drop for SecureBox {
    fn drop(&mut self) {
        unsafe { SecFree(self.as_mut_ptr()) }
    }
}
