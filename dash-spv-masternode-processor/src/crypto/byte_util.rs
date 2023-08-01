use std::{io::Write, mem, net::{IpAddr, Ipv4Addr}, slice};
use byte::{BytesExt, ctx::Endian, LE, Result, TryRead, TryWrite};
use ed25519_dalek::{SigningKey, VerifyingKey};
use hashes::{Hash, hash160, HashEngine, Hmac, HmacEngine, ripemd160, sha1, sha256, sha256d, sha512};
use secp256k1::rand::{Rng, thread_rng};
#[cfg(feature = "generate-dashj-tests")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::chain::params::{BIP32_SEED_KEY, ED25519_SEED_KEY};
use crate::consensus::{Decodable, Encodable, ReadExt, WriteExt};
use crate::ffi;
use crate::hashes::{hex::{FromHex, ToHex}, hex};
use crate::util::base58;
use crate::util::data_ops::short_hex_string_from;

pub trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

pub trait Reversable {
    fn reverse(&mut self) -> Self;
    fn reversed(&self) -> Self;
}

pub trait Zeroable {
    fn is_zero(&self) -> bool;
}

pub trait Random {
    fn random() -> Self where Self: Sized;
}

pub trait MutDecodable<'a, T: TryRead<'a, Endian>> {
    fn from_mut(bytes: *mut u8) -> Option<T>;
}
pub trait ConstDecodable<'a, T: TryRead<'a, Endian>> {
    fn from_const(bytes: *const u8) -> Option<T>;
}
pub trait BytesDecodable<'a, T: TryRead<'a, Endian>> {
    fn from_bytes(bytes: &'a [u8], offset: &mut usize) -> Option<T>;
}

// todo: migrate?
// #[repr(C, align(32))]
// pub union UInt256 {
//     pub u8: [u8; 256 / 8],
//     pub u16: [u16; 256 / 16],
//     pub u32: [u32; 256 / 32],
//     pub u64: [u64; 256 / 64],
// }


#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub struct UInt128(pub [u8; 16]);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub struct UInt160(pub [u8; 20]);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub struct UInt256(pub [u8; 32]);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub struct UInt384(pub [u8; 48]);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub struct UInt512(pub [u8; 64]);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[dash_spv_macro_derive::impl_ffi_conv]
pub struct UInt768(pub [u8; 96]);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ECPoint(pub [u8; 33]);

#[macro_export]
macro_rules! impl_ffi_bytearray {
    ($var_type: ident) => {
        impl From<$var_type> for ffi::ByteArray {
            fn from(value: $var_type) -> Self {
                let vec = value.0.to_vec();
                vec.into()
            }
        }
        impl From<Option<$var_type>> for ffi::ByteArray {
            fn from(value: Option<$var_type>) -> Self {
                if let Some(v) = value {
                    v.into()
                } else {
                    ffi::ByteArray::default()
                }
            }
        }
    }
}


#[macro_export]
macro_rules! impl_random {
    ($var_type: ident, $byte_len: expr) => {
        impl Random for $var_type {
            fn random() -> Self where Self: Sized {
                let mut data: [u8; $byte_len] = [0u8; $byte_len];
                for i in 0..32 {
                    data[i] = thread_rng().gen();
                }
                $var_type(data)
            }
        }
    }
}

#[macro_export]
macro_rules! impl_bytes_decodable {
    ($var_type: ident) => {
        impl<'a> BytesDecodable<'a, $var_type> for $var_type {
            fn from_bytes(bytes: &'a [u8], offset: &mut usize) -> Option<Self> {
                bytes.read_with(offset, LE).ok()
            }
        }
    }
}
#[macro_export]
macro_rules! impl_bytes_decodable_lt {
    ($var_type: ident) => {
        impl<'a> BytesDecodable<'a, $var_type<'a>> for $var_type<'a> {
            fn from_bytes(bytes: &'a [u8], offset: &mut usize) -> Option<Self> {
                bytes.read_with(offset, LE).ok()
            }
        }
    }
}

#[macro_export]
macro_rules! impl_decodable {
    ($var_type: ident, $byte_len: expr) => {
        impl_bytes_decodable!($var_type);

        impl<'a> ConstDecodable<'a, $var_type> for $var_type {
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            fn from_const(bytes: *const u8) -> Option<Self> {
                let safe_bytes = unsafe { slice::from_raw_parts(bytes, $byte_len) };
                safe_bytes.read_with::<Self>(&mut 0, LE).ok()
            }
        }
        impl<'a> MutDecodable<'a, $var_type> for $var_type {
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            fn from_mut(bytes: *mut u8) -> Option<Self> {
                let safe_bytes = unsafe { slice::from_raw_parts_mut(bytes, $byte_len) };
                safe_bytes.read_with::<Self>(&mut 0, LE).ok()
            }
        }
    }
}

#[macro_export]
macro_rules! define_try_from_bytes {
    ($var_type: ident) => {
        impl AsRef<[u8]> for $var_type {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }
        impl From<&[u8]> for $var_type {
            fn from(value: &[u8]) -> Self {
                value.read_with::<$var_type>(&mut 0, byte::LE).unwrap()
            }
        }
        impl From<Vec<u8>> for $var_type {
            fn from(value: Vec<u8>) -> Self {
                value.read_with::<$var_type>(&mut 0, byte::LE).unwrap()
            }
        }
        impl From<&Vec<u8>> for $var_type {
            fn from(value: &Vec<u8>) -> Self {
                value.read_with::<$var_type>(&mut 0, byte::LE).unwrap()
            }
        }
    }
}

#[macro_export]
macro_rules! define_try_read_to_big_uint {
    ($uint_type: ident, $byte_len: expr) => {
        impl<'a> TryRead<'a, Endian> for $uint_type {
            fn try_read(bytes: &'a [u8], endian: Endian) -> Result<(Self, usize)> {
                // Ok(($uint_type(bytes[..$byte_len].try_into().unwrap_or([0u8; $byte_len])), $byte_len))
                let mut offset = 0usize;
                let mut data: [u8; $byte_len] = [0u8; $byte_len];
                for i in 0..$byte_len {
                    data[i] = bytes.read_with::<u8>(&mut offset, endian)?;
                }
                Ok(($uint_type(data), $byte_len))
            }
        }
    }
}

#[cfg(feature = "generate-dashj-tests")]
#[macro_export]
macro_rules! define_serde_big_uint {
    ($uint_type: ident) => {
        impl Serialize for $uint_type {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
                let str = self.0.to_hex();
                serializer.serialize_str(&str)
            }
        }

        impl<'de> Deserialize<'de> for $uint_type {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error> where D: Deserializer<'de> {
                String::deserialize(deserializer)
                    .map(|s| $uint_type::from_hex(s.as_str()).unwrap())
            }
        }
    }
}

