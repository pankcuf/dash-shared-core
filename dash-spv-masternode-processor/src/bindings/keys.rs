use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_ulong, c_void};
use std::ptr::null_mut;
use std::slice;
use byte::BytesExt;
use dash_spv_ffi::{boxed, boxed_vec, unbox_any};
use secp256k1::Scalar;
use crate::chain::bip::bip32;
use crate::chain::bip::bip38::BIP38;
use crate::chain::common::{ChainType, IHaveChainSettings};
use crate::chain::derivation::{BIP32_HARD, IndexPath};
use crate::consensus::Encodable;
use crate::crypto::byte_util::{AsBytes, clone_into_array, ConstDecodable, Reversable, Zeroable};
use crate::crypto::{UInt160, UInt256, UInt384, UInt512, UInt768};
use crate::ffi::{ByteArray, IndexPathData};
use crate::ffi::unboxer::{unbox_opaque_key, unbox_opaque_keys, unbox_opaque_serialized_keys};
use crate::keys::{BLSKey, ECDSAKey, ED25519Key, IKey, KeyKind};
use crate::keys::crypto_data::{CryptoData, DHKey};
use crate::keys::dip14::secp256k1_point_from_bytes;
use crate::processing::keys_cache::KeysCache;
use crate::types::opaque_key::{AsCStringPtr, AsOpaqueKey, OpaqueKey, KeyWithUniqueId, OpaqueKeys, OpaqueSerializedKeys};
use crate::util::address::address;
use crate::util::sec_vec::SecVec;

/// Destroys
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn processor_destroy_opaque_key(data: *mut OpaqueKey) {
    unbox_opaque_key(data);
}

#[no_mangle]
pub unsafe extern "C" fn processor_destroy_opaque_keys(data: *mut OpaqueKeys) {
    unbox_opaque_keys(data);
}

#[no_mangle]
pub unsafe extern "C" fn processor_destroy_serialized_opaque_keys(data: *mut OpaqueSerializedKeys) {
    unbox_opaque_serialized_keys(data);
}

/// Initialize opaque cache to store keys information between FFI calls
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn keys_create_cache() -> *mut KeysCache {
    let cache = KeysCache::default();
    println!("keys_create_cache: {:?}", cache);
    boxed(cache)
}

/// Clear opaque key cache
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn keys_clear_cache(cache: *mut KeysCache) {
    println!("keys_clear_cache: {:p}", cache);
    (*cache).clear();
}

/// Destroy opaque key cache
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn keys_destroy_cache(cache: *mut KeysCache) {
    println!("keys_destroy_cache: {:?}", cache);
    let cache = unbox_any(cache);
}


/// Destroys anonymous internal holder for KeyWithUniqueId
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn processor_destroy_key_wrapper(key: *mut KeyWithUniqueId) {
    let k = unbox_any(key);
    unbox_any(k.ptr);
}

/// Destroys anonymous internal holder for ECDSAKey
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn processor_destroy_ecdsa_key(key: *mut ECDSAKey) {
    unbox_any(key);
}

/// Destroys anonymous internal holder for BLSKey
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn processor_destroy_bls_key(key: *mut BLSKey) {
    unbox_any(key);
}

/// Destroys anonymous internal holder for ED25519Key
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn processor_destroy_ed25519_key(key: *mut ED25519Key) {
    unbox_any(key);
}

/// Removes ECDSA key from cache
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn cache_remove_ecdsa_key(unique_id: u64, cache: *mut KeysCache) {
    let cache = &mut *cache;
    cache.ecdsa.remove(&unique_id);
}

/// Removes BLS key from cache
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn cache_key_remove_bls_key(unique_id: u64, cache: *mut KeysCache) {
    let cache = &mut *cache;
    cache.bls.remove(&unique_id);
}

/// Removes ED25519 key from cache
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn cache_key_remove_ed25519_key(unique_id: u64, cache: *mut KeysCache) {
    let cache = &mut *cache;
    cache.ed25519.remove(&unique_id);
}

/// Replacement for [DSKey keyWithExtendedPublicKeyData]
/// Returns 'unique_id' (u64-equivalent for [DSDerivationPath createIdentifierForDerivationPath])
/// Then key can be removed by this 'unique_id'
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_create_ecdsa_from_extened_public_key_data(ptr: *const u8, len: usize, cache: *mut KeysCache) -> *mut KeyWithUniqueId {
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };
    ECDSAKey::key_with_extended_public_key_data(bytes)
        .map_or(null_mut(), |key| {
            let cache = &mut *cache;
            let unique_id = UInt256::sha256(bytes).u64_le();
            cache.ecdsa.insert(unique_id, key.clone());
            boxed(KeyWithUniqueId { key_type: KeyKind::ECDSA, unique_id, ptr: boxed(key) as *mut c_void })
        })
}

/// Replacement for [DSKey keyWithExtendedPublicKeyData]
/// Returns 'unique_id' (u64-equivalent for [DSDerivationPath createIdentifierForDerivationPath])
/// Then key can be removed by this 'unique_id'
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_create_bls_from_extened_public_key_data(ptr: *const u8, len: usize, use_legacy: bool, cache: *mut KeysCache) -> *mut KeyWithUniqueId {
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };
    BLSKey::key_with_extended_public_key_data(bytes, use_legacy)
        .map_or(null_mut(), |key| {
            let bytes = unsafe { slice::from_raw_parts(ptr, len) };
            let cache = &mut *cache;
            let unique_id = UInt256::sha256(bytes).u64_le();
            cache.bls.insert(unique_id, key.clone());
            boxed(KeyWithUniqueId { key_type: if use_legacy { KeyKind::BLS } else { KeyKind::BLSBasic }, unique_id, ptr: boxed(key) as *mut c_void })
        })
}

/// Replacement for [DSKey keyWithExtendedPublicKeyData]
/// Returns 'unique_id' (u64-equivalent for [DSDerivationPath createIdentifierForDerivationPath])
/// Then key can be removed by this 'unique_id'
/// # Safety
#[no_mangle]
pub extern "C" fn key_create_ed25519_from_extened_public_key_data(ptr: *const u8, len: usize, cache: *mut KeysCache) -> *mut KeyWithUniqueId {
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };
    let cache = unsafe { &mut *cache };
    ED25519Key::key_with_extended_public_key_data(bytes)
        .map_or(null_mut(), |key| {
            let unique_id = UInt256::sha256(bytes).u64_le();
            cache.ed25519.insert(unique_id, key.clone());
            boxed(KeyWithUniqueId { key_type: KeyKind::ED25519, unique_id, ptr: boxed(key) as *mut c_void })
        })
}

