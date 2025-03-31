use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore, Aes256Gcm, KeyInit, Nonce,
};

use crate::constant::KEY;

pub struct AesEncryptor<'a> {
    key: &'a [u8],
}

impl AesEncryptor<'static> {
    pub fn initialize() -> Self {
        let key = KEY;
        let key = key.as_bytes();
        Self { key }
    }
}

impl AesEncryptor<'static> {
    pub fn decrypt_file(&self, path: &Path, file_path: &Path) {
        if let Ok(mut file) = File::open(path) {
            let mut encrypted_data = Vec::new();
            file.read_to_end(&mut encrypted_data).unwrap();

            let nonce = Nonce::from_slice(&encrypted_data[..12]);
            let cipher = Aes256Gcm::new_from_slice(self.key).unwrap();

            let ciphertext = &encrypted_data[12..];
            let plaintext = cipher.decrypt(&nonce, ciphertext).unwrap();

            let mut file = File::create(file_path).unwrap();
            file.write_all(&plaintext).unwrap();
        }
    }

    pub fn encrypt_file(&self, path: &Path, output: &Path) {
        let mut file = File::open(path).unwrap();
        let mut buf = Vec::new();

        file.read_to_end(&mut buf).unwrap();

        let cipher = Aes256Gcm::new_from_slice(self.key).unwrap();
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, buf.as_ref()).unwrap();

        let mut file = File::create(output).unwrap();
        file.write_all(&nonce).unwrap();
        file.write_all(&ciphertext).unwrap();
    }
}
