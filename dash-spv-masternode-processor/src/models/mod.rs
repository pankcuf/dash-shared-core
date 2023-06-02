use byte::{BytesExt, LE};
use crate::crypto::byte_util::BytesDecodable;
use crate::impl_bytes_decodable;

pub mod llmq_entry;
pub mod llmq_typed_hash;
pub mod masternode_entry;
pub mod masternode_list;
pub mod mn_list_diff;
pub mod operator_public_key;
pub mod quorums_cl_sigs_object;
pub mod snapshot;

pub use self::llmq_entry::LLMQEntry;
pub use self::llmq_typed_hash::LLMQIndexedHash;
pub use self::llmq_typed_hash::LLMQTypedHash;
pub use self::masternode_entry::MasternodeEntry;
pub use self::masternode_list::MasternodeList;
pub use self::mn_list_diff::MNListDiff;
pub use self::operator_public_key::OperatorPublicKey;
pub use self::quorums_cl_sigs_object::QuorumsCLSigsObject;
pub use self::snapshot::LLMQSnapshot;

impl_bytes_decodable!(LLMQEntry);
impl_bytes_decodable!(QuorumsCLSigsObject);
