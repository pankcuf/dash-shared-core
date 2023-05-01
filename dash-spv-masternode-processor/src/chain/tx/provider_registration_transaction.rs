use byte::{BytesExt, TryRead};
use crate::crypto::{UInt128, UInt160, UInt256, UInt384, VarBytes};
use crate::chain::tx::{InstantSendLock, ITransaction, Transaction, TransactionInput, TransactionOutput, TransactionType};
use crate::chain::tx::protocol::{MAX_ECDSA_SIGNATURE_SIZE, ReadContext, SIGHASH_ALL};
use crate::chain::chain::Chain;
use crate::chain::common::ChainType;
use crate::chain::constants::DASH_MESSAGE_MAGIC;
use crate::chain::ext::masternodes::TriggerUpdates;
use crate::consensus::Encodable;
use crate::consensus::encode::VarInt;
use crate::crypto::byte_util::{Reversable, Zeroable};
use crate::crypto::UTXO;
use crate::keys::ecdsa_key::ECDSAKey;
use crate::util::address::address;
use crate::util::Shared;

pub const MASTERNODE_COST: u64 = 100000000000;

#[derive(Clone, Debug, Default)]
pub struct ProviderRegistrationTransaction {
    pub base: Transaction,
    pub owner_key_hash: UInt160,
    pub voting_key_hash: UInt160,
    pub operator_key: UInt384,
    pub operator_reward: u16,
    pub ip_address: UInt128,
    pub port: u16,
    pub script_payout: Vec<u8>,
    pub collateral_outpoint: UTXO,
    pub provider_registration_transaction_version: u16,
    pub provider_type: u16,
    pub provider_mode: u16,

    pub inputs_hash: UInt256,
    pub payload_signature: Vec<u8>,
}

impl ITransaction for ProviderRegistrationTransaction {
    fn chain(&self) -> Shared<Chain> {
        self.base.chain()
    }

    fn chain_type(&self) -> ChainType {
        self.base.chain_type()
    }

