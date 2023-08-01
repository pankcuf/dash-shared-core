use ed25519_dalek::{SigningKey, VerifyingKey};
use secp256k1::Scalar;
use crate::chain::derivation::{BIP32_HARD, IIndexPath};
use crate::consensus::encode::Encodable;
use crate::crypto::{ECPoint, UInt256, UInt512, byte_util::{AsBytes, clone_into_array}};
use crate::keys::{ECDSAKey, ED25519Key};

pub trait IChildKeyDerivationData<T, SK, PK> where SK: SignKey {
    fn private_key_data_input<PATH>(key: &SK, path: &PATH, position: usize) -> Vec<u8> where PATH: IIndexPath<Item = T>;
    fn public_key_data_input<PATH>(key: &PK, path: &PATH, position: usize) -> Vec<u8> where PATH: IIndexPath<Item = T>;
}

impl IChildKeyDerivationData<u32, UInt256, ECPoint> for ECDSAKey {
    fn private_key_data_input<PATH>(key: &UInt256, path: &PATH, position: usize) -> Vec<u8> where PATH: IIndexPath<Item=u32> {
        let index = path.index_at_position(position);
        let buf = &mut [0u8; 37];
        if index & BIP32_HARD != 0 {
            buf[1..33].copy_from_slice(&key.0);
        } else {
            buf[..33].copy_from_slice(&secp256k1_point_from_bytes(&key.0));
        }
        buf[33..37].copy_from_slice(index.to_be_bytes().as_slice());
        buf.to_vec()
    }

    fn public_key_data_input<PATH>(key: &ECPoint, path: &PATH, position: usize) -> Vec<u8> where PATH: IIndexPath<Item=u32> {
        let index = path.index_at_position(position);
        if index & BIP32_HARD != 0 {
            panic!("can't derive private child key from public parent key");
        }
        let writer = &mut [0u8; 37];
        writer[..33].copy_from_slice(&key.0);
        writer[33..].copy_from_slice(&index.to_be_bytes());
        writer.to_vec()
    }
}

impl IChildKeyDerivationData<UInt256, UInt256, ECPoint> for ECDSAKey {
    fn private_key_data_input<PATH>(key: &UInt256, path: &PATH, position: usize) -> Vec<u8> where PATH: IIndexPath<Item=UInt256> {
        let index = path.index_at_position(position);
        let is_hardened = path.hardened_at_position(position);
        let i_is_31_bits = index.is_31_bits();
        let mut writer = Vec::<u8>::new();
        if is_hardened {
            0u8.enc(&mut writer);
            writer.extend_from_slice(&key.0);
        } else {
            writer.extend_from_slice(&secp256k1_point_from_bytes(&key.0));
        };
        if i_is_31_bits {
            let mut small_i = index.u32_le();
            if is_hardened {
                small_i |= BIP32_HARD;
            }
            small_i.swap_bytes().enc(&mut writer);
        } else {
            index.enc(&mut writer);
        };
        writer
    }

    fn public_key_data_input<PATH>(key: &ECPoint, path: &PATH, position: usize) -> Vec<u8> where PATH: IIndexPath<Item=UInt256> {
        let index = path.index_at_position(position);
        let is_hardened = path.hardened_at_position(position);
        if is_hardened {
            panic!("can't derive private child key from public parent key");
        }
        let mut writer = key.as_bytes().to_vec();
        if index.is_31_bits() {
            writer.extend_from_slice(&index.u32_le().to_be_bytes());
        } else {
            writer.extend_from_slice(index.as_bytes());
        };
        writer
    }
}

impl IChildKeyDerivationData<u32, SigningKey, UInt256> for ED25519Key {
    fn private_key_data_input<PATH>(key: &SigningKey, path: &PATH, position: usize) -> Vec<u8> where PATH: IIndexPath<Item=u32> {
        let index = path.index_at_position(position);
        // let writer = &mut [0u8; 37];
        let writer = &mut [0u8; 36];
        if index & BIP32_HARD != 0 {
            writer[..32].copy_from_slice(&key.to_bytes());
        } else {
            writer[..32].copy_from_slice(key.verifying_key().as_bytes());
        }
        writer[32..36].copy_from_slice(&index.to_be_bytes());
        writer.to_vec()
    }

    fn public_key_data_input<PATH>(key: &UInt256, path: &PATH, position: usize) -> Vec<u8> where PATH: IIndexPath<Item=u32> {
        let index = path.index_at_position(position);
        if index & BIP32_HARD != 0 {
            panic!("can't derive private child key from public parent key");
        }
        let writer = &mut [0u8; 36];
        writer[..32].copy_from_slice(&key.0);
        writer[32..].copy_from_slice(&index.to_be_bytes());
        writer.to_vec()
    }
}