#[macro_export]
macro_rules! define_try_write_from_big_uint {
    ($uint_type: ident) => {
        impl TryWrite<Endian> for $uint_type {
            fn try_write(self, mut bytes: &mut [u8], endian: Endian) -> byte::Result<usize> {
                bytes.write_all(&self.0).unwrap();
                Ok(self.0.len())
            }
        }
    }
}

#[macro_export]
macro_rules! define_bytes_to_big_uint {
    ($uint_type: ident, $byte_len: expr) => {
        impl_random!($uint_type, $byte_len);
        define_try_read_to_big_uint!($uint_type, $byte_len);
        define_try_write_from_big_uint!($uint_type);
        impl_decodable!($uint_type, $byte_len);
        define_try_from_bytes!($uint_type);
        impl_ffi_bytearray!($uint_type);
        // impl_ffi_conversion!($uint_type, $byte_len);
        #[cfg(feature = "generate-dashj-tests")]
        define_serde_big_uint!($uint_type);

        impl std::default::Default for $uint_type {
            fn default() -> Self {
                let data: [u8; $byte_len] = [0u8; $byte_len];
                Self(data)
            }
        }

        impl std::fmt::Display for $uint_type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0.to_hex())?;
                Ok(())
            }
        }
        // Used for code generation sometime while debugging
        // impl std::fmt::Debug for $uint_type {
        //     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        //         write!(f, "{}::from_hex(\"{}\").unwrap()", stringify!($uint_type), self.0.to_hex())?;
        //         Ok(())
        //     }
        // }
        impl std::fmt::Debug for $uint_type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.0.to_hex())?;
                Ok(())
            }
        }
        impl Encodable for $uint_type {
            #[inline]
            fn consensus_encode<W: std::io::Write>(&self, mut writer: W) -> std::result::Result<usize, std::io::Error> {
                writer.emit_slice(&self.as_bytes())?;
                Ok($byte_len)
            }
        }

        impl Decodable for $uint_type {
            #[inline]
            fn consensus_decode<D: std::io::Read>(mut d: D) -> std::result::Result<Self, crate::consensus::encode::Error> {
                let mut ret = [0; $byte_len];
                d.read_slice(&mut ret)?;
                Ok($uint_type(ret))
            }
        }


        // TODO: as it's often use to compare hashes
        // it's needs to be optimized
        impl Reversable for $uint_type {
            fn reverse(&mut self) -> Self {
                self.0.reverse();
                *self
            }
            fn reversed(&self) -> Self {
                let mut s = self.0.clone();
                s.reverse();
                $uint_type(s)
            }
        }
        impl FromHex for $uint_type {
            fn from_byte_iter<I>(iter: I) -> std::result::Result<Self, hex::Error>
                where I: Iterator<Item=std::result::Result<u8, hashes::hex::Error>> +
                ExactSizeIterator +
                DoubleEndedIterator {
                if iter.len() == $byte_len {
                    let mut ret = [0; $byte_len];
                    for (n, byte) in iter.enumerate() {
                        ret[n] = byte?;
                    }
                    Ok($uint_type(ret))
                } else {
                    Err(hex::Error::InvalidLength(2 * $byte_len, 2 * iter.len()))
                }
            }
        }

        impl Zeroable for $uint_type {
            fn is_zero(&self) -> bool {
                !self.0.iter().any(|&byte| byte > 0)
            }
        }

        impl $uint_type {
            pub const MIN: Self = $uint_type([0; $byte_len]);
            pub const MAX: Self = $uint_type([!0; $byte_len]);
            pub const SIZE: usize = $byte_len;
        }

        impl AsBytes for $uint_type {
            fn as_bytes(&self) -> &[u8] {
                &self.0[..]
            }
        }
    }
}