    fn r#type(&self) -> TransactionType {
        TransactionType::ProviderRegistration
    }

    fn block_height(&self) -> u32 {
        self.base.block_height()
    }

    fn tx_hash(&self) -> UInt256 {
        self.base.tx_hash()
    }

    fn tx_lock_time(&self) -> u32 {
        self.base.tx_lock_time()
    }

    fn inputs(&self) -> Vec<TransactionInput> {
        self.base.inputs()
    }

    fn outputs(&self) -> Vec<TransactionOutput> {
        self.base.outputs()
    }

    fn input_addresses(&self) -> Vec<String> {
        self.base.input_addresses()
    }

    fn output_addresses(&self) -> Vec<String> {
        self.base.output_addresses()
    }

    fn size(&self) -> usize {
        if !self.tx_hash().is_zero() {
            return self.to_data().len();
        }
        self.base.size() +
            VarInt(self.payload_data().len() as u64).len() +
            self.base_payload_data().len() +
            MAX_ECDSA_SIGNATURE_SIZE
    }

    fn payload_data(&self) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        self.base_payload_data().enc(&mut writer);
        // as we know payload_signature size can't exceed u8::MAX
        (self.payload_signature.len() as u8).enc(&mut writer);
        self.payload_signature.enc(&mut writer);
        writer
    }

    fn payload_data_for_hash(&self) -> Vec<u8> {
        self.base_payload_data()
    }

    fn to_data_with_subscript_index(&self, subscript_index: Option<u64>) -> Vec<u8> {
        let mut data = self.base.to_data_with_subscript_index(subscript_index);
        self.payload_data().enc(&mut data);
        if subscript_index.is_some() {
            SIGHASH_ALL.enc(&mut data);
        }
        data
    }

    fn set_instant_send_received_with_instant_send_lock(&mut self, instant_send_lock: Option<Shared<InstantSendLock>>) {
        self.base.set_instant_send_received_with_instant_send_lock(instant_send_lock);
    }

    fn is_coinbase_classic_transaction(&self) -> bool {
        self.base.is_coinbase_classic_transaction()
    }

    fn has_set_inputs_and_outputs(&mut self) {
        self.update_inputs_hash();
        if self.collateral_outpoint.is_zero() {
            if let Some(index) = self.masternode_output_index() {
                self.collateral_outpoint = UTXO::with_index(index as u32);
                self.payload_signature = vec![];
            }
        }
    }

    // fn has_non_dust_output_in_wallet(&self, wallet: &Wallet) -> bool {
    //     self.base.has_non_dust_output_in_wallet(wallet)
    // }
    //
    // fn set_initial_persistent_attributes_in_context(&mut self, context: &ManagedContext) -> bool {
    //     todo!()
    // }
    //
    // fn to_entity_with_chain_entity(&self, chain_entity: ChainEntity) -> NewTransactionEntity {
    //     let mut base = self.base.to_entity_with_chain_entity(chain_entity);
    //     base.special_transaction_version = Some(self.provider_registration_transaction_version as i16);
    //     base.provider_type = self.provider_type as i16;
    //     base.provider_mode = self.provider_mode as i16;
    //     base.collateral_outpoint = Some(self.collateral_outpoint);
    //     base.ip_address = Some(self.ip_address);
    //     base.port = self.port as i16;
    //     base.owner_key_hash = Some(self.owner_key_hash);
    //     base.operator_key = Some(self.operator_key);
    //     base.voting_key_hash = Some(self.voting_key_hash);
    //     base.operator_reward = self.operator_reward as i16;
    //     base.script_payout = Some(self.script_payout.clone());
    //     base.payload_signature = Some(self.payload_signature.clone());
    //     // let owner_address = address_from_hash160_for_chain(&self.owner_key_hash, self.chain());
    //     // let operator_address = address_with_public_key_data(self.operator_key.as_bytes_vec(), self.chain());
    //     // let voting_address = address_from_hash160_for_chain(&self.voting_key_hash, self.chain());
    //     // let payout_address = address_with_script_pub_key(&self.script_payout, self.chain());
    //     // let owner_address_entities = AddressEntity::
    //
    //     // [super setAttributesFromTransaction:transaction];
    //     // DSProviderRegistrationTransaction *providerRegistrationTransaction = (DSProviderRegistrationTransaction *)transaction;
    //     // self.specialTransactionVersion = providerRegistrationTransaction.providerRegistrationTransactionVersion;
    //     // self.providerType = providerRegistrationTransaction.providerType;
    //     // self.providerMode = providerRegistrationTransaction.providerMode;
    //     // self.collateralOutpoint = dsutxo_data(providerRegistrationTransaction.collateralOutpoint);
    //     // self.ipAddress = uint128_data(providerRegistrationTransaction.ipAddress);
    //     // self.port = providerRegistrationTransaction.port;
    //     // self.ownerKeyHash = uint160_data(providerRegistrationTransaction.ownerKeyHash);
    //     // self.operatorKey = uint384_data(providerRegistrationTransaction.operatorKey);
    //     // self.votingKeyHash = uint160_data(providerRegistrationTransaction.votingKeyHash);
    //     // self.operatorReward = providerRegistrationTransaction.operatorReward;
    //     // self.scriptPayout = providerRegistrationTransaction.scriptPayout;
    //     // self.payloadSignature = providerRegistrationTransaction.payloadSignature;
    //     // NSString *ownerAddress = [self.ownerKeyHash addressFromHash160DataForChain:transaction.chain];
    //     // NSString *operatorAddress = [DSKey addressWithPublicKeyData:self.operatorKey forChain:transaction.chain];
    //     // NSString *votingAddress = [self.votingKeyHash addressFromHash160DataForChain:transaction.chain];
    //     // NSString *payoutAddress = [NSString addressWithScriptPubKey:self.scriptPayout onChain:transaction.chain];
    //
    //     // NSArray *ownerAddressEntities = [DSAddressEntity objectsInContext:self.managedObjectContext matching:@"address == %@ && derivationPath.chain == %@", ownerAddress, [transaction.chain chainEntityInContext:self.managedObjectContext]];
    //     // if ([ownerAddressEntities count]) {
    //     // NSAssert([ownerAddressEntities count] == 1, @"addresses should not be duplicates");
    //     // [self addAddressesObject:[ownerAddressEntities firstObject]];
    //     // }
    //     //
    //     // NSArray *operatorAddressEntities = [DSAddressEntity objectsInContext:self.managedObjectContext matching:@"address == %@ && derivationPath.chain == %@", operatorAddress, [transaction.chain chainEntityInContext:self.managedObjectContext]];
    //     // if ([operatorAddressEntities count]) {
    //     // NSAssert([operatorAddressEntities count] == 1, @"addresses should not be duplicates");
    //     // [self addAddressesObject:[operatorAddressEntities firstObject]];
    //     // }
    //     //
    //     // NSArray *votingAddressEntities = [DSAddressEntity objectsInContext:self.managedObjectContext matching:@"address == %@ && derivationPath.chain == %@", votingAddress, [transaction.chain chainEntityInContext:self.managedObjectContext]];
    //     // if ([votingAddressEntities count]) {
    //     // NSAssert([votingAddressEntities count] == 1, @"addresses should not be duplicates");
    //     // [self addAddressesObject:[votingAddressEntities firstObject]];
    //     // }
    //     //
    //     // NSArray *payoutAddressEntities = [DSAddressEntity objectsInContext:self.managedObjectContext matching:@"address == %@ && derivationPath.chain == %@", payoutAddress, [transaction.chain chainEntityInContext:self.managedObjectContext]];
    //     // if ([payoutAddressEntities count]) {
    //     // NSAssert([payoutAddressEntities count] == 1, @"addresses should not be duplicates");
    //     // [self addAddressesObject:[payoutAddressEntities firstObject]];
    //     // }
    //
    //     base
    // }
    //
    fn trigger_updates_for_local_references(&self) {
        self.chain().create_local_masternode_if_need(self);
        // self.chain().with(|chain| {
        //     if
        //     chain.wallet_having_provider_owner_authentication_hash(&self.owner_key_hash).is_some() ||
        //         chain.wallet_having_provider_voting_authentication_hash(&self.voting_key_hash).is_some() ||
        //         chain.wallet_having_provider_operator_authentication_key(&self.operator_key).is_some() {
        //         chain.masternode_manager.local_masternode_from_provider_registration_transaction(self, true);
        //     }
        // });
    }
    // fn load_blockchain_identities_from_derivation_paths(&mut self, derivation_paths: Vec<&dyn IDerivationPath>) {
    //     self.base.load_blockchain_identities_from_derivation_paths(derivation_paths)
    // }
}

