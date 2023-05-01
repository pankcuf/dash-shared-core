use byte::ctx::Bytes;
use byte::{BytesExt, LE, TryRead};
use hashes::hex::{FromHex, ToHex};
use crate::chain::common::ChainType;
use crate::chain::constants::DASH_MESSAGE_MAGIC;
use crate::chain::spork::Identifier;
use crate::consensus::Encodable;
use crate::consensus::encode::VarInt;
use crate::crypto::UInt256;
use crate::keys::ecdsa_key::ECDSAKey;
use crate::keys::IKey;
use crate::util::address::address;

#[derive(Clone, Debug, Default)]
pub struct Spork {
    pub identifier: Identifier,
    pub is_valid: bool,
    pub time_signed: u64,
    pub value: u64,
    pub signature: Vec<u8>,
    pub chain_type: ChainType,
}

impl PartialEq for Spork {
    fn eq(&self, other: &Self) -> bool {
        self.chain_type == other.chain_type &&
            self.identifier == other.identifier &&
            self.value == other.value &&
            self.time_signed == other.time_signed &&
            self.is_valid == other.is_valid
    }
}

#[derive(Clone)]
pub struct ReadContext(pub ChainType, pub bool);

impl<'a> TryRead<'a, ReadContext> for Spork {
    fn try_read(bytes: &'a [u8], context: ReadContext) -> byte::Result<(Self, usize)> {
        let offset = &mut 0;
        let identifier = bytes.read_with::<Identifier>(offset, LE).unwrap();
        let value = bytes.read_with::<u64>(offset, LE).unwrap();
        let time_signed = bytes.read_with::<u64>(offset, LE).unwrap();
        let signature_length = bytes.read_with::<VarInt>(offset, LE).unwrap().0 as usize;
        let signature_bytes: &[u8] = bytes.read_with(offset, Bytes::Len(signature_length)).unwrap();
        let signature = signature_bytes.to_vec();
        let mut spork = Self {
            identifier,
            is_valid: false,
            time_signed,
            value,
            signature,
            chain_type: context.0,
            // chain: context.1
        };
        spork.is_valid = spork.check_signature(&spork.signature, context.1);
        // spork.check_validity();
        Ok((spork, *offset))
    }
}

impl Spork {

    // fn check_validity(&mut self) {
    //     self.is_valid = self.check_signature(&self.signature);
    // }

    pub fn is_equal_to_spork(&self, spork: &Spork) -> bool {
        self.chain_type == spork.chain_type &&
            self.identifier == spork.identifier &&
            self.value == spork.value &&
            self.time_signed == spork.time_signed &&
            self.is_valid == self.is_valid
    }

    pub fn key(&self) -> Option<String> {
        let params = self.chain_type.spork_params();
        params.public_key_hex_string
            .or(params.private_key_base58_string
                .and_then(|value| ECDSAKey::init_with_private_key(&value.to_string(), self.chain_type)
                    .map(|private_key| private_key.pubkey.to_hex())))
    }

    /// starting in 12.3 sporks use addresses instead of public keys
    pub fn address(&self) -> String {
        self.chain_type.spork_params().address.clone()
    }


    fn check_signature_70208_method(&self, signature: &Vec<u8>) -> bool {
        let string_message = format!("{:?}{}{}", self.identifier, self.value, self.time_signed);
        let mut writer: Vec<u8> = Vec::new();
        DASH_MESSAGE_MAGIC.to_string().enc(&mut writer);
        string_message.enc(&mut writer);
        let message_digest = UInt256::sha256d(writer);
        let message_public_key = ECDSAKey::init_with_compact_sig(signature, message_digest);
        let spork_public_key = ECDSAKey::init_with_public_key(Vec::from_hex(self.key().unwrap().as_str()).unwrap());
        spork_public_key.unwrap().public_key_data() == message_public_key.unwrap().public_key_data()
    }


    fn check_signature(&self, signature: &Vec<u8>, is_updated_signatures: bool) -> bool {
        if self.chain_type.protocol_version() < 70209 {
            self.check_signature_70208_method(signature)
        } else {
            let message_digest = self.calculate_spork_hash();
            let msg_public_key = ECDSAKey::init_with_compact_sig(signature, message_digest).unwrap();
            let spork_address = address::with_public_key_data(&msg_public_key.public_key_data(), &self.chain_type.script_map());
            self.address() == spork_address.as_str() || (is_updated_signatures && self.check_signature_70208_method(signature))
        }
    }

    pub fn calculate_spork_hash(&self) -> UInt256 {
        let mut writer = Vec::<u8>::new();
        let id: u16 = self.identifier.clone().into();
        id.enc(&mut writer);
        self.value.enc(&mut writer);
        self.time_signed.enc(&mut writer);
        UInt256::sha256d(writer)
    }

}

/*impl Spork {

    // pub fn update_values_with_hash<T, V>(&self, hash: &UInt256) -> Box<dyn EntityUpdates<V>>
    //     where T: Table,
    //           V: AsChangeset<Target=T> {
    pub fn update_values_with_hash(&self, hash: &UInt256) -> Box<dyn EntityUpdates<bool, ResultType = (bool, )>> {
        let mut values = self.update_values();
        Box::new(values.append(sporks::spork_hash.eq(hash)))
    }

    pub fn update_values(&self) -> Box<dyn EntityUpdates<bool, ResultType = (bool, )>> {
        Box::new((
            sporks::identifier.eq(self.identifier.into() as i32),
            sporks::signature.eq(&self.signature),
            sporks::time_signed.eq(self.time_signed as i64),
            sporks::value.eq(self.value as i64)
        ))
    }

    pub fn to_entity_with_hash<T, U>(&self, hash: UInt256, chain_id: i32) -> U
        where
            T: Table,
            T::FromClause: diesel::query_builder::QueryFragment<diesel::sqlite::Sqlite>,
            U: Insertable<T>,
            U::Values: diesel::query_builder::QueryFragment<diesel::sqlite::Sqlite> + diesel::insertable::CanInsertInSingleQuery<diesel::sqlite::Sqlite> {
        let mut new_entity: NewSporkEntity = self.to_entity();
        new_entity.spork_hash = hash.clone();
        new_entity.chain_id = chain_id;
        new_entity
    }

    pub fn to_entity<T, U>(&self) -> U
        where
            T: Table,
            T::FromClause: diesel::query_builder::QueryFragment<diesel::sqlite::Sqlite>,
            U: Insertable<T>,
            U::Values: diesel::query_builder::QueryFragment<diesel::sqlite::Sqlite> + diesel::insertable::CanInsertInSingleQuery<diesel::sqlite::Sqlite> {
        NewSporkEntity {
            identifier: self.identifier.into() as i32,
            time_signed: self.time_signed as i64,
            value: self.value as i64,
            signature: self.signature.clone(),
            ..Default::default()
        }
    }

    pub fn from_entity(entity: &SporkEntity, chain: &Chain) -> Self {
        Self {
            identifier: Identifier::from(entity.identifier),
            is_valid: true,
            time_signed: entity.time_signed as u64,
            value: entity.value as u64,
            signature: entity.signature.clone(),
            chain
        }
    }
}*/
//
// impl Spork {
//     pub fn feature_is_activated(&mut self) -> bool {
//         self.chain.with(|chain| self.value <= chain.last_terminal_block_height() as u64)
//     }
// }
