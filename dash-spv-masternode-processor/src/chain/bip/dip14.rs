use secp256k1::Scalar;
use crate::consensus::Encodable;
use crate::crypto::byte_util::{AsBytes, clone_into_array};
use crate::crypto::{ECPoint, UInt256, UInt512};
use crate::chain::derivation::BIP32_HARD;

// multiplies secp256k1 generator by 256bit big endian int i and
// adds the result to ec-point self returns it on success
// pub fn secp256k1_point_add(p: &ECPoint, i: &UInt256) -> ECPoint {
//     // secp256k1::Signing + secp256k1::Verification
//     let s = secp256k1::Secp256k1::new();
//     let pub_key = secp256k1::PublicKey::from_slice(p.as_bytes()).unwrap();
//     let tweak = Scalar::from_be_bytes(i.0).unwrap();
//     let k = pub_key.add_exp_tweak(&s, &tweak).unwrap();
//     ECPoint(k.serialize())
// }
//
// multiplies secp256k1 generator by 256bit big endian int i and stores the result in p
// returns true on success
// pub fn secp256k1_point_gen(i: &UInt256) -> ECPoint {
//     let pub_key = secp256k1::PublicKey::from_slice(&i.0).unwrap();
//     ECPoint(pub_key.serialize())
// }
//
// multiplies 256bit big endian ints a and b (mod secp256k1 order) and stores the result in a
// returns true on success
// pub fn secp256k1_mod_mul(a: &mut UInt256, b: &UInt256) -> secp256k1::SecretKey {
//     let sec_key = secp256k1::SecretKey::from_slice(a.as_bytes()).unwrap();
//     let tweak = Scalar::from_le_bytes(b.0).unwrap();
//     sec_key.mul_tweak(&tweak).unwrap()
// }
//
// adds 256bit big endian ints a and b (mod secp256k1 order) and stores the result in a
// returns true on success
// pub fn secp256k1_mod_add(a: &mut UInt256, b: &UInt256) -> secp256k1::SecretKey {
//     // _ctx = secp256k1_context_create(SECP256K1_CONTEXT_SIGN | SECP256K1_CONTEXT_VERIFY);
//     let sec_key = secp256k1::SecretKey::from_slice(a.as_bytes()).unwrap();
//     let tweak = Scalar::from_le_bytes(b.0).unwrap();
//     sec_key.add_tweak(&tweak).unwrap()
// }

fn secp256k1_point_from_bytes(data: &[u8]) -> [u8; 33] {
    let sec = secp256k1::SecretKey::from_slice(data).unwrap();
    let s = secp256k1::Secp256k1::new();
    let pub_key = secp256k1::PublicKey::from_secret_key(&s, &sec);
    pub_key.serialize()
}

// BIP32 is a scheme for deriving chains of addresses from a seed value
// https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki

// Private parent key -> private child key
//
// CKDpriv((kpar, cpar), i) -> (ki, ci) computes a child extended private key from the parent extended private key:
//
// - Check whether i >= 2^31 (whether the child is a hardened key).
//     - If so (hardened child): let I = HMAC-SHA512(Key = cpar, Data = 0x00 || ser256(kpar) || ser32(i)).
//       (Note: The 0x00 pads the private key to make it 33 bytes long.)
//     - If not (normal child): let I = HMAC-SHA512(Key = cpar, Data = serP(point(kpar)) || ser32(i)).
// - Split I into two 32-byte sequences, IL and IR.
// - The returned child key ki is parse256(IL) + kpar (mod n).
// - The returned chain code ci is IR.
// - In case parse256(IL) >= n or ki = 0, the resulting key is invalid, and one should proceed with the next value for i
//   (Note: this has probability lower than 1 in 2^127.)
//
// fn ckd_priv(k: &mut UInt512, key: &[u8]) {
//     // I = HMAC-SHA512(c, k|P(k) || i)
//     // k = IL + k (mod n)
//     // c = IR
//     let i = UInt512::hmac(&k.0[32..], key);
//     let mut sec_key = secp256k1::SecretKey::from_slice(&k.0[..32]).unwrap();
//     let tweak = Scalar::from_be_bytes(clone_into_array(&i.0[..32])).unwrap();
//     sec_key = sec_key.add_tweak(&tweak).unwrap();
//     k.0[..32].copy_from_slice(&sec_key.secret_bytes());
//     k.0[32..].copy_from_slice(&i.0[32..]);
// }
// pub fn derive_child_private_key(k: &mut UInt512, i: u32) {
//     let buf = &mut [0u8; 37];
//     if i & BIP32_HARD != 0 {
//         buf[1..33].copy_from_slice(&k.0[..32]);
//     } else {
//         buf[..33].copy_from_slice(&secp256k1_point_from_bytes(&k.0[..32]));
//     }
//     buf[33..37].copy_from_slice(i.to_be_bytes().as_slice());
//     ckd_priv(k, buf);
// }
// pub fn derive_child_private_key_256(k: &mut UInt512, i: UInt256, hardened: bool) {
//     let i_is_31_bits = i.is_31_bits();
//     let mut writer = Vec::<u8>::new();
//     if hardened {
//         0u8.enc(&mut writer);
//         writer.extend_from_slice(&k.0[..32]);
//     } else {
//         writer.extend_from_slice(&secp256k1_point_from_bytes(&k.0[..32]));
//     };
//     if i_is_31_bits {
//         let mut small_i = i.u32_le();
//         if hardened {
//             small_i |= BIP32_HARD;
//         }
//         small_i.swap_bytes().enc(&mut writer);
//     } else {
//         i.enc(&mut writer);
//     };
//     ckd_priv(k, &writer)
// }