impl ProviderRegistrationTransaction {

    pub fn payload_hash(&self) -> UInt256 {
        UInt256::sha256d(self.payload_data_for_hash())
    }

    pub fn payload_collateral_string(&self) -> String {
        format!("{}|{}|{}|{}|{}",
                self.payout_address().unwrap_or(String::new()),
                self.operator_reward,
                self.owner_address(),
                self.voting_address(),
                self.payload_hash().clone().reversed())
    }


    pub fn payload_collateral_digest(&self) -> UInt256 {
        let mut writer = Vec::<u8>::new();
        DASH_MESSAGE_MAGIC.to_string().enc(&mut writer);
        self.payload_collateral_string().enc(&mut writer);
        // writer.append_string(DASH_MESSAGE_MAGIC.to_string());
        // writer.append_string(self.payload_collateral_string());
        UInt256::sha256d(writer)
    }

    pub fn check_payload_signature_with_key(&self, key: &mut ECDSAKey) -> bool {
        key.hash160() == self.owner_key_hash
    }

    pub fn check_payload_signature(&self) -> bool {
        if let Some(mut provider_owner_public_key) = ECDSAKey::key_recovered_from_compact_sig(&self.payload_signature, self.payload_hash()) {
            self.check_payload_signature_with_key(&mut provider_owner_public_key)
        } else {
            false
        }
    }

    pub fn base_payload_data(&self) -> Vec<u8> {
        let mut writer = Vec::<u8>::new();
        self.provider_registration_transaction_version.enc(&mut writer);    // 16
        self.provider_type.enc(&mut writer);                                // 32
        self.provider_mode.enc(&mut writer);                                // 48
        self.collateral_outpoint.enc(&mut writer);                          // 84
        self.ip_address.enc(&mut writer);                                   // 212
        self.port.swap_bytes().enc(&mut writer);                            // 228
        self.owner_key_hash.enc(&mut writer);                               // 388
        self.operator_key.enc(&mut writer);                                 // 772
        self.voting_key_hash.enc(&mut writer);                              // 788 ???? offset?
        self.operator_reward.enc(&mut writer);                              // 804 ???? offset?
        VarInt(self.script_payout.len() as u64).enc(&mut writer);
        self.script_payout.enc(&mut writer);
        self.inputs_hash.enc(&mut writer);
        writer
    }

    pub fn owner_address(&self) -> String {
        address::from_hash160_for_script_map(&self.owner_key_hash, &self.chain_type().script_map())
    }

    pub fn operator_address(&self) -> String {
        address::with_public_key_data(&self.operator_key.0.to_vec(), &self.chain_type().script_map())
    }

    pub fn operator_key_string(&self) -> String {
        format!("{}", self.operator_key)
    }

