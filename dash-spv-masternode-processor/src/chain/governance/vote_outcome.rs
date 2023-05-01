use byte::ctx::Endian;
use byte::{BytesExt, TryRead};

#[derive(Clone, Debug, Default)]
pub enum VoteOutcome {
    #[default]
    None = 0,
    Yes = 1,
    No = 2,
    Abstain = 3
}

impl From<u32> for VoteOutcome {
    fn from(orig: u32) -> Self {
        match orig {
            0 => VoteOutcome::None,
            1 => VoteOutcome::Yes,
            2 => VoteOutcome::No,
            3 => VoteOutcome::Abstain,
            _ => VoteOutcome::None,
        }
    }
}

impl From<VoteOutcome> for u32 {
    fn from(value: VoteOutcome) -> Self {
        match value {
            VoteOutcome::None => 0,
            VoteOutcome::Yes => 1,
            VoteOutcome::No => 2,
            VoteOutcome::Abstain => 3,
        }
    }
}

impl<'a> TryRead<'a, Endian> for VoteOutcome {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let orig = bytes.read_with::<u32>(&mut 0, endian).unwrap();
        let data = VoteOutcome::from(orig);
        Ok((data, std::mem::size_of::<u32>()))
    }
}

// impl Encodable for VoteOutcome {
//     fn consensus_encode<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
//         let s: u32 = *self.into();
//         s.enc(&mut writer);
//         Ok(std::mem::size_of::<u32>())
//     }
// }
