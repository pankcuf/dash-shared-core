use std::fmt::{Debug, Display};
use std::os::raw::c_ulong;
use std::slice;
use byte::{BytesExt, LE, TryRead};
use crate::consensus::Encodable;
use crate::crypto::byte_util::clone_into_array;
use crate::crypto::UInt256;
use super::BIP32_HARD;

pub trait Extremum {
    fn max() -> Self;
    fn min() -> Self;
}

impl Extremum for u32 {
    fn max() -> Self {
        u32::MAX
    }
    fn min() -> Self {
        u32::MIN
    }
}

impl Extremum for UInt256 {
    fn max() -> Self {
        UInt256::MAX
    }
    fn min() -> Self {
        UInt256::MIN
    }
}

pub trait IndexHardSoft {
    fn harden(&self) -> Self;
    fn soften(&self) -> Self;
    fn hardened(&self) -> u64;
    fn softened(&self) -> u64;
}

impl IndexHardSoft for u32 {
    fn harden(&self) -> Self {
        self | BIP32_HARD
    }
    fn soften(&self) -> Self {
        self & !BIP32_HARD
    }
    fn hardened(&self) -> u64 {
        self.harden() as u64
    }
    fn softened(&self) -> u64 {
        *self as u64
    }
}

impl IndexHardSoft for UInt256 {
    fn harden(&self) -> Self {
        let mut v = [0u8; 32];
        for i in 0..8 {
            let start = i << 2;
            let end = start + 4;
            let hard = u32::from_le_bytes(clone_into_array(&self.0[start..end])) | BIP32_HARD;
            v[start..end].copy_from_slice(&hard.to_le_bytes())
        }
        UInt256(v)
    }

    fn soften(&self) -> Self {
        let mut v = [0u8; 32];
        for i in 0..8 {
            let start = i << 2;
            let end = start + 4;
            let hard = u32::from_le_bytes(clone_into_array(&self.0[start..end])) & !BIP32_HARD;
            v[start..end].copy_from_slice(&hard.to_le_bytes())
        }
        UInt256(v)
    }

    fn hardened(&self) -> u64 {
        self.u64_le() | BIP32_HARD as u64
    }

    fn softened(&self) -> u64 {
        self.u64_le()
    }
}

pub trait IIndexPath: Sized {
    type Item: Copy + Clone + Display + Debug + Encodable + IndexHardSoft + PartialEq + Extremum;

    fn new(indexes: Vec<Self::Item>) -> Self;
    fn new_hardened(indexes: Vec<Self::Item>, hardened: Vec<bool>) -> Self;
    fn index_path_with_index(index: Self::Item) -> Self {
        Self::new(vec![index])
    }
    fn index_path_with_indexes(indexes: Vec<Self::Item>) -> Self {
        Self::new(indexes)
    }
    fn base_index_path(&self) -> IndexPath<u32> {
        IndexPath::index_path_with_indexes(
            (0..self.indexes().len())
                .into_iter()
                .map(|position| self.index_u64_at_position(position) as u32)
                .collect())
    }
    fn last_index(&self) -> Self::Item {
        self.index_at_position(self.length() - 1)
    }
    fn last_hardened(&self) -> bool {
        self.hardened_at_position(self.length() - 1)
    }
    fn indexes(&self) -> &Vec<Self::Item>;
    fn hardened_indexes(&self) -> &Vec<bool>;
    fn index_at_position(&self, position: usize) -> Self::Item {
        if position >= self.length() {
            Self::Item::max()
        } else {
            self.indexes()[position]
        }
    }
    fn hardened_at_position(&self, position: usize) -> bool {
        if position >= self.length() {
            false
        } else {
            self.hardened_indexes()[position]
        }
    }
    fn terminal_hardened(&self) -> bool {
        self.hardened_at_position(self.hardened_indexes().len() - 1)
    }

    fn index_u64_at_position(&self, position: usize) -> u64 {
        if self.hardened_at_position(position) {
            self.index_at_position(position).hardened()
        } else {
            self.index_at_position(position).softened()
        }
    }

    fn is_empty(&self) -> bool {
        self.indexes().is_empty()
    }
    fn length(&self) -> usize {
        self.indexes().len()
    }
    fn index_path_string(&self) -> String {
        if self.is_empty() {
            format!("")
        } else {
            self.indexes().into_iter().map(|index| format!("{}", index)).collect::<Vec<_>>().join(".")
        }
    }
    fn index_path_enumerated_string(&self) -> String {
        (0..self.length())
            .map(|position| format!("_{}", self.index_u64_at_position(position)))
            .collect::<Vec<_>>()
            .join(".")
    }

    fn harden_all_items(&self) -> IndexPath<Self::Item> {
        IndexPath::index_path_with_indexes(self.indexes().iter().map(Self::Item::harden).collect())
    }
    fn soften_all_items(&self) -> IndexPath<Self::Item> {
        IndexPath::index_path_with_indexes(self.indexes().iter().map(Self::Item::soften).collect())
    }
    fn index_path_by_adding_index(&mut self, index: Self::Item) -> Self {
        // TODO: impl optimized version
        // let size = mem::size_of::<Self::Item>();
        // let memory_size = (self.length() + 1)  * size;
        if self.is_empty() {
            Self::index_path_with_indexes(vec![index])
        } else {
            let mut indexes = self.indexes().clone();
            indexes.push(index);
            Self::index_path_with_indexes(indexes)
        }
    }
    fn index_path_by_removing_first_index(&self) -> Self {
        Self::index_path_with_indexes(if self.indexes().len() <= 1 {
            vec![]
        } else {
            self.indexes()[1..].to_vec()
        })
    }
    fn index_path_by_removing_last_index(&self) -> Self {
        let len = self.length();
        Self::index_path_with_indexes(if len <= 1 {
            vec![]
        } else {
            self.indexes()[..len].to_vec()
        })
    }

    fn as_bytes_vec(&self) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        self.indexes().iter().for_each(|index| {
            index.enc(&mut writer);
        });
        writer
    }
}

#[derive(Debug, Default)]
pub struct IndexPath<T> {
    pub indexes: Vec<T>,
    pub hardened: Vec<bool>,
}

impl<T> IIndexPath for IndexPath<T> where T: Copy + Debug + Display + Encodable + IndexHardSoft + PartialEq + Extremum {
    type Item = T;
    // TODO: avoid hardened allocation for u32 index paths
    fn new(indexes: Vec<Self::Item>) -> Self {
        Self::new_hardened(indexes, vec![])
    }

    fn new_hardened(indexes: Vec<Self::Item>, hardened: Vec<bool>) -> Self {
        Self { indexes, hardened }
    }

    fn indexes(&self) -> &Vec<Self::Item> {
        &self.indexes
    }

    fn hardened_indexes(&self) -> &Vec<bool> {
        &self.hardened
    }
}

impl IndexPath<u32> {
    pub fn from_ffi(indexes: *const c_ulong, length: usize) -> Self {
        let indexes_slice = unsafe { slice::from_raw_parts(indexes, length) };
        IndexPath::new(indexes_slice.iter().map(|&index| index as u32).collect())
    }
}

impl<'a> TryRead<'a, usize> for IndexPath<UInt256> {
    #[inline]
    fn try_read(bytes: &'a [u8], size: usize) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let mut indexes = Vec::with_capacity(size);
        let mut hardened = Vec::with_capacity(size);
        for i in 0..size {
            indexes.push(bytes.read_with::<UInt256>(offset, LE)?);
            hardened.push(bytes.read_with::<bool>(offset, ())?);
        }
        Ok((Self::new_hardened(indexes, hardened), size))
    }
}