#[no_mangle]
pub extern "C" fn key_derive_key_from_extened_private_key_data_for_index_path(secret: *const u8, secret_len: usize, key_type: KeyKind, indexes: *const c_ulong, length: usize) -> *mut OpaqueKey {
    let bytes = unsafe { slice::from_raw_parts(secret, secret_len) };
    let path = IndexPath::from_ffi(indexes, length);
    match key_type {
        KeyKind::ECDSA => ECDSAKey::key_with_extended_private_key_data(bytes)
            .and_then(|key| key.private_derive_to_path(&path))
            .to_opaque_ptr(),
        KeyKind::ED25519 => ED25519Key::key_with_extended_private_key_data(bytes)
            .and_then(|key| key.private_derive_to_path(&path))
            .to_opaque_ptr(),
        _ => BLSKey::key_with_extended_private_key_data(bytes, key_type == KeyKind::BLS)
            .ok()
            .and_then(|key| key.private_derive_to_path(&path))
            .to_opaque_ptr(),
    }
}

#[no_mangle]
pub extern "C" fn key_derive_ecdsa_from_extened_private_key_data_for_index_path(secret: *const u8, secret_len: usize, indexes: *const c_ulong, length: usize) -> *mut ECDSAKey {
    let bytes = unsafe { slice::from_raw_parts(secret, secret_len) };
    let path = IndexPath::from_ffi(indexes, length);
    ECDSAKey::key_with_extended_private_key_data(bytes)
        .and_then(|key| key.private_derive_to_path(&path))
        .map_or(null_mut(), boxed)
}

/// # Safety
/// digest is UInt256
#[no_mangle]
pub unsafe extern "C" fn key_sign_message_digest(key: *mut OpaqueKey, digest: *const u8) -> ByteArray {
    // let key = unsafe { &mut *key };
    let message_digest = UInt256::from_const(digest).unwrap();
    match *key {
        OpaqueKey::ECDSA(ptr) => (&*ptr).compact_sign(message_digest).into(),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => (&*ptr).sign_digest(message_digest).into(),
        OpaqueKey::ED25519(ptr) => (&*ptr).sign(&message_digest.0).into()
    }
}