impl_decodable!(u8, 1);
impl_decodable!(u16, 2);
impl_decodable!(u32, 4);
impl_decodable!(u64, 8);
impl_decodable!(usize, mem::size_of::<usize>());
impl_decodable!(i8, 1);
impl_decodable!(i16, 2);
impl_decodable!(i32, 4);
impl_decodable!(i64, 8);
impl_decodable!(isize, mem::size_of::<isize>());


define_bytes_to_big_uint!(UInt128, 16);
define_bytes_to_big_uint!(UInt160, 20);
define_bytes_to_big_uint!(UInt256, 32);
define_bytes_to_big_uint!(UInt384, 48);
define_bytes_to_big_uint!(UInt512, 64);
define_bytes_to_big_uint!(UInt768, 96);

define_bytes_to_big_uint!(ECPoint, 33);

pub const fn merge<const N: usize>(mut buf: [u8; N], bytes: &[u8]) -> [u8; N] {
    let mut i = 0;
    while i < bytes.len() {
        buf[i] = bytes[i];
        i += 1;
    }
    buf
}

pub fn clone_into_array<A, T>(slice: &[T]) -> A where A: Default + AsMut<[T]>, T: Clone {
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

impl From<u32> for UInt256 {
    fn from(value: u32) -> Self {
        let mut r = [0u8; 32];
        r[..4].copy_from_slice(&value.to_le_bytes());
        UInt256(r)
    }
}

impl From<u64> for UInt256 {
    fn from(value: u64) -> Self {
        let mut r = [0u8; 32];
        r[..8].copy_from_slice(&value.to_le_bytes());
        UInt256(r)
    }
}

impl From<[u64; 4]> for UInt256 {
    fn from(value: [u64; 4]) -> Self {
        let mut r = [0u8; 32];
        r[..8].copy_from_slice(&value[0].to_le_bytes());
        r[8..16].copy_from_slice(&value[1].to_le_bytes());
        r[16..24].copy_from_slice(&value[2].to_le_bytes());
        r[24..].copy_from_slice(&value[3].to_le_bytes());
        UInt256(r)
    }
}

impl std::ops::Shr<usize> for UInt256 {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        let len = self.0.len();
        let (a, b) = self.0.split_at(len - rhs);
        let mut spun = [0u8; 32];
        spun[0..rhs].copy_from_slice(b);
        spun[rhs..len].copy_from_slice(a);
        Self(spun)
    }
}

impl std::ops::Shl<usize> for UInt256 {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        let len = self.0.len();
        let (a, b) = self.0.split_at(rhs);
        let mut spun = [0u8; 32];
        spun[0..rhs].copy_from_slice(b);
        spun[rhs..len].copy_from_slice(a);
        Self(spun)
    }
}

