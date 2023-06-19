use dash_spv_ffi::boxed;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr::null_mut;
use crate::keys::{BLSKey, ECDSAKey, ED25519Key, KeyKind};

pub trait AsOpaqueKey {
    fn to_opaque_ptr(self) -> *mut OpaqueKey;
}

pub trait AsCStringPtr {
    fn to_c_string_ptr(self) -> *mut c_char;
}

impl AsCStringPtr for String {
    fn to_c_string_ptr(self) -> *mut c_char {
        CString::new(self).unwrap().into_raw()
    }
}

impl AsCStringPtr for Option<String> {
    fn to_c_string_ptr(self) -> *mut c_char {
        if let Some(str) = self {
            CString::new(str).unwrap().into_raw()
        } else {
            null_mut()
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub enum OpaqueKey {
    ECDSA(*mut ECDSAKey),
    BLSLegacy(*mut BLSKey),
    BLSBasic(*mut BLSKey),
    ED25519(*mut ED25519Key),
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OpaqueKeys {
    pub keys: *mut *mut OpaqueKey,
    pub len: usize,
}

impl AsOpaqueKey for ECDSAKey {
    fn to_opaque_ptr(self) -> *mut OpaqueKey {
        boxed(OpaqueKey::ECDSA(boxed(self)))
    }
}

impl AsOpaqueKey for BLSKey {
    fn to_opaque_ptr(self) -> *mut OpaqueKey {
        boxed(if self.use_legacy {
            OpaqueKey::BLSLegacy(boxed(self))
        } else {
            OpaqueKey::BLSBasic(boxed(self))
        })
    }
}

impl AsOpaqueKey for ED25519Key {
    fn to_opaque_ptr(self) -> *mut OpaqueKey {
        boxed(OpaqueKey::ED25519(boxed(self)))
    }
}

impl AsOpaqueKey for Option<ECDSAKey> {
    fn to_opaque_ptr(self) -> *mut OpaqueKey {
        if let Some(key) = self {
            key.to_opaque_ptr()
        } else {
            null_mut()
        }
    }
}

impl AsOpaqueKey for Option<BLSKey> {
    fn to_opaque_ptr(self) -> *mut OpaqueKey {
        if let Some(key) = self {
            key.to_opaque_ptr()
        } else {
            null_mut()
        }
    }
}

impl AsOpaqueKey for Option<ED25519Key> {
    fn to_opaque_ptr(self) -> *mut OpaqueKey {
        if let Some(key) = self {
            key.to_opaque_ptr()
        } else {
            null_mut()
        }
    }
}


impl From<ECDSAKey> for *mut OpaqueKey {
    fn from(value: ECDSAKey) -> Self {
        boxed(OpaqueKey::ECDSA(boxed(value)))
    }
}

impl From<BLSKey> for *mut OpaqueKey {
    fn from(value: BLSKey) -> Self {
        boxed(if value.use_legacy {
            OpaqueKey::BLSLegacy(boxed(value))
        } else {
            OpaqueKey::BLSBasic(boxed(value))
        })
    }
}

impl From<ED25519Key> for *mut OpaqueKey {
    fn from(value: ED25519Key) -> Self {
        boxed(OpaqueKey::ED25519(boxed(value)))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OpaqueSerializedKeys {
    pub keys: *mut *mut c_char,
    pub len: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct KeyWithUniqueId {
    pub key_type: KeyKind,
    pub unique_id: u64,
    // ECDSAKey, ED25519Key or BLSKey
    pub ptr: *mut c_void,
}


