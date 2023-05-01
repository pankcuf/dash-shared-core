use byte::{BytesExt, TryRead};
use byte::ctx::{NULL, Str};
use crate::chain::network::MessageType;
use crate::crypto::{byte_util::Zeroable, UInt256};

#[derive(Clone, Debug, Default)]
pub struct Reject {
    pub r#type: MessageType,
    pub code: u8,
    pub reason: String,
    pub hash: Option<UInt256>,
}

impl<'a> TryRead<'a, ()> for Reject {
    fn try_read(bytes: &'a [u8], nothing: ()) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let r#type = bytes.read_with::<MessageType>(offset, byte::LE).unwrap();
        let code = bytes.read_with::<u8>(offset, byte::LE).unwrap();
        let reason = bytes.read_with::<&str>(offset, Str::Delimiter(NULL)).unwrap().to_string();
        let tx_hash = if r#type == MessageType::Tx || r#type == MessageType::Ix {
            let t = bytes.read_with::<UInt256>(offset, byte::LE).unwrap();
            (!t.is_zero()).then_some(t)
        } else {
            None
        };
        Ok((Self { r#type, code, reason, hash: tx_hash }, *offset))
    }
}