pub fn add_one_le(a: UInt256) -> UInt256 {
    let mut r = [0u8; 32];
    r[0..8].copy_from_slice(&1u64.to_le_bytes());
    add_le(a, UInt256(r))
}

pub fn add_le(x: UInt256, a: UInt256) -> UInt256 {
    let mut carry = 0u64;
    let mut r = [0u8; 32];
    for i in 0..8 {
        let len = i + 4;
        let xb: [u8; 4] = clone_into_array(&x.0[i..len]);
        let ab: [u8; 4] = clone_into_array(&a.0[i..len]);
        let sum = u32::from_le_bytes(xb) as u64 + u32::from_le_bytes(ab) as u64 + carry;
        r[i..len].copy_from_slice(&(sum as u32).to_le_bytes());
        carry = sum >> 32;
    }
    UInt256(r)
}



fn multiply_u32_le(mut a: UInt256, b: u32) -> UInt256 {
    let mut carry = 0u64;
    for i in 0..8 {
        let len = i + 4;
        let ab: [u8; 4] = clone_into_array(&a.0[i..len]);
        let n = carry + (b as u64) * (u32::from_le_bytes(ab) as u64);
        a.0[i..len].copy_from_slice(&(n as u32 & 0xffffffff).to_le_bytes());
        carry = n >> 32;
    }
    return a;
}

pub fn shift_left_le(a: UInt256, bits: u8) -> UInt256 {
    let mut r = [0u8; 32];
    let k = bits / 8;
    let bits = bits % 8;
    for i in 0..32 {
        let ik = i + k as usize;
        let ik1 = ik + 1;
        let u8s = a.0[i];
        if ik1 < 32 && bits != 0 {
            r[ik1] |= u8s >> (8 - bits);
        }
        if ik < 32 {
            r[ik] |= u8s << bits;
        }
    }
    UInt256(r)
}

fn shift_right_le(a: UInt256, bits: u8) -> UInt256 {
    let mut r = [0u8; 32];
    let k = bits / 8;
    let bits = bits % 8;
    for i in 0..32 {
        let ik = i - k as isize;
        let ik1 = ik - 1;
        let u8s = a.0[i as usize];
        if ik1 >= 0 && bits != 0 {
            r[ik1 as usize] |= u8s << (8 - bits);
        }
        if ik >= 0 {
            r[ik as usize] |= u8s >> bits;
        }
    }
    UInt256(r)
}

impl UInt160 {
    pub fn hash160(data: &[u8]) -> Self {
        UInt160(hash160::Hash::hash(data).into_inner())
    }
    pub fn hash160u32le(data: &[u8]) -> u32 {
        Self::hash160(data).u32_le()
    }
    pub fn ripemd160(data: &[u8]) -> Self {
        UInt160(ripemd160::Hash::hash(data).into_inner())
    }
    pub fn sha1(data: &[u8]) -> Self {
        UInt160(sha1::Hash::hash(data).into_inner())
    }

    pub fn u32_le(&self) -> u32 {
        u32::from_le_bytes(clone_into_array(&self.0[..4]))
    }
}

impl UInt128 {
    pub fn ip_address_from_u32(value: u32) -> Self {
        //UInt128 address = {.u32 = {0, 0, CFSwapInt32HostToBig(0xffff), CFSwapInt32HostToBig(self.address)}};
        let mut writer = Vec::<u8>::new();
        0u64.enc(&mut writer);
        0xffffu32.swap_bytes().enc(&mut writer);
        value.swap_bytes().enc(&mut writer);
        UInt128(clone_into_array(&writer))
    }

    pub fn ip_address_to_i32(&self) -> i32 {
        // todo: check impl
        // if (p.address.u64[0] != 0 || p.address.u32[2] != CFSwapInt32HostToBig(0xffff)) continue; // skip IPv6 for now
        // CFSwapInt32BigToHost(p.address.u32[3])
        i32::from_be_bytes(clone_into_array(&self.0[12..]))
    }

    pub fn to_ip_addr(&self) -> IpAddr {
        IpAddr::from(self.0)
    }

    pub fn to_ipv4_addr(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.ip_address_to_i32() as u32)
    }
}

