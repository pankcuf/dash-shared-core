use byte::ctx::{Bytes, Endian};
use byte::{BytesExt, TryRead, LE};
use hashes::hex::ToHex;
#[cfg(feature = "generate-dashj-tests")]
use serde::{Serialize, Serializer};
#[cfg(feature = "generate-dashj-tests")]
use serde::ser::SerializeStruct;
use crate::common::LLMQSnapshotSkipMode;
use crate::consensus::encode::VarInt;
use crate::crypto::{byte_util::BytesDecodable, data_ops::Data};
use crate::impl_bytes_decodable;
use crate::models::MasternodeEntry;

#[derive(Clone)]
pub struct LLMQSnapshot {
    // The bitset of nodes already in quarters at the start of cycle at height n
    // (masternodeListSize + 7)/8
    pub member_list: Vec<u8>,
    // Skiplist at height n
    pub skip_list: Vec<i32>,
    //  Mode of the skip list
    pub skip_list_mode: LLMQSnapshotSkipMode,
}
impl Default for LLMQSnapshot {
    fn default() -> Self {
        Self {
            member_list: vec![],
            skip_list: vec![],
            skip_list_mode: LLMQSnapshotSkipMode::NoSkipping,
        }
    }
}

#[cfg(feature = "generate-dashj-tests")]
impl Serialize for LLMQSnapshot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("LLMQSnapshot", 3)?;
        let len = self.member_list.len() * 8 - 7;
        let members = (0..len).map(|i| self.member_is_true_at_index(i as u32)).collect::<Vec<_>>();
        state.serialize_field("member_list", &members)?;
        state.serialize_field("skip_list", &self.skip_list)?;
        state.serialize_field("skip_list_mode", &self.skip_list_mode)?;
        state.end()

    }
}

impl<'a> std::fmt::Debug for LLMQSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMQSnapshot")
            .field("member_list", &self.member_list.to_hex())
            .field("skip_list", &self.skip_list.iter())
            .field("skip_list_mode", &self.skip_list_mode)
            .finish()
    }
}
impl<'a> TryRead<'a, Endian> for LLMQSnapshot {
    fn try_read(bytes: &'a [u8], _endian: Endian) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let skip_list_mode = bytes.read_with::<LLMQSnapshotSkipMode>(offset, LE)?;
        let member_list_length = bytes.read_with::<VarInt>(offset, LE)?.0 as usize;
        let member_list: &[u8] =
            bytes.read_with(offset, Bytes::Len((member_list_length + 7) / 8))?;
        let skip_list_length = bytes.read_with::<VarInt>(offset, LE)?.0 as usize;
        let mut skip_list = Vec::with_capacity(skip_list_length);
        for _i in 0..skip_list_length {
            skip_list.push(bytes.read_with::<i32>(offset, LE)?);
        }
        let snapshot = Self {
            member_list: member_list.to_vec(),
            skip_list,
            skip_list_mode,
        };
        Ok((snapshot, *offset))
    }
}

impl LLMQSnapshot {

    pub fn new(member_list: Vec<u8>, skip_list: Vec<i32>, skip_list_mode: LLMQSnapshotSkipMode) -> Self {
        LLMQSnapshot {
            member_list,
            skip_list,
            skip_list_mode
        }
    }

    pub fn length(&self) -> usize {
        self.member_list.len() + 1 + 2 + self.skip_list.len() * 2
    }

    pub fn member_is_true_at_index(&self, i: u32) -> bool {
        self.member_list.as_slice().bit_is_true_at_le_index(i)
    }

    pub fn apply_skip_strategy(
        &self,
        sorted_combined_mns_list: Vec<MasternodeEntry>,
        quorum_count: usize,
        quarter_size: usize,
    ) -> Vec<Vec<MasternodeEntry>> {
        let mut quarter_quorum_members = vec![Vec::<MasternodeEntry>::new(); quorum_count];
        match self.skip_list_mode {
            LLMQSnapshotSkipMode::NoSkipping => {
                let mut iter = sorted_combined_mns_list.iter();
                (0..quorum_count).for_each(|_i| {
                    let mut quarter = Vec::<MasternodeEntry>::new();
                    while quarter.len() < quarter_size {
                        if let Some(node) = iter.next() {
                            quarter.push(node.clone());
                        } else {
                            iter = sorted_combined_mns_list.iter();
                        }
                    }
                    quarter_quorum_members.push(quarter);
                });
            }
            LLMQSnapshotSkipMode::SkipFirst => {
                let mut first_entry_index = 0;
                let mut processed_skip_list = Vec::<i32>::new();
                for &s in &self.skip_list {
                    if first_entry_index == 0 {
                        first_entry_index = s;
                        processed_skip_list.push(s);
                    } else {
                        processed_skip_list.push(first_entry_index + s);
                    }
                }
                let mut idx = 0;
                let mut idxk = 0;
                for i in 0..quorum_count {
                    while quarter_quorum_members[i].len() < quarter_size {
                        if idxk != processed_skip_list.len() && idx == processed_skip_list[idxk] {
                            idxk += 1;
                        } else {
                            quarter_quorum_members[i].push(sorted_combined_mns_list[idx as usize].clone());
                        }
                        idx += 1;
                        if idx == sorted_combined_mns_list.len() as i32 {
                            idx = 0;
                        }
                    }
                }
            }
            LLMQSnapshotSkipMode::SkipExcept => {
                (0..quorum_count).for_each(|_i| {
                    let mut quarter = Vec::<MasternodeEntry>::new();
                    self.skip_list.iter().for_each(|unskipped| {
                        if let Some(node) = sorted_combined_mns_list.get(*unskipped as usize) {
                            if quarter.len() < quarter_size {
                                quarter.push(node.clone());
                            }
                        }
                    });
                    quarter_quorum_members.push(quarter);
                });
            }
            LLMQSnapshotSkipMode::SkipAll => {
                // TODO: do we need to impl smth in this strategy ?
            }
        }
        quarter_quorum_members
    }
}
impl_bytes_decodable!(LLMQSnapshot);