    pub fn voting_address(&self) -> String {
        address::from_hash160_for_script_map(&self.voting_key_hash, &self.chain_type().script_map())
    }

    pub fn holding_address(&self) -> Option<String> {
        self.masternode_output_index()
            .filter(|_| self.collateral_outpoint.hash.is_zero())
            .and_then(|index| self.outputs().get(index).and_then(|out| out.address.clone()))
    }

    pub fn payout_address(&self) -> Option<String> {
        address::with_script_pub_key(&self.script_payout, &self.chain_type().script_map())
    }

    pub fn location(&self) -> String {
        // todo: check if to_string here is correct + v4 vs v6
        format!("{}:{}", self.ip_address.to_ip_addr(), self.port)
    }

    pub fn core_registration_command(&self) -> String {
        format!("protx register_prepare {} {} {} {} {} {} {} {}",
                self.collateral_outpoint.hash.clone().reversed(),
                self.collateral_outpoint.n,
                self.location(),
                self.owner_address(),
                self.operator_key_string(),
                self.voting_address(),
                self.operator_reward,
                self.payout_address().unwrap_or("".to_string()))
    }

    pub fn update_inputs_hash(&mut self) {
        let mut writer = Vec::<u8>::new();
        self.inputs().iter().for_each(|input| {
            input.input_hash.enc(&mut writer);
            input.index.enc(&mut writer);
        });
        self.inputs_hash = UInt256::sha256d(writer);
    }

    // pub fn local_masternode(&self) -> Option<&LocalMasternode> {
    //     self.chain().local_masternode_from_provider_registration_transaction(self, true)
    // }
    //
    // // wallet and index
    // pub fn masternode_holding_wallet(&self) -> Option<(&Wallet, u32)> {
    //     self.chain().wallet_containing_masternode_holding_address_for_provider_registration_transaction(self)
    // }

    pub fn masternode_output_index(&self) -> Option<usize> {
        // What if a masternode's cost is equal to smth another?
        self.outputs().iter().position(|o| o.amount == MASTERNODE_COST)
    }
}

// todo: migrate to custom trait which allows passing of custom context, like Chain etc.
impl<'a> TryRead<'a, ReadContext> for ProviderRegistrationTransaction {
    fn try_read(bytes: &'a [u8], context: ReadContext) -> byte::Result<(Self, usize)> {
        let (mut base, mut offset) = Transaction::try_read(bytes, context)?;
        base.tx_type = TransactionType::ProviderRegistration;
        let _extra_payload_size = bytes.read_with::<VarInt>(&mut offset, byte::LE)?;
        let provider_registration_transaction_version = bytes.read_with::<u16>(&mut offset, byte::LE)?;
        let provider_type = bytes.read_with::<u16>(&mut offset, byte::LE)?;
        let provider_mode = bytes.read_with::<u16>(&mut offset, byte::LE)?;
        let collateral_outpoint = bytes.read_with::<UTXO>(&mut offset, byte::LE)?;
        let ip_address = bytes.read_with::<UInt128>(&mut offset, byte::LE)?;
        let port = bytes.read_with::<u16>(&mut offset, byte::BE)?;
        let owner_key_hash = bytes.read_with::<UInt160>(&mut offset, byte::LE)?;
        let operator_key = bytes.read_with::<UInt384>(&mut offset, byte::LE)?;
        let voting_key_hash = bytes.read_with::<UInt160>(&mut offset, byte::LE)?;
        let operator_reward = bytes.read_with::<u16>(&mut offset, byte::LE)?;
        let script_payout = bytes.read_with::<VarBytes>(&mut offset, byte::LE)?.1.to_vec();
        let inputs_hash = bytes.read_with::<UInt256>(&mut offset, byte::LE)?;
        let payload_signature = bytes.read_with::<VarBytes>(&mut offset, byte::LE)?.1.to_vec();
        base.payload_offset = offset;
        let mut tx = Self {
            base,
            owner_key_hash,
            voting_key_hash,
            operator_key,
            operator_reward,
            ip_address,
            port,
            script_payout,
            collateral_outpoint,
            provider_registration_transaction_version,
            provider_type,
            provider_mode,
            inputs_hash,
            payload_signature
        };
        // todo verify inputs hash
        assert_eq!(tx.payload_data().len(), offset, "Payload length doesn't match ");
        tx.base.tx_hash = UInt256::sha256d(tx.to_data());
        Ok((tx, offset))
    }
}