impl From<IpAddr> for UInt128 {
    fn from(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(ipv4) => {
                let mut writer = [0u8; 16];
                writer[8..12].copy_from_slice(&0xffffu32.to_be_bytes());
                writer[12..].copy_from_slice(&ipv4.octets());
                UInt128(writer)
            },
            IpAddr::V6(ipv6) => UInt128(ipv6.octets())
        }
    }
}

impl UInt256 {
    pub fn sha256(data: &[u8]) -> Self {
        UInt256(sha256::Hash::hash(data).into_inner())
    }
    pub fn sha256_str(data: &str) -> Self {
        UInt256(sha256::Hash::hash(data.as_bytes()).into_inner())
    }
    pub fn sha256d(data: impl AsRef<[u8]>) -> Self {
        UInt256(sha256d::Hash::hash(data.as_ref()).into_inner())
    }
    pub fn sha256d_str(data: &str) -> Self {
        UInt256(sha256d::Hash::hash(data.as_bytes()).into_inner())
    }
    pub fn x11_hash(data: &[u8]) -> Self {
        let hash = rs_x11_hash::get_x11_hash(&data);
        UInt256(hash)
    }

    pub fn block_hash_for_dev_net_genesis_block_with_version(version: u32, prev_hash: UInt256, merkle_root: UInt256, timestamp: u32, target: u32, nonce: u32) -> Self {
        let mut writer = Vec::<u8>::new();
        version.enc(&mut writer);
        prev_hash.enc(&mut writer);
        merkle_root.enc(&mut writer);
        timestamp.enc(&mut writer);
        target.enc(&mut writer);
        nonce.enc(&mut writer);
        Self::x11_hash(&writer)
    }
}

impl UInt256 {
    pub fn add_le_u32(&self, a: UInt256) -> UInt256 {
        let mut carry = 0u64;
        let mut r = [0u8; 32];
        for i in 0..8 {
            let len = i + 4;
            let xb: [u8; 4] = clone_into_array(&self.0[i..len]);
            let ab: [u8; 4] = clone_into_array(&a.0[i..len]);
            let sum = u32::from_le_bytes(xb) as u64 + u32::from_le_bytes(ab) as u64 + carry;
            r[i..len].copy_from_slice(&(sum as u32).to_le_bytes());
            carry = sum >> 32;
        }
        UInt256(r)
    }

    pub fn add_le(&self, a: UInt256) -> UInt256 {
        let mut carry = 0u64;
        let mut r = [0u8; 32];
        for i in 0..8 {
            let ix = i * 4;
            let len = ix + 4;
            let xb: [u8; 4] = clone_into_array(&self.0[ix..len]);
            let ab: [u8; 4] = clone_into_array(&a.0[ix..len]);
            let sum = u32::from_le_bytes(xb) as u64 + u32::from_le_bytes(ab) as u64 + carry;
            r[ix..len].copy_from_slice(&(sum as u32).to_le_bytes());
            carry = sum >> 32;
        }
        UInt256(r)
    }

    pub fn add_be(&self, a: UInt256) -> UInt256 {
        self.reversed().add_le(a.reversed()).reverse()
    }

    // add 1u64
    pub fn add_one_le(&self) -> UInt256 {
        let mut r = [0u8; 32];
        r[..8].copy_from_slice(&1u64.to_le_bytes());
        let one = UInt256(r);
        self.add_le(one)
    }

    pub fn neg_le(&self) -> UInt256 {
        let mut r = [0u8; 32];
        for i in 0..32 {
            r[i] = !self.0[i];
        }
        UInt256(r)
    }

    pub fn subtract_le(&self, rhs: UInt256) -> UInt256 {
        self.add_le(rhs.neg_le().add_one_le())
    }

    pub fn subtract_be(&self, rhs: UInt256) -> UInt256 {
        self.clone().reversed().add_le(rhs.clone().reversed().neg_le().add_one_le()).reversed()
    }

    pub fn shift_left_le(&self, bits: u8) -> UInt256 {
        let mut r = [0u8; 32];
        let k = bits / 8;
        let bits = bits % 8;
        for i in 0..32 {
            let ik = i + k as usize;
            let ik1 = ik + 1;
            let u8s = self.0[i];
            if ik1 < 32 && bits != 0 {
                r[ik1] |= u8s >> (8 - bits);
            }
            if ik < 32 {
                r[ik] |= u8s << bits;
            }
        }
        UInt256(r)
    }

