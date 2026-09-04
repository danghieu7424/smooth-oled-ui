// src/utils/crypto.rs
#![allow(dead_code)]
use aes_gcm::{aead::{Aead, KeyInit}, Aes128Gcm, Nonce};
use rand::{rngs::OsRng, RngCore};

pub fn encrypt_audio_to_bin(raw_audio: &[u8], content_key: &[u8; 16]) -> Vec<u8> {
    let cipher = Aes128Gcm::new_from_slice(content_key).unwrap();
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Mã hóa: Kết quả bao gồm Ciphertext + 16 bytes Tag
    let ciphertext = cipher.encrypt(nonce, raw_audio).expect("Encryption failed");

    // Đóng gói theo chuẩn: [12B Nonce] + [Data + Tag]
    let mut final_bin = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    final_bin.extend_from_slice(&nonce_bytes);
    final_bin.extend_from_slice(&ciphertext);
    final_bin
}