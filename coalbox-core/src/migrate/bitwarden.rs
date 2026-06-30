use std::fs::File;
use std::io::Write;
use std::path::Path;

use aes::cipher::block_padding::NoPadding;
use aes::cipher::{BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::entry::Entry;
use crate::error::CoalboxError;

fn derive_bitwarden_key(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
    let mut key = [0u8; 32];

    let argon2 = argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(64 * 1024, iterations, 4, Some(32)).unwrap(),
    );

    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("Key derivation failed");

    key.to_vec()
}

fn generate_bitwarden_json(entries: &[Entry]) -> serde_json::Value {
    let items: Vec<serde_json::Value> = entries
        .iter()
        .map(|entry| {
            let mut item = serde_json::json!({
                "id": entry.id.to_string(),
                "type": 1,
                "name": entry.title,
                "login": {
                    "username": entry.username,
                    "password": entry.password,
                    "totp": entry.totp_secret,
                },
                "notes": entry.notes,
                "fields": entry.tags.iter().map(|t| serde_json::json!({
                    "name": t,
                    "type": 0,
                    "value": ""
                })).collect::<Vec<_>>(),
                "creationDate": entry.created.to_rfc3339(),
                "modificationDate": entry.modified.to_rfc3339(),
            });

            if let Some(ref url) = entry.url {
                item["login"]["uris"] = serde_json::json!([{"uri": url}]);
            }

            item
        })
        .collect();

    serde_json::json!({
        "items": items,
        "encrypted": true,
        "encType": 2
    })
}

pub fn export_bitwarden_encrypted(
    entries: &[Entry],
    path: &Path,
    password: &str,
) -> Result<(), CoalboxError> {
    let mut rng = rand::thread_rng();

    // Generate salt and IV
    let mut salt = [0u8; 32];
    let mut iv = [0u8; 16];
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut iv);

    // Derive key
    let enc_key = derive_bitwarden_key(password, &salt, 100_000);

    // Generate JSON
    let json = generate_bitwarden_json(entries);
    let mut payload = serde_json::to_vec(&json)
        .map_err(|e| CoalboxError::Import(format!("JSON serialization failed: {}", e)))?;

    // Pad to AES block size
    let padding_len = 16 - (payload.len() % 16);
    payload.resize(payload.len() + padding_len, padding_len as u8);

    // Encrypt with AES-256-CBC
    let cipher = cbc::Encryptor::<aes::Aes256>::new(enc_key.as_slice().into(), &iv.into());
    let mut encrypted = payload.clone();
    let _ = cipher.encrypt_padded_mut::<NoPadding>(&mut encrypted, payload.len());

    // Compute MAC
    let mut mac_input = Vec::new();
    mac_input.extend_from_slice(&iv);
    mac_input.extend_from_slice(&encrypted);
    mac_input.extend_from_slice(&iv);

    let mut hasher = Sha256::new();
    hasher.update(&mac_input);
    let mac = hasher.finalize();

    // Build Bitwarden encrypted export format
    let export = serde_json::json!({
        "encryptedData": base64_encode(&encrypted),
        "iv": base64_encode(&iv),
        "mac": base64_encode(&mac),
        "encType": 2
    });

    let mut file = File::create(path)
        .map_err(|e| CoalboxError::Import(format!("Failed to create file: {}", e)))?;

    let json_str = serde_json::to_string_pretty(&export)
        .map_err(|e| CoalboxError::Import(format!("JSON serialization failed: {}", e)))?;

    file.write_all(json_str.as_bytes())
        .map_err(|e| CoalboxError::Import(format!("Failed to write file: {}", e)))?;

    Ok(())
}

fn base64_encode(data: &[u8]) -> String {
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
        .chars()
        .collect();
    let mut output = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        output.push(chars[((triple >> 18) & 0x3F) as usize]);
        output.push(chars[((triple >> 12) & 0x3F) as usize]);
        if chunk.len() > 1 {
            output.push(chars[((triple >> 6) & 0x3F) as usize]);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(chars[(triple & 0x3F) as usize]);
        } else {
            output.push('=');
        }
    }
    output
}