    pub fn shift_right_le(&self, bits: u8) -> UInt256 {
        let mut r = [0u8; 32];
        let k = bits / 8;
        let bits = bits % 8;
        for i in 0..32 {
            let ik = i - k as isize;
            let ik1 = ik - 1;
            let u8s = self.0[i as usize];
            if ik1 >= 0 && bits != 0 {
                r[ik1 as usize] |= u8s << (8 - bits);
            }
            if ik >= 0 {
                r[ik as usize] |= u8s >> bits;
            }
        }
        UInt256(r)
    }

    pub fn multiply_u32_le(&self, b: u32) -> UInt256 {
        let mut r = [0u8; 32];
        let mut carry = 0u64;
        for i in 0..8 {
            let len = i + 4;
            let ab: [u8; 4] = clone_into_array(&self.0[i..len]);
            let n = carry + (b as u64) * (u32::from_le_bytes(ab) as u64);
            r[i..len].copy_from_slice(&(n as u32 & 0xffffffff).to_le_bytes());
            carry = n >> 32;
        }
        UInt256(r)
    }

    // #define uint256_is_31_bits(u) ((((u).u64[1] | (u).u64[2] | (u).u64[3]) == 0) && ((u).u32[1] == 0) && (((u).u32[0] & 0x80000000) == 0))
    pub fn is_31_bits(&self) -> bool {
        let u64_1 = u64::from_le_bytes(clone_into_array(&self.0[8..16]));
        let u64_2 = u64::from_le_bytes(clone_into_array(&self.0[16..24]));
        let u64_3 = u64::from_le_bytes(clone_into_array(&self.0[24..]));
        let u32_1 = u32::from_le_bytes(clone_into_array(&self.0[4..8]));
        let u32_0 = u32::from_le_bytes(clone_into_array(&self.0[..4]));
        (u64_1 | u64_2 | u64_3) == 0 && u32_1 == 0 && u32_0 & 0x80000000 == 0
    }

    pub fn sup(&self, rhs: &UInt256) -> bool {
        for i in (0..32).rev() {
            if self.0[i] > rhs.0[i] {
                return true;
            } else if self.0[i] < rhs.0[i] {
                return false;
            }
        }
        // equal
        return false;
    }

    pub fn supeq(&self, rhs: &UInt256) -> bool {
        for i in (0..32).rev() {
            if self.0[i] > rhs.0[i] {
                return true;
            } else if self.0[i] < rhs.0[i] {
                return false;
            }
        }
        // equal
        return true;
    }

    pub fn xor(&self, rhs: &UInt256) -> UInt256 {
        let mut r = [0u8; 32];
        for i in 0..32 {
            r[i] = self.0[i] ^ rhs.0[i];
        }
        UInt256(r)
    }

    pub fn inverse(&self) -> UInt256 {
        self.xor(&UInt256::MAX)
    }

    pub fn divide_le(&self, rhs: UInt256) -> UInt256 {
        let mut r = UInt256::MIN; // the quotient
        let mut num = self.clone();
        let mut div = rhs.clone();
        let num_bits = num.compact_bits_le();
        let div_bits = div.compact_bits_le();
        assert_ne!(div_bits, 0, "");
        if div_bits > num_bits {
            // the result is certainly 0
            return r;
        } else {
            let mut shift = (num_bits - div_bits) as isize;
            div = div.shift_left_le(shift as u8); // shift so that div and nun align
            while shift >= 0 {
                if num.supeq(&div) {
                    num = num.subtract_le(div);
                    r.0[(shift / 8) as usize] |= 1 << (shift & 7); // set a bit of the result
                }
                div = div.shift_right_le(1); // shift back
                shift -= 1;
            }
        }
        // num now contains the remainder of the division
        r
    }



    pub fn compact_bits_le(&self) -> u16 {
        for pos in (0..8).rev() {
            let posx = pos * 4;
            let len = posx + 4;
            let ab: [u8; 4] = clone_into_array(&self.0[posx..len]);
            let ab_32 = u32::from_le_bytes(ab);
            if ab_32 != 0 {
                for bits in (0..32).rev() {
                    if ab_32 & 1 << bits != 0 {
                        return (8 * posx + bits + 1) as u16;
                    }
                }
                return (8 * posx + 1) as u16;
            }
        }
        return 0;
    }