fn ckd_priv(k: &mut UInt256, c: &mut UInt256, key: &[u8]) {
    // I = HMAC-SHA512(c, k|P(k) || i)
    // k = IL + k (mod n)
    // c = IR
    let i = UInt512::hmac(&c.0, key);
    let mut sec_key = secp256k1::SecretKey::from_slice(&k.0).unwrap();
    let tweak = Scalar::from_be_bytes(clone_into_array(&i.0[..32])).unwrap();
    sec_key = sec_key.add_tweak(&tweak).unwrap();
    k.0.copy_from_slice(&sec_key.secret_bytes());
    c.0.copy_from_slice(&i.0[32..]);
}

pub fn derive_child_private_key(k: &mut UInt256, c: &mut UInt256, i: u32) {
    let buf = &mut [0u8; 37];
    if i & BIP32_HARD != 0 {
        buf[1..33].copy_from_slice(&k.0);
    } else {
        buf[..33].copy_from_slice(&secp256k1_point_from_bytes(&k.0));
    }
    buf[33..37].copy_from_slice(i.to_be_bytes().as_slice());
    ckd_priv(k, c, buf);
}

pub fn derive_child_private_key_256(k: &mut UInt256, c: &mut UInt256, i: UInt256, hardened: bool) {
    let i_is_31_bits = i.is_31_bits();
    let mut writer = Vec::<u8>::new();
    if hardened {
        0u8.enc(&mut writer);
        writer.extend_from_slice(&k.0);
    } else {
        writer.extend_from_slice(&secp256k1_point_from_bytes(&k.0));
    };
    if i_is_31_bits {
        let mut small_i = i.u32_le();
        if hardened {
            small_i |= BIP32_HARD;
        }
        small_i.swap_bytes().enc(&mut writer);
    } else {
        i.enc(&mut writer);
    };
    ckd_priv(k, c, &writer)
}

// Public parent key -> public child key
//
// CKDpub((Kpar, cpar), i) -> (Ki, ci) computes a child extended public key from the parent extended public key.
// It is only defined for non-hardened child keys.
//
// - Check whether i >= 2^31 (whether the child is a hardened key).
//     - If so (hardened child): return failure
//     - If not (normal child): let I = HMAC-SHA512(Key = cpar, Data = serP(Kpar) || ser32(i)).
// - Split I into two 32-byte sequences, IL and IR.
// - The returned child key Ki is point(parse256(IL)) + Kpar.
// - The returned chain code ci is IR.
// - In case parse256(IL) >= n or Ki is the point at infinity, the resulting key is invalid, and one should proceed with
//   the next value for i.
//
fn ckd_pub(k: &mut ECPoint, c: &mut UInt256, key: &[u8]) {
    let key = UInt512::hmac(&c.0, key);
    c.0.copy_from_slice(&key.0[32..]);
    let s = secp256k1::Secp256k1::new();
    let mut pub_key = secp256k1::PublicKey::from_slice(&k.0).expect("invalid public key");
    let tweak = Scalar::from_be_bytes(clone_into_array(&key.0[..32])).expect("invalid tweak");
    pub_key = pub_key.add_exp_tweak(&s, &tweak).expect("failed to add tweak");
    k.0.copy_from_slice(pub_key.serialize().as_ref())
}

// I = HMAC-SHA512(c, P(K) || i)
// c = IR
// K = P(IL) + K
pub fn derive_child_public_key(k: &mut ECPoint, c: &mut UInt256, i: u32) {
    if i & BIP32_HARD != 0 {
        // can't derive private child key from public parent key
        return;
    }
    let writer = &mut [0u8; 37];
    writer[..33].copy_from_slice(&k.0);
    writer[33..].copy_from_slice(&i.to_be_bytes());
    ckd_pub(k, c, writer);
}