impl IChildKeyDerivationData<UInt256, SigningKey, UInt256> for ED25519Key {
    fn private_key_data_input<PATH>(key: &SigningKey, path: &PATH, position: usize) -> Vec<u8> where PATH: IIndexPath<Item=UInt256> {
        let index = path.index_at_position(position);
        let is_hardened = path.hardened_at_position(position);
        let i_is_31_bits = index.is_31_bits();
        let mut writer = Vec::<u8>::new();
        if is_hardened {
            0u8.enc(&mut writer);
            writer.extend_from_slice(&key.to_bytes());
        } else {
            panic!("For ED25519 only hardened derivation is supported");
        };
        if i_is_31_bits {
            let mut small_i = index.u32_le();
            if is_hardened {
                small_i |= BIP32_HARD;
            }
            small_i.swap_bytes().enc(&mut writer);
        } else {
            index.enc(&mut writer);
        };
        writer
    }

    fn public_key_data_input<PATH>(key: &UInt256, path: &PATH, position: usize) -> Vec<u8> where PATH: IIndexPath<Item=UInt256> {
        let index = path.index_at_position(position);
        let is_hardened = path.hardened_at_position(position);
        if is_hardened {
            panic!("can't derive private child key from public parent key");
        }
        let mut writer = key.as_bytes().to_vec();
        if index.is_31_bits() {
            writer.extend_from_slice(&index.u32_le().to_be_bytes());
        } else {
            writer.extend_from_slice(index.as_bytes());
        };
        writer
    }
}

pub trait SignKey: Sized {
    // type Inner;
}

impl SignKey for UInt256 { /*type Inner = UInt256;*/ }
impl SignKey for SigningKey { /*type Inner = SigningKey;*/ }

pub trait IChildKeyDerivation<T, SK, PK> where SK: SignKey + ?Sized {
    fn derive_child_private_key<PATH>(key: &mut SK, chaincode: &mut UInt256, path: &PATH, position: usize) where PATH: IIndexPath<Item = T>;
    fn derive_child_public_key<PATH>(key: &mut PK, chaincode: &mut UInt256, path: &PATH, position: usize) where PATH: IIndexPath<Item = T>;
}

pub fn secp256k1_point_from_bytes(data: &[u8]) -> [u8; 33] {
    let sec = secp256k1::SecretKey::from_slice(data).unwrap();
    let s = secp256k1::Secp256k1::new();
    let pub_key = secp256k1::PublicKey::from_secret_key(&s, &sec);
    pub_key.serialize()
}


impl<T> IChildKeyDerivation<T, UInt256, ECPoint> for ECDSAKey where Self: IChildKeyDerivationData<T, UInt256, ECPoint> {
    fn derive_child_private_key<PATH>(key: &mut UInt256, chaincode: &mut UInt256, path: &PATH, position: usize)
        where PATH: IIndexPath<Item=T> {
        let i = UInt512::hmac(chaincode.as_ref(), Self::private_key_data_input(key, path, position).as_ref());
        let mut sec_key = secp256k1::SecretKey::from_slice(&key.0).expect("invalid private key");
        let tweak = Scalar::from_be_bytes(clone_into_array(&i.0[..32])).expect("invalid tweak");
        sec_key = sec_key.add_tweak(&tweak).expect("failed to add tweak");
        key.0.copy_from_slice(&sec_key.secret_bytes());
        chaincode.0.copy_from_slice(&i.0[32..]);
    }

    fn derive_child_public_key<PATH>(key: &mut ECPoint, chaincode: &mut UInt256, path: &PATH, position: usize)
        where PATH: IIndexPath<Item=T> {
        let i = UInt512::hmac(chaincode.as_ref(), Self::public_key_data_input(key, path, position).as_ref());
        let s = secp256k1::Secp256k1::new();
        let mut pub_key = secp256k1::PublicKey::from_slice(&key.0).expect("invalid public key");
        let tweak = Scalar::from_be_bytes(clone_into_array(&i.0[..32])).expect("invalid tweak");
        pub_key = pub_key.add_exp_tweak(&s, &tweak).expect("failed to add exp tweak");
        chaincode.0.copy_from_slice(&i.0[32..]);
        key.0.copy_from_slice(pub_key.serialize().as_slice())
    }
}

impl<T> IChildKeyDerivation<T, SigningKey, UInt256> for ED25519Key where Self: IChildKeyDerivationData<T, SigningKey, UInt256> {
    fn derive_child_private_key<PATH>(key: &mut SigningKey, chaincode: &mut UInt256, path: &PATH, position: usize)
        where PATH: IIndexPath<Item=T> {
        let i = UInt512::hmac(chaincode.as_ref(), Self::private_key_data_input(key, path, position).as_ref());
        let scalar: [u8; 32] = i.0[..32].try_into().unwrap();
        key.clone_from(&SigningKey::from(&scalar));
        chaincode.0.copy_from_slice(&i.0[32..]);
    }

    fn derive_child_public_key<PATH>(key: &mut UInt256, chaincode: &mut UInt256, path: &PATH, position: usize)
        where PATH: IIndexPath<Item=T> {
        let i = UInt512::hmac(chaincode.as_ref(), Self::public_key_data_input(key, path, position).as_ref());
        let scalar: [u8; 32] = i.0[..32].try_into().unwrap();
        match VerifyingKey::from_bytes(&scalar) {
            Ok(pub_key) => {
                key.0.copy_from_slice(pub_key.as_bytes());
                chaincode.0.copy_from_slice(&i.0[32..]);
            },
            Err(err) => panic!("{}", err)
        }
    }
}