    pub fn get_compact_le(&self) -> i32 {
        let mut size = ((self.compact_bits_le() + 7) / 8) as u32;
        let mut compact = if size <= 3 {
            u32::from_le_bytes(clone_into_array(&self.0[..4])) << 8 * (3 - size)
        } else {
            u32::from_le_bytes(clone_into_array(&self.shift_right_le((8 * (size - 3)) as u8).0[..4]))
        };
        // The 0x00800000 bit denotes the sign.
        // Thus, if it is already set, divide the mantissa by 256 and increase the exponent.
        if compact & 0x00800000 != 0 {
            compact >>= 8;
            size += 1;
        }
        assert_eq!(compact & !0x007fffff, 0, "--1--");
        assert!(size < 256, "--2--");
        compact |= size << 24;
        compact as i32
    }

    pub fn set_compact_le(compact: i32) -> UInt256 {
        let size = compact >> 24;
        let mut word = Self::MIN;
        word.0[..4].copy_from_slice(&(compact as i32 & 0x007fffff).to_le_bytes());
        if size <= 3 {
            word = word.shift_right_le((8 * (3 - size)) as u8);
        } else {
            word = word.shift_left_le((8 * (size - 3)) as u8);
        }
        word
    }

    pub fn set_compact_be(compact: i32) -> UInt256 {
        Self::set_compact_le(compact).reversed()
    }

    pub fn u32_le(&self) -> u32 {
        u32::from_le_bytes(clone_into_array(&self.0[..4]))
    }

    pub fn u64_le(&self) -> u64 {
        u64::from_le_bytes(clone_into_array(&self.0[..8]))
    }
    pub fn u64_2_le(&self) -> u64 {
        u64::from_le_bytes(clone_into_array(&self.0[8..16]))
    }
    pub fn u64_3_le(&self) -> u64 {
        u64::from_le_bytes(clone_into_array(&self.0[16..24]))
    }
    pub fn u64_4_le(&self) -> u64 {
        u64::from_le_bytes(clone_into_array(&self.0[24..]))
    }
}

impl UInt256 {
    pub fn short_hex(&self) -> String {
        short_hex_string_from(self.as_bytes())
    }
}

impl UInt256 {
    pub fn from_base58_string(data: &str) -> Option<Self> {
        base58::from(data)
            .ok()
            .and_then(|d| Self::from_bytes(&d, &mut 0))
    }
}

impl UInt256 {
    pub fn hmac<T: Hash<Inner = [u8; 32]>>(key: &[u8], input: &[u8]) -> Self {
        let mut engine = HmacEngine::<T>::new(key);
        engine.input(input);
        Self(Hmac::<T>::from_engine(engine).into_inner())
    }
}

impl secp256k1::ThirtyTwoByteHash for UInt256 {
    fn into_32(self) -> [u8; 32] {
        self.0
    }
}

impl From<SigningKey> for UInt256 {
    fn from(value: SigningKey) -> Self {
        UInt256(value.to_bytes())
    }
}

impl From<UInt256> for SigningKey {
    fn from(value: UInt256) -> Self {
        SigningKey::from_bytes(&value.0)
    }
}

impl From<VerifyingKey> for UInt256 {
    fn from(value: VerifyingKey) -> Self {
        UInt256(value.to_bytes())
        // let mut data = [0u8; 33];
        // data[1..33].copy_from_slice(value.as_bytes());
        // Self(data)
    }
}
impl From<VerifyingKey> for ECPoint {
    fn from(value: VerifyingKey) -> Self {
        let mut data = [0u8; 33];
        data[1..33].copy_from_slice(value.as_bytes());
        Self(data)
    }
}

impl UInt512 {
    pub fn sha512(data: &[u8]) -> Self {
        UInt512(sha512::Hash::hash(data).into_inner())
    }
    pub fn hmac(key: &[u8], input: &[u8]) -> Self {
        let mut engine = HmacEngine::<sha512::Hash>::new(key);
        engine.input(input);
        Self(Hmac::<sha512::Hash>::from_engine(engine).into_inner())
    }

    pub fn bip32_seed_key(input: &[u8]) -> Self {
        Self::hmac(BIP32_SEED_KEY.as_bytes(), input)
    }

    pub fn ed25519_seed_key(input: &[u8]) -> Self {
        Self::hmac(ED25519_SEED_KEY.as_bytes(), input)
    }

    pub fn from(a: UInt256, b: UInt256) -> Self {
        let mut result = [0u8; 64];
        result[..32].copy_from_slice(&a.0);
        result[32..].copy_from_slice(&b.0);
        Self(result)
    }
}