pub fn derive_child_public_key_256(k: &mut ECPoint, c: &mut UInt256, i: UInt256, hardened: bool) {
    if hardened {
        // can't derive private child key from public parent key
        return;
    }
    let mut writer = k.as_bytes().to_vec();
    if i.is_31_bits() {
        writer.extend_from_slice(&i.u32_le().to_be_bytes());
    } else {
        writer.extend_from_slice(i.as_bytes());
    };
    ckd_pub(k, c, &writer);
}


// Let n denote the order of the curve.
//
// The function CKDpriv((kpar, cpar), i) → (ki, ci) computes a child extended private key from the parent extended private key:
//
// Check whether i ≥ 2^31 (whether the child is a hardened key).
// If so (hardened child): let I = HMAC-SHA512(Key = cpar, Data = 0x00 || ser256(kpar) || ser32(i)). (Note: The 0x00 pads the private key to make it 33 bytes long.)
// If not (normal child):
// If curve is ed25519: return failure.
// let I = HMAC-SHA512(Key = cpar, Data = serP(point(kpar)) || ser32(i)).
// Split I into two 32-byte sequences, IL and IR.
// The returned chain code ci is IR.
// If curve is ed25519: The returned child key ki is parse256(IL).
// If parse256(IL) ≥ n or parse256(IL) + kpar (mod n) = 0 (resulting key is invalid):
// let I = HMAC-SHA512(Key = cpar, Data = 0x01 || IR || ser32(i) and restart at step 2.
// Otherwise: The returned child key ki is parse256(IL) + kpar (mod n).
// The HMAC-SHA512 function is specified in RFC 4231.
fn ckd_priv_ed25519(signing_key: &mut ed25519_dalek::SigningKey, chain: &mut UInt256, key: &[u8]) {
    let i = UInt512::hmac(&chain.0, key);
    let scalar: [u8; 32] = i.0[..32].try_into().unwrap();
    signing_key.clone_from(&ed25519_dalek::SigningKey::from(&scalar));
    chain.0.copy_from_slice(&i.0[32..]);
}
pub fn derive_child_private_key_ed25519(signing_key: &mut ed25519_dalek::SigningKey, chaincode: &mut UInt256, i: u32) {
    let buf = &mut [0u8; 37];
    if i & BIP32_HARD != 0 {
        buf[1..33].copy_from_slice(&signing_key.to_bytes());
    } else {
        buf[1..33].copy_from_slice(signing_key.verifying_key().as_bytes());
    }
    buf[33..37].copy_from_slice(i.to_be_bytes().as_slice());
    ckd_priv_ed25519(signing_key, chaincode, buf);
}

pub fn derive_child_private_key_256_ed25519(signing_key: &mut ed25519_dalek::SigningKey, chaincode: &mut UInt256, i: UInt256, hardened: bool) {
    let i_is_31_bits = i.is_31_bits();
    let mut writer = Vec::<u8>::new();
    if hardened {
        0u8.enc(&mut writer);
        writer.extend_from_slice(&signing_key.to_bytes());
    } else {
        panic!("For ED25519 only hardened derivation is supported");
    };
    if i_is_31_bits {
        let mut small_i = i.u32_le();
        if hardened {
            small_i |= BIP32_HARD;
        }
        small_i.swap_bytes().enc(&mut writer);
    } else {
        i.enc(&mut writer);
    };
    ckd_priv_ed25519(signing_key, chaincode, &writer)
}

fn ckd_pub_ed25519(k: &mut ECPoint, c: &mut UInt256, key: &[u8]) {
    let key = UInt512::hmac(&c.0, key);
    c.0.copy_from_slice(&key.0[32..]);
    let s = secp256k1::Secp256k1::new();
    let mut pub_key = secp256k1::PublicKey::from_slice(&k.0).expect("invalid public key");
    let tweak = Scalar::from_be_bytes(clone_into_array(&key.0[..32])).expect("invalid tweak");
    pub_key = pub_key.add_exp_tweak(&s, &tweak).expect("failed to add tweak");
    k.0.copy_from_slice(pub_key.serialize().as_ref())
}

pub fn derive_child_public_key_ed25519(k: &mut ECPoint, c: &mut UInt256, i: u32) {
    if i & BIP32_HARD != 0 {
        // can't derive private child key from public parent key
        return;
    }
    let writer = &mut [0u8; 37];
    writer[..33].copy_from_slice(&k.0);
    writer[33..].copy_from_slice(&i.to_be_bytes());
    ckd_pub_ed25519(k, c, writer);
}

pub fn derive_child_public_key_256_ed25519(k: &mut ECPoint, c: &mut UInt256, i: UInt256, hardened: bool) {
    if hardened {
        // can't derive private child key from public parent key
        return;
    }
    let mut writer = k.as_bytes().to_vec();
    if i.is_31_bits() {
        writer.extend_from_slice(&i.u32_le().to_be_bytes());
    } else {
        writer.extend_from_slice(i.as_bytes());
    };
    ckd_pub_ed25519(k, c, &writer);
}

