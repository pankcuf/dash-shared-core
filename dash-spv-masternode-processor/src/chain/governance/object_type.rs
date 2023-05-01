use std::io::{Error, Write};
use byte::ctx::Endian;
use byte::{BytesExt, TryRead, TryWrite};
use crate::consensus::Encodable;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum ObjectType {
    #[default]
    Unknown = 0,
    Proposal = 1,
    Trigger = 2,
    /// deprecated
    Watchdog = 3
}

impl From<u32> for ObjectType {
    fn from(orig: u32) -> Self {
        match orig {
            0 => ObjectType::Unknown,
            1 => ObjectType::Proposal,
            2 => ObjectType::Trigger,
            3 => ObjectType::Watchdog,
            _ => ObjectType::Unknown,
        }
    }
}

impl From<ObjectType> for u32 {
    fn from(value: ObjectType) -> Self {
        match value {
            ObjectType::Unknown => 0,
            ObjectType::Proposal => 1,
            ObjectType::Trigger => 2,
            ObjectType::Watchdog => 3,
        }
    }
}

impl From<&ObjectType> for u32 {
    fn from(value: &ObjectType) -> Self {
        match value {
            ObjectType::Unknown => 0,
            ObjectType::Proposal => 1,
            ObjectType::Trigger => 2,
            ObjectType::Watchdog => 3,
        }
    }
}

impl<'a> TryRead<'a, Endian> for ObjectType {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let orig = bytes.read_with::<u32>(&mut 0, endian).unwrap();
        let data = ObjectType::from(orig);
        Ok((data, std::mem::size_of::<u32>()))
    }
}

impl TryWrite<Endian> for ObjectType {
    fn try_write(self, bytes: &mut [u8], endian: Endian) -> Result<usize, byte::Error> {
        let offset = &mut 0;
        bytes.write_with::<u32>(offset, self.into(), endian)?;
        Ok(*offset)
    }
}


impl Encodable for ObjectType {
    fn consensus_encode<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
        u32::from(self).enc(&mut writer);
        Ok(std::mem::size_of::<u32>())
    }
}