/// # Safety
/// digest is UInt256
#[no_mangle]
pub unsafe extern "C" fn key_verify_message_digest(key: *mut OpaqueKey, md: *const u8, sig: *const u8, sig_len: usize) -> bool {
    let digest = slice::from_raw_parts(md, UInt256::SIZE);
    let signature = slice::from_raw_parts(sig, sig_len);
    match *key {
        OpaqueKey::ECDSA(ptr) => (&mut *ptr).verify(digest, signature),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => (&mut *ptr).verify(digest, signature),
        OpaqueKey::ED25519(ptr) => (&mut *ptr).verify(digest, signature)
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_has_private_key(key: *mut OpaqueKey) -> bool {
    match *key {
        OpaqueKey::ECDSA(ptr) => (&*ptr).has_private_key(),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => (&*ptr).has_private_key(),
        OpaqueKey::ED25519(ptr) => (&*ptr).has_private_key(),
    }
}

// serializedPrivateKeyForChain
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_serialized_private_key_for_chain(key: *mut OpaqueKey, chain_type: ChainType) -> *mut c_char {
    let script = chain_type.script_map();
    match *key {
        OpaqueKey::ECDSA(ptr) => (&*ptr).serialized_private_key_for_script(&script),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => (&*ptr).serialized_private_key_for_script(&script),
        OpaqueKey::ED25519(ptr) => (&*ptr).serialized_private_key_for_script(&script),
    }.to_c_string_ptr()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_with_private_key(secret: *const c_char, key_type: KeyKind, chain_type: ChainType) -> *mut OpaqueKey {
    let c_str = unsafe { CStr::from_ptr(secret) };
    let private_key_string = c_str.to_str().unwrap();
    match key_type {
        KeyKind::ECDSA => ECDSAKey::key_with_private_key(private_key_string, chain_type).to_opaque_ptr(),
        KeyKind::BLS => BLSKey::key_with_private_key(private_key_string, true).to_opaque_ptr(),
        KeyKind::BLSBasic => BLSKey::key_with_private_key(private_key_string, false).to_opaque_ptr(),
        KeyKind::ED25519 => ED25519Key::key_with_private_key(private_key_string).to_opaque_ptr(),
    }
}

/// # Safety
#[no_mangle]
pub extern "C" fn key_with_seed_data(data: *const u8, len: usize, key_type: KeyKind) -> *mut OpaqueKey {
    let seed = unsafe { slice::from_raw_parts(data, len) };
    key_type.key_with_seed_data(seed)
        .to_opaque_ptr()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn forget_private_key(key: *mut OpaqueKey) {
    match *key {
        OpaqueKey::ECDSA(ptr) => (&mut *ptr).forget_private_key(),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => (&mut *ptr).forget_private_key(),
        OpaqueKey::ED25519(ptr) => (&mut *ptr).forget_private_key()
    }
}


// _extendedPublicKey = [parentDerivationPath.extendedPublicKey publicDeriveTo256BitDerivationPath:self derivationPathOffset:parentDerivationPath.length];
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_public_derive_to_256bit(key: *mut OpaqueKey, derivation_indexes: *const u8, derivation_hardened: *const bool, derivation_len: usize, offset: usize) -> *mut OpaqueKey {
    let path = IndexPath::from((derivation_indexes, derivation_hardened, derivation_len));
    match *key {
        OpaqueKey::ECDSA(ptr) => (&mut *ptr).public_derive_to_256bit_derivation_path_with_offset(&path, offset).to_opaque_ptr(),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => (&mut *ptr).public_derive_to_256bit_derivation_path_with_offset(&path, offset).to_opaque_ptr(),
        OpaqueKey::ED25519(ptr) => (&mut *ptr).public_derive_to_256bit_derivation_path_with_offset(&path, offset).to_opaque_ptr()
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_private_key_data(key: *mut OpaqueKey) -> ByteArray {
    match *key {
        OpaqueKey::ECDSA(ptr) => (&*ptr).private_key_data(),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => (&*ptr).private_key_data(),
        OpaqueKey::ED25519(ptr) => (&*ptr).private_key_data()
    }.into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_public_key_data(key: *mut OpaqueKey) -> ByteArray {
    match *key {
        OpaqueKey::ECDSA(ptr) => (&*ptr).public_key_data(),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => (&*ptr).public_key_data(),
        OpaqueKey::ED25519(ptr) => (&*ptr).public_key_data()
    }.into()
}
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_extended_public_key_data(key: *mut OpaqueKey) -> ByteArray {
    match *key {
        OpaqueKey::ECDSA(ptr) => (&*ptr).extended_public_key_data(),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => (&*ptr).extended_public_key_data(),
        OpaqueKey::ED25519(ptr) => (&*ptr).extended_public_key_data(),
    }.into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_extended_private_key_data(key: *mut OpaqueKey) -> ByteArray {
    match *key {
        OpaqueKey::ECDSA(ptr) => (&*ptr).extended_private_key_data(),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => (&*ptr).extended_private_key_data(),
        OpaqueKey::ED25519(ptr) => (&*ptr).extended_private_key_data()
    }.into()
}

// - (DSKey *)privateKeyAtIndexPath:(NSIndexPath *)indexPath fromSeed:(NSData *)seed;
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_private_key_at_index_path(seed: *const u8, seed_length: usize, key_type: KeyKind, index_path: *const IndexPathData, derivation_indexes: *const u8, derivation_hardened: *const bool, derivation_len: usize) -> *mut OpaqueKey {
    let seed_bytes = slice::from_raw_parts(seed, seed_length);
    let path = IndexPath::from((derivation_indexes, derivation_hardened, derivation_len));
    key_type.key_with_seed_data(seed_bytes)
        .and_then(|top_key| top_key.private_derive_to_256bit_derivation_path(&path))
        .and_then(|path_extended_key| path_extended_key.private_derive_to_path(&IndexPath::from(index_path)))
        .to_opaque_ptr()
}

// - (DSKey *)publicKeyAtIndexPath:(NSIndexPath *)indexPath;
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_public_key_at_index_path(key: *mut OpaqueKey, index_path: *const IndexPathData) -> *mut OpaqueKey {
    let index_path = IndexPath::from(index_path);
    match *key {
        OpaqueKey::ECDSA(ptr) => ECDSAKey::public_key_from_extended_public_key_data_at_index_path(&*ptr, &index_path).to_opaque_ptr(),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => BLSKey::public_key_from_extended_public_key_data_at_index_path(&*ptr, &index_path).to_opaque_ptr(),
        OpaqueKey::ED25519(ptr) => ED25519Key::public_key_from_extended_public_key_data_at_index_path(&*ptr, &index_path).to_opaque_ptr(),
    }
}

// - (NSData *)publicKeyDataAtIndexPath:(NSIndexPath *)indexPath;
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_public_key_data_at_index_path(key_ptr: *mut OpaqueKey, index_path: *const IndexPathData) -> ByteArray {
    let path = IndexPath::from(index_path);
    match *key_ptr {
        OpaqueKey::ECDSA(key) => (&*key).extended_public_key_data()
            .and_then(|data| ECDSAKey::public_key_from_extended_public_key_data(&data, &path)),
        OpaqueKey::BLSLegacy(key) => (&*key).extended_public_key_data()
            .and_then(|data| BLSKey::public_key_from_extended_public_key_data(&data, &path, true)),
        OpaqueKey::BLSBasic(key) => (&*key).extended_public_key_data()
            .and_then(|data| BLSKey::public_key_from_extended_public_key_data(&data, &path, false)),
        OpaqueKey::ED25519(key) => (&*key).extended_public_key_data()
            .and_then(|data| ED25519Key::public_key_from_extended_public_key_data(&data, &path)),
    }.into()
}

//- (NSArray *)privateKeysAtIndexPaths:(NSArray *)indexPaths fromSeed:(NSData *)seed;
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_private_keys_at_index_paths(
    seed: *const u8, seed_len: usize, key_type: KeyKind,
    index_paths: *const IndexPathData,
    index_paths_len: usize,
    derivation_indexes: *const u8,
    derivation_hardened: *const bool,
    derivation_len: usize) -> *mut OpaqueKeys {
    let seed_bytes = slice::from_raw_parts(seed, seed_len);
    let index_paths = slice::from_raw_parts(index_paths, index_paths_len);
    let derivation_path = IndexPath::from((derivation_indexes, derivation_hardened, derivation_len));
    key_type.key_with_seed_data(seed_bytes)
        .and_then(|top_key| top_key.private_derive_to_256bit_derivation_path(&derivation_path))
        .map_or(null_mut(), |derivation_path_extended_key| {
            let keys = index_paths.iter()
                .map(|p| derivation_path_extended_key.private_derive_to_path(&IndexPath::from(p as *const IndexPathData))
                    .map(|private_key| private_key.to_opaque_ptr()))
                .flatten()
                .collect::<Vec<_>>();
            let len = keys.len();
            boxed(OpaqueKeys { keys: boxed_vec(keys), len })
        })
}

//- (NSArray *)serializedPrivateKeysAtIndexPaths:(NSArray *)indexPaths fromSeed:(NSData *)seed
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn serialized_key_private_keys_at_index_paths(
    seed: *const u8, seed_len: usize, key_type: KeyKind,
    index_paths: *const IndexPathData,
    index_paths_len: usize,
    derivation_indexes: *const u8,
    derivation_hardened: *const bool,
    derivation_len: usize,
    chain_type: ChainType,
) -> *mut OpaqueSerializedKeys {
    let seed_bytes = slice::from_raw_parts(seed, seed_len);
    let index_paths = slice::from_raw_parts(index_paths, index_paths_len);
    let derivation_path = IndexPath::from((derivation_indexes, derivation_hardened, derivation_len));
    key_type.key_with_seed_data(seed_bytes)
        .and_then(|top_key| top_key.private_derive_to_256bit_derivation_path(&derivation_path))
        .map_or(null_mut(), |derivation_path_extended_key| {
            let script = chain_type.script_map();
            let keys = index_paths.iter()
                .map(|p| derivation_path_extended_key.private_derive_to_path(&IndexPath::from(p as *const IndexPathData))
                    .map(|private_key| CString::new(private_key.serialized_private_key_for_script(&script))
                        .unwrap()
                        .into_raw()))
                .flatten()
                .collect::<Vec<_>>();
            let len = keys.len();
            boxed(OpaqueSerializedKeys { keys: boxed_vec(keys), len })
        })
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_private_derive_to_path(key: *mut BLSKey, index_path: *const IndexPathData) -> *mut BLSKey {
    let key = &mut *key;
    key.private_derive_to_path(&IndexPath::from(index_path))
        .map_or(null_mut(), boxed)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_public_derive_to_path(key: *mut BLSKey, index_path: *const IndexPathData) -> *mut BLSKey {
    let key = &mut *key;
    key.public_derive_to_path(&IndexPath::from(index_path))
        .map_or(null_mut(), boxed)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_sign_data(key: *mut BLSKey, ptr: *const u8, len: usize) -> ByteArray {
    let key = &mut *key;
    let data = slice::from_raw_parts(ptr, len);
    key.sign_data(data).into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_with_seed_data(data: *const u8, len: usize, use_legacy: bool) -> *mut BLSKey {
    let seed_data = slice::from_raw_parts(data, len);
    boxed(BLSKey::key_with_seed_data(seed_data, use_legacy))
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_with_bip32_seed_data(data: *const u8, len: usize, use_legacy: bool) -> *mut BLSKey {
    let seed_data = slice::from_raw_parts(data, len);
    BLSKey::extended_private_key_with_seed_data(seed_data, use_legacy)
        .map_or(null_mut(), boxed)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_fingerprint(key: *mut BLSKey) -> u32 {
    (&*key).public_key_fingerprint()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_sign_data_single_sha256(key: *mut BLSKey, data: *const u8, len: usize) -> ByteArray {
    let data_to_sign = slice::from_raw_parts(data, len);
    (&*key).sign_data_single_sha256(data_to_sign).into()
}



/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_public_key(key: *mut BLSKey) -> ByteArray {
    (&*key).pubkey.into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_chaincode(key: *mut BLSKey) -> ByteArray {
    (&*key).chaincode().into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_serialize(key: *mut BLSKey, legacy: bool) -> ByteArray {
    (&*key).bls_public_key()
        .map(|key| UInt384(*if legacy { key.serialize_legacy() } else { key.serialize() }))
        .ok()
        .into()

}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_secret_key(key: *mut BLSKey) -> ByteArray {
    (&*key).seckey.into()
}

/// # Safety
/// public_key: UInt384
/// digest: UInt256
/// signature: UInt768
#[no_mangle]
pub unsafe extern "C" fn key_bls_verify(public_key: *const u8, use_legacy: bool, digest: *const u8, signature: *const u8) -> bool {
    let public_key = slice::from_raw_parts(public_key, UInt384::SIZE);
    let pubkey = UInt384::from(public_key);
    let message_digest = slice::from_raw_parts(digest, UInt256::SIZE);
    let signature = slice::from_raw_parts(signature, UInt768::SIZE);
    BLSKey::key_with_public_key(pubkey, use_legacy)
        .verify(message_digest, signature)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_migrate_from_legacy_extended_public_key_data(ptr: *const u8, len: usize) -> *mut OpaqueKey {
    let bytes = slice::from_raw_parts(ptr, len);
    BLSKey::migrate_from_legacy_extended_public_key_data(bytes)
        .to_opaque_ptr()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_bls_migrate_from_basic_extended_public_key_data(ptr: *const u8, len: usize) -> *mut OpaqueKey {
    let bytes = slice::from_raw_parts(ptr, len);
    BLSKey::migrate_from_basic_extended_public_key_data(bytes)
        .to_opaque_ptr()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_public_key_data(key: *mut ECDSAKey) -> ByteArray {
    (&*key).public_key_data().into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_secret_key_is_empty(key: *const c_char, chain_type: ChainType) -> bool {
    ECDSAKey::key_with_private_key(CStr::from_ptr(key).to_str().unwrap(), chain_type)
        .map_or(true, |key| key.seckey.is_zero())
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_compact_sign(key: *mut ECDSAKey, digest: *const u8) -> ByteArray {
    UInt256::from_const(digest)
        .map(|message_digest| (&mut *key).compact_sign(message_digest))
        .into()
}

/// # Safety
/// decrypts & serializes a BIP38 key using the given passphrase or returns NULL if passphrase is incorrect
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_with_bip38_key(private_key: *const c_char, passphrase: *const c_char, chain_type: ChainType) -> *mut c_char {
    let private_key = CStr::from_ptr(private_key).to_str().unwrap();
    let passphrase = CStr::from_ptr(passphrase).to_str().unwrap();
    let script = chain_type.script_map();
    ECDSAKey::key_with_bip38_key(private_key, passphrase, &script)
        .map(|key| key.serialized_private_key_for_script(&script))
        .to_c_string_ptr()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_sign(key: *mut ECDSAKey, data: *const u8, len: usize) -> ByteArray {
    let key = unsafe { &mut *key };
    let data = slice::from_raw_parts(data, len);
    key.sign(data).into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_with_seed_data(ptr: *const u8, len: usize) -> *mut ECDSAKey {
    let seed = slice::from_raw_parts(ptr, len);
    ECDSAKey::init_with_seed_data(seed)
        .map_or(null_mut(), boxed)
}

//+ (NSString *)serializedPrivateMasterFromSeedData:(NSData *)seedData forChain:(DSChain *)chain
/// # Safety
/// For test only
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_serialized_private_master_from_seed_data(ptr: *const u8, len: usize, chain_type: ChainType) -> *mut c_char {
    let seed = slice::from_raw_parts(ptr, len);
    if seed.is_empty() {
        return null_mut();
    }
    let seed_key = UInt512::bip32_seed_key(seed);
    let key = bip32::Key::new(0, 0, UInt256::MIN, UInt256::from(&seed_key.0[32..]), seed_key.0[..32].to_vec(), false)
        .serialize(chain_type);
    CString::new(key).unwrap().into_raw()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_with_private_key(secret: *const c_char, chain_type: ChainType) -> *mut ECDSAKey {
    let c_str = unsafe { CStr::from_ptr(secret) };
    let private_key_string = c_str.to_str().unwrap();
    ECDSAKey::key_with_private_key(private_key_string, chain_type)
        .map_or(null_mut(), |key| boxed(key))
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_public_key_data_for_private_key(secret: *const c_char, chain_type: ChainType) -> ByteArray {
    let c_str = unsafe { CStr::from_ptr(secret) };
    let private_key_string = c_str.to_str().unwrap();
    ECDSAKey::key_with_private_key(private_key_string, chain_type)
        .map(|key| key.public_key_data())
        .into()
}
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_has_private_key(key: *mut ECDSAKey) -> bool {
    (&*key).has_private_key()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_serialized_private_key_for_chain(key: *mut ECDSAKey, chain_type: ChainType) -> *mut c_char {
    (&*key).serialized_private_key_for_script(&chain_type.script_map())
        .to_c_string_ptr()
}

// + (NSString *)serializedAuthPrivateKeyFromSeed:(NSData *)seed forChain:(DSChain *)chain
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_serialized_auth_private_key_for_chain(seed: *const u8, seed_len: usize, chain_type: ChainType) -> *mut c_char {
    let seed = slice::from_raw_parts(seed, seed_len);
    let script_map = chain_type.script_map();
    ECDSAKey::serialized_auth_private_key_from_seed(seed, script_map)
        .to_c_string_ptr()
}

/// # Safety
#[no_mangle]
pub extern "C" fn key_ecdsa_recovered_from_compact_sig(data: *const u8, len: usize, digest: *const u8) -> *mut OpaqueKey {
    let compact_sig = unsafe { slice::from_raw_parts(data, len) };
    UInt256::from_const(digest)
        .and_then(|message_digest| ECDSAKey::key_with_compact_sig(compact_sig, message_digest))
        .to_opaque_ptr()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_ecdsa_verify_compact_sig(signature: *const u8, signature_len: usize, payload: *const u8, payload_len: usize, owner_key_hash: *const u8) -> bool {
    let compact_sig = slice::from_raw_parts(signature, signature_len);
    let payload = slice::from_raw_parts(payload, payload_len);
    let message_digest = UInt256::sha256d(payload);
    let owner_key_hash = UInt160::from_const(owner_key_hash)
        .expect("Owner key hash has wrong length");
    ECDSAKey::key_with_compact_sig(compact_sig, message_digest)
        .map_or(false, |key| key.hash160().eq(&owner_key_hash))

}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_create_ecdsa_from_secret(ptr: *const u8, len: usize, compressed: bool) -> *mut OpaqueKey {
    let bytes = slice::from_raw_parts(ptr, len);
    ECDSAKey::key_with_secret_data(bytes, compressed)
        .to_opaque_ptr()
}


/// Deserializes extended private key from string and create opaque pointer to ECDSAKey
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_create_ecdsa_from_serialized_extended_private_key(key: *const c_char, chain_type: ChainType) -> *mut ECDSAKey {
    // NSData *extendedPrivateKey = [self deserializedExtendedPrivateKey:serializedExtendedPrivateKey onChain:chain];
    // [DSECDSAKey keyWithSecret:*(UInt256 *)extendedPrivateKey.bytes compressed:YES];
    (CStr::from_ptr(key).to_str().unwrap(), chain_type)
        .try_into()
        .ok()
        .and_then(|key: bip32::Key| ECDSAKey::key_with_secret_data(&key.extended_key_data(), true))
        .map_or(null_mut(), boxed)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_create_ecdsa_from_extended_public_key_data(ptr: *const u8, len: usize) -> *mut OpaqueKey {
    let bytes = slice::from_raw_parts(ptr, len);
    ECDSAKey::key_with_extended_public_key_data(bytes)
        .to_opaque_ptr()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_create_with_private_key_data(ptr: *const u8, len: usize, key_type: KeyKind) -> *mut OpaqueKey {
    let bytes = slice::from_raw_parts(ptr, len);
    match key_type {
        KeyKind::ECDSA => ECDSAKey::key_with_secret_data(bytes, true).to_opaque_ptr(),
        KeyKind::ED25519 => ED25519Key::key_with_secret_data(bytes).to_opaque_ptr(),
        KeyKind::BLS => BLSKey::key_with_private_key_data(bytes, true).to_opaque_ptr(),
        KeyKind::BLSBasic => BLSKey::key_with_private_key_data(bytes, false).to_opaque_ptr()
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_create_with_public_key_data(ptr: *const u8, len: usize, key_type: KeyKind) -> *mut OpaqueKey {
    let bytes = slice::from_raw_parts(ptr, len);
    match key_type {
        KeyKind::ECDSA => ECDSAKey::key_with_public_key_data(bytes).to_opaque_ptr(),
        KeyKind::ED25519 => ED25519Key::key_with_public_key_data(bytes).to_opaque_ptr(),
        KeyKind::BLS => BLSKey::key_with_public_key(UInt384::from(bytes), true).to_opaque_ptr(),
        KeyKind::BLSBasic => BLSKey::key_with_public_key(UInt384::from(bytes), false).to_opaque_ptr()
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_create_from_extended_public_key_data(ptr: *const u8, len: usize, key_type: KeyKind) -> *mut OpaqueKey {
    let bytes = slice::from_raw_parts(ptr, len);
    match key_type {
        KeyKind::ECDSA => ECDSAKey::key_with_extended_public_key_data(bytes).to_opaque_ptr(),
        KeyKind::ED25519 => ED25519Key::key_with_extended_public_key_data(bytes).to_opaque_ptr(),
        KeyKind::BLS => BLSKey::key_with_extended_public_key_data(bytes, true).to_opaque_ptr(),
        KeyKind::BLSBasic => BLSKey::key_with_extended_public_key_data(bytes, false).to_opaque_ptr()
    }
}

/// Deserializes extended private key from string and create opaque pointer to ECDSAKey
/// To pass NSIndexPath need to be serialized as byte array with u264 with path_length = bytes.length / 33
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_serialized_extended_private_key_from_seed(
    secret: *const u8,
    secret_len: usize,
    derivation_indexes: *const u8,
    derivation_hardened: *const bool,
    derivation_len: usize,
    chain_type: ChainType) -> *mut c_char {
    let secret_slice = unsafe { slice::from_raw_parts(secret, secret_len) };
    let index_path = IndexPath::from((derivation_indexes, derivation_hardened, derivation_len));
    ECDSAKey::serialized_extended_private_key_from_seed(secret_slice, index_path, chain_type)
        .to_c_string_ptr()
}

/// # Safety
#[no_mangle]
pub extern "C" fn ecdsa_public_key_hash_from_secret(secret: *const c_char, chain_type: ChainType) -> ByteArray {
    let c_str = unsafe { CStr::from_ptr(secret) };
    let private_key_string = c_str.to_str().unwrap();
    ECDSAKey::key_with_private_key(private_key_string, chain_type)
        .map(|key| key.hash160())
        .into()
}



/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_address_for_key(key: *mut OpaqueKey, chain_type: ChainType) -> *mut c_char {
    let script_map = chain_type.script_map();
    CString::new(match *key {
        OpaqueKey::ECDSA(ptr) => (&*ptr).address_with_public_key_data(&script_map),
        OpaqueKey::BLSLegacy(ptr) |
        OpaqueKey::BLSBasic(ptr) => (&*ptr).address_with_public_key_data(&script_map),
        OpaqueKey::ED25519(ptr) => (&*ptr).address_with_public_key_data(&script_map)
    }).unwrap().into_raw()
}
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_address_with_public_key_data(data: *const u8, len: usize, chain_type: ChainType) -> *mut c_char {
    let map = chain_type.script_map();
    let data = slice::from_raw_parts(data, len);
    address::with_public_key_data(data, &map)
        .to_c_string_ptr()
}
/// # Safety
#[no_mangle]
pub extern "C" fn address_for_ecdsa_key(key: *mut ECDSAKey, chain_type: ChainType) -> *mut c_char {
    let key = unsafe { &*key };
    let script_map = chain_type.script_map();
    key.address_with_public_key_data(&script_map).to_c_string_ptr()
}
/// # Safety
#[no_mangle]
pub extern "C" fn address_for_bls_key(key: *mut BLSKey, chain_type: ChainType) -> *mut c_char {
    let key = unsafe { &*key };
    let script_map = chain_type.script_map();
    key.address_with_public_key_data(&script_map).to_c_string_ptr()
}
/// # Safety
#[no_mangle]
pub extern "C" fn address_for_ed25519_key(key: *mut ED25519Key, chain_type: ChainType) -> *mut c_char {
    let key = unsafe { &*key };
    let script_map = chain_type.script_map();
    key.address_with_public_key_data(&script_map).to_c_string_ptr()
}






/// # Safety
#[no_mangle]
pub extern "C" fn address_for_ecdsa_key_recovered_from_compact_sig(data: *const u8, len: usize, digest: *const u8, chain_type: ChainType) -> *mut c_char {
    let compact_sig = unsafe { slice::from_raw_parts(data, len) };
    let script_map = chain_type.script_map();
    UInt256::from_const(digest)
        .and_then(|message_digest| ECDSAKey::key_with_compact_sig(compact_sig, message_digest))
        .map(|key| key.address_with_public_key_data(&script_map))
        .to_c_string_ptr()
}
/// # Safety
#[no_mangle]
pub extern "C" fn ecdsa_public_key_unique_id_from_derived_key_data(data: *const u8, len: usize, chain_type: ChainType) -> u64 {
    let derived_key_data = unsafe { slice::from_raw_parts(data, len) };
    let seed_key = UInt512::bip32_seed_key(derived_key_data);
    let secret = UInt256::from(&seed_key.0[..32]);
    ECDSAKey::key_with_secret(&secret, true)
        .map_or(0, |public_key| {
            let data = public_key.public_key_data();
            let mut writer = SecVec::new();
            chain_type.genesis_hash().enc(&mut writer);
            writer.extend(data);
            // one way injective function?
            UInt256::sha256(writer.as_slice()).u64_le()
        })
}

/// # Safety
#[no_mangle]
pub extern "C" fn ecdsa_address_from_public_key_data(data: *const u8, len: usize, chain_type: ChainType) -> *mut c_char {
    let public_key_data = unsafe { slice::from_raw_parts(data, len) };
    ECDSAKey::key_with_public_key_data(public_key_data)
        .map(|key| key.address_with_public_key_data(&chain_type.script_map()))
        .to_c_string_ptr()
}



// - (DSKey *)generateExtendedPublicKeyFromSeed:(NSData *)seed storeUnderWalletUniqueId:(NSString *)walletUniqueId storePrivateKey:(BOOL)storePrivateKey;
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn generate_extended_public_key_from_seed(seed: *const u8, seed_length: usize, key_type: KeyKind, derivation_indexes: *const u8, derivation_hardened: *const bool, derivation_len: usize) -> *mut OpaqueKey {
    let seed_bytes = slice::from_raw_parts(seed, seed_length);
    let derivation_path = IndexPath::from((derivation_indexes, derivation_hardened, derivation_len));
    key_type.key_with_seed_data(seed_bytes)
        .and_then(|seed_key| seed_key.private_derive_to_256bit_derivation_path(&derivation_path))
        .to_opaque_ptr()
}




// - (DSKey *)deprecatedIncorrectExtendedPublicKeyFromSeed:(NSData *)seed;
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn deprecated_incorrect_extended_public_key_from_seed(seed: *const u8, seed_len: usize, derivation_indexes: *const u8, derivation_hardened: *const bool, derivation_len: usize) -> *mut OpaqueKey {
    let i = UInt512::bip32_seed_key(slice::from_raw_parts(seed, seed_len));
    let secret = &i.0[..32];
    let mut writer = SecVec::new();
    let mut chaincode = UInt256::from(&i.0[32..]);
    let derivation_path = IndexPath::from((derivation_indexes, derivation_hardened, derivation_len));
    ECDSAKey::key_with_secret_data(secret, true)
        .and_then(|key| {
            key.hash160().u32_le().enc(&mut writer);
            let mut key = UInt256::from(secret);
            let hashes = unsafe { slice::from_raw_parts(derivation_indexes, derivation_len * 32) };
            (0..derivation_len).into_iter().for_each(|position| {
                // let index = (*derivation_path).indexes.offset(position as isize);
                // let index = derivation_indexes.add(position);
                // let slice = slice::from_raw_parts(index as *const u8, 8);
                let soft_index = hashes.read_with::<u64>(&mut position.clone(), byte::LE).unwrap() as u32;
                // let soft_index = slice.read_with::<u64>(&mut 0, byte::BE).unwrap() as u32;
                let buf = &mut [0u8; 37];
                if soft_index & BIP32_HARD != 0 {
                    buf[1..33].copy_from_slice(&key.0);
                } else {
                    buf[..33].copy_from_slice(&secp256k1_point_from_bytes(&key.0));
                }
                buf[33..37].copy_from_slice(soft_index.to_be_bytes().as_slice());
                let i = UInt512::hmac(chaincode.as_ref(), buf);
                let mut sec_key = secp256k1::SecretKey::from_slice(&key.0).expect("invalid private key");
                let tweak = Scalar::from_be_bytes(clone_into_array(&i.0[..32])).expect("invalid tweak");
                sec_key = sec_key.add_tweak(&tweak).expect("failed to add tweak");
                key.0.copy_from_slice(&sec_key.secret_bytes());
                chaincode.0.copy_from_slice(&i.0[32..]);
            });
            if let Some(seckey) = ECDSAKey::key_with_secret(&key, true) {
                chaincode.enc(&mut writer);
                writer.extend(seckey.public_key_data());
                ECDSAKey::key_with_extended_public_key_data(&writer)
            } else {
                None
            }
        })
        .to_opaque_ptr()
}

// + (NSData *)deserializedExtendedPrivateKey:(NSString *)extendedPrivateKeyString onChain:(DSChain *)chain;
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn deserialized_extended_private_key(ptr: *const c_char, chain_type: ChainType) -> ByteArray {
    (CStr::from_ptr(ptr).to_str().unwrap(), chain_type)
        .try_into()
        .ok()
        .map(|key: bip32::Key| key.extended_key_data())
        .into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn keys_private_key_data_is_equal(key1_ptr: *mut OpaqueKey, key2_ptr: *mut OpaqueKey) -> bool {
    let seckey1 = match *key1_ptr {
        OpaqueKey::ECDSA(key) => (&*key).seckey,
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => (&*key).seckey,
        OpaqueKey::ED25519(key) => (&*key).seckey
    };
    let seckey2 = match *key2_ptr {
        OpaqueKey::ECDSA(key) => (&*key).seckey,
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => (&*key).seckey,
        OpaqueKey::ED25519(key) => (&*key).seckey
    };
    seckey1 == seckey2
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn keys_public_key_data_is_equal(key1_ptr: *mut OpaqueKey, key2_ptr: *mut OpaqueKey) -> bool {
    let pubkey_data1 = match *key1_ptr {
        OpaqueKey::ECDSA(key) => (&*key).public_key_data(),
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => (&*key).public_key_data(),
        OpaqueKey::ED25519(key) => (&*key).public_key_data()
    };
    let pubkey_data2 = match *key2_ptr {
        OpaqueKey::ECDSA(key) => (&*key).public_key_data(),
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => (&*key).public_key_data(),
        OpaqueKey::ED25519(key) => (&*key).public_key_data()
    };
    pubkey_data1 == pubkey_data2
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_check_payload_signature(key_ptr: *mut OpaqueKey, key_hash: *const u8) -> bool {
    let key_hash = slice::from_raw_parts(key_hash, UInt160::SIZE);
    match *key_ptr {
        OpaqueKey::ECDSA(key) => (&*key).hash160().as_bytes().eq(key_hash),
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => (&*key).hash160().as_bytes().eq(key_hash),
        OpaqueKey::ED25519(key) => (&*key).hash160().as_bytes().eq(key_hash),
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_secret_key_string(ptr: *mut OpaqueKey) -> *mut c_char {
    match *ptr {
        OpaqueKey::ECDSA(key) => (&*key).secret_key_string(),
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => (&*key).secret_key_string(),
        OpaqueKey::ED25519(key) => (&*key).secret_key_string(),
    }.to_c_string_ptr()

}


/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_encrypt_data(data: *const u8, len: usize, private_key: *mut OpaqueKey, public_key: *mut OpaqueKey) -> ByteArray {
    let data = slice::from_raw_parts(data, len);
    match (&*private_key, &mut *public_key) {
        (OpaqueKey::ECDSA(prv_ptr), OpaqueKey::ECDSA(pub_ptr)) =>
            ECDSAKey::init_with_dh_key_exchange_with_public_key(&mut *(*pub_ptr), &*(*prv_ptr))
                .and_then(|key| <Vec<u8> as CryptoData<ECDSAKey>>::encrypt_with_dh_key(&mut data.to_vec(), &key)),
        (OpaqueKey::BLSLegacy(prv_ptr), OpaqueKey::BLSLegacy(pub_ptr)) |
        (OpaqueKey::BLSBasic(prv_ptr), OpaqueKey::BLSBasic(pub_ptr)) =>
            BLSKey::init_with_dh_key_exchange_with_public_key(&mut *(*pub_ptr), &*(*prv_ptr))
                .and_then(|key| <Vec<u8> as CryptoData<BLSKey>>::encrypt_with_dh_key(&mut data.to_vec(), &key)),
        _ => None
    }.into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_encrypt_data_using_iv(data: *const u8, len: usize, private_key: *mut OpaqueKey, public_key: *mut OpaqueKey, iv_data: *const u8, iv_len: usize) -> ByteArray {
    let data = slice::from_raw_parts(data, len);
    let iv = slice::from_raw_parts(iv_data, iv_len);
    match (&*private_key, &*public_key) {
        (OpaqueKey::ECDSA(prv_ptr), OpaqueKey::ECDSA(pub_ptr)) =>
            ECDSAKey::init_with_dh_key_exchange_with_public_key(&mut *(*pub_ptr), &*(*prv_ptr))
                .and_then(|key| <Vec<u8> as CryptoData<ECDSAKey>>::encrypt_with_dh_key_using_iv(&mut data.to_vec(), &key, iv.to_vec())),
        (OpaqueKey::BLSLegacy(prv_ptr), OpaqueKey::BLSLegacy(pub_ptr)) |
        (OpaqueKey::BLSBasic(prv_ptr), OpaqueKey::BLSBasic(pub_ptr)) =>
            BLSKey::init_with_dh_key_exchange_with_public_key(&mut *(*pub_ptr), &*(*prv_ptr))
                .and_then(|key| <Vec<u8> as CryptoData<BLSKey>>::encrypt_with_dh_key_using_iv(&mut data.to_vec(), &key, iv.to_vec())),
        _ => None
    }.into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_decrypt_data(data: *const u8, len: usize, private_key: *mut OpaqueKey, public_key: *mut OpaqueKey) -> ByteArray {
    let data = slice::from_raw_parts(data, len);
    match (&*private_key, &*public_key) {
        (OpaqueKey::ECDSA(prv_ptr), OpaqueKey::ECDSA(pub_ptr)) =>
            ECDSAKey::init_with_dh_key_exchange_with_public_key(&mut *(*pub_ptr), &*(*prv_ptr))
                .and_then(|key| <Vec<u8> as CryptoData<ECDSAKey>>::decrypt_with_dh_key(&mut data.to_vec(), &key)),
        (OpaqueKey::BLSLegacy(prv_ptr), OpaqueKey::BLSLegacy(pub_ptr)) |
        (OpaqueKey::BLSBasic(prv_ptr), OpaqueKey::BLSBasic(pub_ptr)) =>
            BLSKey::init_with_dh_key_exchange_with_public_key(&mut *(*pub_ptr), &*(*prv_ptr))
                .and_then(|key| <Vec<u8> as CryptoData<BLSKey>>::decrypt_with_dh_key(&mut data.to_vec(), &key)),
        _ => None
    }.into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_decrypt_data_using_iv_size(data: *const u8, len: usize, private_key: *mut OpaqueKey, public_key: *mut OpaqueKey, iv_size: usize) -> ByteArray {
    let data = slice::from_raw_parts(data, len);
    match (&*private_key, &*public_key) {
        (OpaqueKey::ECDSA(prv_ptr), OpaqueKey::ECDSA(pub_ptr)) =>
            ECDSAKey::init_with_dh_key_exchange_with_public_key(&mut *(*pub_ptr), &*(*prv_ptr))
                .and_then(|key| <Vec<u8> as CryptoData<ECDSAKey>>::decrypt_with_dh_key_using_iv_size(&mut data.to_vec(), &key, iv_size)),
        (OpaqueKey::BLSLegacy(prv_ptr), OpaqueKey::BLSLegacy(pub_ptr)) |
        (OpaqueKey::BLSBasic(prv_ptr), OpaqueKey::BLSBasic(pub_ptr)) =>
            BLSKey::init_with_dh_key_exchange_with_public_key(&mut *(*pub_ptr), &*(*prv_ptr))
                .and_then(|key| <Vec<u8> as CryptoData<BLSKey>>::decrypt_with_dh_key_using_iv_size(&mut data.to_vec(), &key, iv_size)),
        _ => None
    }.into()
}


/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_encrypt_data_with_dh_key(data: *const u8, len: usize, key_ptr: *mut OpaqueKey) -> ByteArray {
    let data = slice::from_raw_parts(data, len);
    match *key_ptr {
        OpaqueKey::ECDSA(key) => <Vec<u8> as CryptoData<ECDSAKey>>::encrypt_with_dh_key(&mut data.to_vec(), &*key),
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => <Vec<u8> as CryptoData<BLSKey>>::encrypt_with_dh_key(&mut data.to_vec(), &*key),
        _ => None
    }.into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_decrypt_data_with_dh_key(data: *const u8, len: usize, key_ptr: *mut OpaqueKey) -> ByteArray {
    let data = slice::from_raw_parts(data, len);
    match *key_ptr {
        OpaqueKey::ECDSA(key) => <Vec<u8> as CryptoData<ECDSAKey>>::decrypt_with_dh_key(&mut data.to_vec(), &*key),
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => <Vec<u8> as CryptoData<BLSKey>>::decrypt_with_dh_key(&mut data.to_vec(), &*key),
        _ => None
    }.into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_encrypt_data_with_dh_key_using_iv(data: *const u8, len: usize, key_ptr: *mut OpaqueKey, iv_data: *const u8, iv_len: usize) -> ByteArray {
    let data = slice::from_raw_parts(data, len);
    let iv = slice::from_raw_parts(iv_data, iv_len);
    match *key_ptr {
        OpaqueKey::ECDSA(key) =>
            <Vec<u8> as CryptoData<ECDSAKey>>::encrypt_with_dh_key_using_iv(&mut data.to_vec(), &*key, iv.to_vec()),
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) =>
            <Vec<u8> as CryptoData<BLSKey>>::encrypt_with_dh_key_using_iv(&mut data.to_vec(), &*key, iv.to_vec()),
        _ => None
    }.into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_decrypt_data_with_dh_key_using_iv_size(data: *const u8, len: usize, key_ptr: *mut OpaqueKey, iv_size: usize) -> ByteArray {
    let data = slice::from_raw_parts(data, len);
    match *key_ptr {
        OpaqueKey::ECDSA(key) => <Vec<u8> as CryptoData<ECDSAKey>>::decrypt_with_dh_key_using_iv_size(&mut data.to_vec(), &*key, iv_size),
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => <Vec<u8> as CryptoData<BLSKey>>::decrypt_with_dh_key_using_iv_size(&mut data.to_vec(), &*key, iv_size),
        _ => None
    }.into()
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn key_create_account_reference(source_key: *mut OpaqueKey, extended_public_key: *mut OpaqueKey, account_number: usize) -> u32 {
    let extended_public_key_data = match *extended_public_key {
        OpaqueKey::ECDSA(key) => (&*key).extended_public_key_data(),
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => (&*key).extended_public_key_data(),
        OpaqueKey::ED25519(key) => (&*key).extended_public_key_data(),
    }.unwrap_or(vec![]);

    let account_secret_key = match *source_key  {
        OpaqueKey::ECDSA(key) => (&*key).hmac_256_data(&extended_public_key_data),
        OpaqueKey::BLSLegacy(key) |
        OpaqueKey::BLSBasic(key) => (&*key).hmac_256_data(&extended_public_key_data),
        OpaqueKey::ED25519(key) => (&*key).hmac_256_data(&extended_public_key_data)
    }.reversed();
    let account_secret_key28 = account_secret_key.u32_le() >> 4;
    let shortened_account_bits = (account_number as u32) & 0x0FFFFFFF;
    let version = 0; // currently set to 0
    let version_bits = version << 28;
    // this is the account ref
    return version_bits | (account_secret_key28 ^ shortened_account_bits)
}

/// # Safety
/// decrypts & serializes a BIP38 key using the given passphrase or returns NULL if passphrase is incorrect
#[no_mangle]
pub unsafe extern "C" fn key_is_valid_bip38_key(key: *const c_char) -> bool {
    let key = CStr::from_ptr(key).to_str().unwrap();
    ECDSAKey::is_valid_bip38_key(key)
}
