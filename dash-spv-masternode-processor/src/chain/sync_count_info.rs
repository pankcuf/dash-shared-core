use byte::ctx::Endian;
use byte::{BytesExt, TryRead};

#[derive(Debug, Copy, Clone)]
pub enum SyncCountInfo {
    Unknown = 0,
    List = 2,
    MNW = 3,
    GovernanceObject = 10,
    GovernanceObjectVote = 11,
}

impl From<u32> for SyncCountInfo {
    fn from(orig: u32) -> Self {
        match orig {
            2 => SyncCountInfo::List,
            3 => SyncCountInfo::MNW,
            10 => SyncCountInfo::GovernanceObject,
            11 => SyncCountInfo::GovernanceObjectVote,
            _ => SyncCountInfo::Unknown,
        }
    }
}

impl From<SyncCountInfo> for u32 {
    fn from(value: SyncCountInfo) -> Self {
        match value {
            SyncCountInfo::List => 2,
            SyncCountInfo::MNW => 3,
            SyncCountInfo::GovernanceObject => 10,
            SyncCountInfo::GovernanceObjectVote => 11,
            _ => 0
        }
    }
}

impl<'a> TryRead<'a, Endian> for SyncCountInfo {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let orig = bytes.read_with::<u32>(&mut 0, endian).unwrap();
        let data = SyncCountInfo::from(orig);
        Ok((data, std::mem::size_of::<u32>()))
    }
}
