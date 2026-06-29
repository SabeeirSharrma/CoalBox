use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use zeroize::Zeroize;

use crate::error::CoalboxError;
use crate::format::{NONCE_LEN, SALT_LEN};

const ARGON2_MEMORY_COST: u32 = 65536; // 64 MB
const ARGON2_TIME_COST: u32 = 3;
const ARGON2_PARALLELISM: u32 = 4;

pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn derive_key(password: &str, salt: &[u8; SALT_LEN]) -> Result<[u8; 32], CoalboxError> {
    let params = Params::new(
        ARGON2_MEMORY_COST,
        ARGON2_TIME_COST,
        ARGON2_PARALLELISM,
        Some(32),
    )
    .map_err(|e| CoalboxError::Argon2(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| CoalboxError::Argon2(e.to_string()))?;

    Ok(key)
}

pub fn encrypt(plaintext: &[u8], key: &[u8; 32], nonce: &[u8; NONCE_LEN]) -> Result<(Vec<u8>, [u8; 16]), CoalboxError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CoalboxError::Crypto(e.to_string()))?;

    let nonce_obj = Nonce::from_slice(nonce);

    let ciphertext = cipher
        .encrypt(nonce_obj, plaintext)
        .map_err(|e| CoalboxError::Crypto(e.to_string()))?;

    // AES-GCM appends the tag to the ciphertext
    if ciphertext.len() < 16 {
        return Err(CoalboxError::Crypto("ciphertext too short".into()));
    }

    let tag_start = ciphertext.len() - 16;
    let tag = {
        let mut t = [0u8; 16];
        t.copy_from_slice(&ciphertext[tag_start..]);
        t
    };
    let encrypted = ciphertext[..tag_start].to_vec();

    Ok((encrypted, tag))
}

pub fn decrypt(
    encrypted: &[u8],
    key: &[u8; 32],
    nonce: &[u8; NONCE_LEN],
    tag: &[u8; 16],
) -> Result<Vec<u8>, CoalboxError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CoalboxError::Crypto(e.to_string()))?;

    let nonce_obj = Nonce::from_slice(nonce);

    // Reconstruct the full ciphertext with tag appended
    let mut full_ciphertext = Vec::with_capacity(encrypted.len() + 16);
    full_ciphertext.extend_from_slice(encrypted);
    full_ciphertext.extend_from_slice(tag);

    cipher
        .decrypt(nonce_obj, full_ciphertext.as_ref())
        .map_err(|_| CoalboxError::DecryptionFailed)
}

pub fn secure_zero(data: &mut [u8]) {
    data.zeroize();
}
