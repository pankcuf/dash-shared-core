use byte::ctx::Endian;
use byte::{BytesExt, TryRead};

#[derive(Clone, Debug, Default)]
pub enum VoteSignal {
    #[default]
    None = 0,
    Funding = 1,
    Valid = 2,
    Delete = 3,
    Endorsed = 4
}

impl From<u32> for VoteSignal {
    fn from(orig: u32) -> Self {
        match orig {
            0 => VoteSignal::None,
            1 => VoteSignal::Funding,
            2 => VoteSignal::Valid,
            3 => VoteSignal::Delete,
            4 => VoteSignal::Endorsed,
            _ => VoteSignal::None,
        }
    }
}

impl From<VoteSignal> for u32 {
    fn from(value: VoteSignal) -> Self {
        match value {
            VoteSignal::None => 0,
            VoteSignal::Funding => 1,
            VoteSignal::Valid => 2,
            VoteSignal::Delete => 3,
            VoteSignal::Endorsed => 4,
        }
    }
}

impl<'a> TryRead<'a, Endian> for VoteSignal {
    fn try_read(bytes: &'a [u8], endian: Endian) -> byte::Result<(Self, usize)> {
        let orig = bytes.read_with::<u32>(&mut 0, endian).unwrap();
        let data = VoteSignal::from(orig);
        Ok((data, std::mem::size_of::<u32>()))
    }
}

// impl Encodable for VoteSignal {
//     fn consensus_encode<W: Write>(&self, mut writer: W) -> Result<usize, Error> {
//         // let s: u32 = <VoteSignal as Into<u32>>::into(self.clone());
//         let s: u32 = match self {
//             VoteSignal::None => 0,
//             VoteSignal::Funding => 1,
//             VoteSignal::Valid => 2,
//             VoteSignal::Delete => 3,
//             VoteSignal::Endorsed => 4
//         };
//         writer.emit_slice(&s.to_le_bytes())?;
//         Ok(std::mem::size_of::<u32>())
//
//     }
// }
