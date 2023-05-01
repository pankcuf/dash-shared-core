use byte::BytesExt;
use hashes::hex::FromHex;
use crate::chain::chain_lock;
use crate::chain::chain_lock::ChainLock;
use crate::chain::common::ChainType;
use crate::consensus::Encodable;
use crate::crypto::{UInt256, UInt768};
use crate::util::Shared;

#[test]
fn test_chain_lock_deserialization() {
    let read_context = chain_lock::ReadContext(ChainType::MainNet, Shared::None);
    let mut writer = Vec::<u8>::new();
    1177907u32.enc(&mut writer);
    UInt256::from_hex("0000000000000027b4f24c02e3e81e41e2ec4db8f1c42ee1f3923340a22680ee").unwrap().enc(&mut writer);
    UInt768::from_hex("8ee1ecc07ee989230b68ccabaa95ef4c6435e642a61114595eb208cb8bfad5c8731d008c96e62519cb60a642c4999c880c4b92a73a99f6ff667b0961eb4b74fc1881c517cf807c8c4aed2c6f3010bb33b255ae75b7593c625e958f34bf8c02be").unwrap().enc(&mut writer);
    let chain_lock = writer.as_slice().read_with::<ChainLock>(&mut 0, read_context).unwrap();
    assert_eq!(chain_lock.get_request_id(), UInt256::from_hex("f79d7cee1eea5839d91da7921920f19258e08b51c7cda01086e52d1b1d86510c").unwrap(), "request ids don't match");
}
