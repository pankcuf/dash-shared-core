use std::os::raw::c_void;

pub enum Operation {
    Encrypt = 0,
    Decrypt = 1,
}

extern "C" {
    /// Stateless, one-shot encrypt or decrypt operation. This basically performs a sequence of
    /// CCCrytorCreate(), CCCryptorUpdate(), CCCryptorFinal(), and CCCryptorRelease().
    ///
    /// # Arguments
    ///
    /// * `op`: Defines the basic operation: kCCEncrypt or kCCDecrypt.
    /// * `alg`: Defines the encryption algorithm.
    /// * `options`: A word of flags defining options. See discussion for the CCOptions type.
    /// * `key`: Raw key material, length keyLength bytes.
    /// * `key_length`: Length of key material. Must be appropriate for the select algorithm.
    /// Some algorithms may provide for varying key lengths.
    /// * `iv`: Initialization vector, optional. Used for Cipher Block Chaining (CBC) mode. If
    /// present, must be the same length as the selected algorithm's block size. If CBC mode is
    /// selected (by the absence of any mode bits in the options flags) and no IV is present, a
    /// NULL (all zeroes) IV will be used. This is ignored if ECB mode is used or if a stream
    /// cipher algorithm is selected. For sound encryption, always initialize IV with random data.
    /// * `data_in`: Data to encrypt or decrypt, length dataInLength bytes.
    /// * `data_in_length`: Length of data to encrypt or decrypt.
    /// * `data_out`: Result is written here. Allocated by caller. Encryption and decryption can be
    /// performed "in-place", with the same buffer used for input and output.
    /// * `data_out_available`: The size of the dataOut buffer in bytes.
    /// * `data_out_moved`: On successful return, the number of bytes written to dataOut. If
    /// kCCBufferTooSmall is returned as a result of insufficient buffer space being provided, the
    /// required buffer space is returned here.
    ///
    /// returns: CryptorStatus
    ///
    /// kCCBufferTooSmall: indicates insufficent space in the dataOut buffer. In this case, the *dataOutMoved parameter will indicate the size of the buffer needed to complete the operation. The operation can be retried with minimal runtime penalty.
    ///
    /// kCCAlignmentError: indicates that dataInLength was not properly aligned. This can only be returned for block ciphers, and then only when decrypting or when encrypting with block with padding disabled.
    ///
    /// kCCDecodeError: Indicates improperly formatted ciphertext or a "wrong key" error; occurs only during decrypt operations.
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    /// API_AVAILABLE(macos(10.4), ios(2.0));
    fn CCCrypt(
        operation: u32,
        alg: u32,
        options: u32,
        key: *const c_void,
        key_length: usize,
        iv: *const c_void,
        data_in: *const c_void,
        data_in_length: usize,
        data_out: *mut c_void,
        data_out_available: usize,
        data_out_moved: *mut usize,
    ) -> i32;
}

fn random_initialization_vector_of_size(size: usize) -> Vec<u8> {
    use secp256k1::rand;
    use secp256k1::rand::distributions::Uniform;
    use secp256k1::rand::Rng;
    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, 255);
    (0..size).map(|_| rng.sample(&range)).collect()
}

pub fn aes256_encrypt_decrypt(operation: Operation, data: impl AsRef<[u8]>, key: impl AsRef<[u8]>, iv: impl AsRef<[u8]>) -> Option<Vec<u8>> {
    let operation = match operation {
        Operation::Encrypt => 0, // kCCEncrypt
        Operation::Decrypt => 1, // kCCDecrypt
    };
    let alg = 0; // kCCAlgorithmAES
    let options = 0x0001; // kCCOptionPKCS7Padding
    let data_ref = data.as_ref();
    let key_ref = key.as_ref();
    let iv_ref = iv.as_ref();
    let data_in_len = data_ref.len();
    let data_out_len = data_in_len + 16; // Add space for kCCBlockSizeAES128
    let mut data_out = vec![0u8; data_out_len];
    let mut bytes_written: usize = 0;
    let result = unsafe {
        CCCrypt(
            operation as u32,
            alg,
            options,
            key_ref.as_ptr() as *const c_void,
            key_ref.len(),
            iv_ref.as_ptr() as *const c_void,
            data_ref.as_ptr() as *const c_void,
            data_in_len,
            data_out.as_mut_ptr() as *mut c_void,
            data_out_len,
            &mut bytes_written as *mut usize,
        )
    };

    if result == 0 {
        // kCCSuccess
        data_out.truncate(bytes_written);
        Some(data_out)
    } else {
        None
    }
}
