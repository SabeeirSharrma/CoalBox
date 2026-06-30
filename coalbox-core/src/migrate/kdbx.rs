use std::fs::File;
use std::io::Write;
use std::path::Path;

use aes::cipher::block_padding::NoPadding;
use aes::cipher::{BlockEncryptMut, KeyIvInit};
use argon2::Argon2;
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::entry::Entry;
use crate::error::CoalboxError;

// KDBX4 magic bytes and version
const KDBX_MAGIC: &[u8; 4] = b"KDBX";

// Cipher IDs
const CIPHER_AES: [u8; 16] = [
    0x31, 0xC1, 0xF2, 0xE6, 0xBF, 0x71, 0x43, 0x50,
    0xBE, 0x58, 0x05, 0x21, 0x6A, 0xFC, 0xFA, 0x24,
];

// KDF IDs
const KDF_ARGON2: [u8; 16] = [
    0xEF, 0x63, 0x6D, 0x91, 0x70, 0x9D, 0x16, 0x93,
    0x9B, 0x55, 0x15, 0x32, 0x44, 0xBC, 0x91, 0x2E,
];

// Header end marker
const HEADER_END: u32 = 0x0D0AD105;

fn derive_key_argon2(password: &str, salt: &[u8], memory_kb: u32, iterations: u32, parallelism: u32) -> [u8; 32] {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(memory_kb, iterations, parallelism, Some(32)).unwrap(),
    );

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("Argon2 key derivation failed");
    key
}

fn generate_xml(entries: &[Entry]) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<Database>\n");
    xml.push_str("<Meta>\n");
    xml.push_str("  <DatabaseName>Coalbox</DatabaseName>\n");
    xml.push_str("  <DatabaseNameChanged>1</DatabaseNameChanged>\n");
    xml.push_str("</Meta>\n");
    xml.push_str("<Root>\n");
    xml.push_str("<Group>\n");
    xml.push_str("  <Name>Coalbox</Name>\n");

    for entry in entries {
        xml.push_str("  <Entry>\n");
        xml.push_str("    <UUID>");
        xml.push_str(&entry.id.to_string());
        xml.push_str("</UUID>\n");
        xml.push_str("    <Group>Default</Group>\n");

        match entry.entry_type {
            crate::entry::EntryType::Login => {
                xml.push_str("    <Title>");
                xml.push_str(&xml_escape(&entry.title));
                xml.push_str("</Title>\n");

                if let Some(ref username) = entry.username {
                    xml.push_str("    <UserName>");
                    xml.push_str(&xml_escape(username));
                    xml.push_str("</UserName>\n");
                }

                if let Some(ref password) = entry.password {
                    xml.push_str("    <Password>");
                    xml.push_str(&xml_escape(password));
                    xml.push_str("</Password>\n");
                }

                if let Some(ref url) = entry.url {
                    xml.push_str("    <URL>");
                    xml.push_str(&xml_escape(url));
                    xml.push_str("</URL>\n");
                }
            }
            crate::entry::EntryType::Note => {
                xml.push_str("    <Title>");
                xml.push_str(&xml_escape(&entry.title));
                xml.push_str("</Title>\n");
                xml.push_str("    <Password />\n");

                if let Some(ref notes) = entry.notes {
                    xml.push_str("    <Notes>");
                    xml.push_str(&xml_escape(notes));
                    xml.push_str("</Notes>\n");
                }
            }
            _ => {
                xml.push_str("    <Title>");
                xml.push_str(&xml_escape(&entry.title));
                xml.push_str("</Title>\n");
                xml.push_str("    <Password />\n");
            }
        }

        xml.push_str("  </Entry>\n");
    }

    xml.push_str("</Group>\n");
    xml.push_str("</Root>\n");
    xml.push_str("</Database>\n");
    xml
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn export_kdbx(entries: &[Entry], path: &Path, password: &str) -> Result<(), CoalboxError> {
    let mut rng = rand::thread_rng();

    // Generate random seeds
    let mut master_seed = [0u8; 32];
    let mut argon2_salt = [0u8; 32];
    let mut iv = [0u8; 16];

    rng.fill_bytes(&mut master_seed);
    rng.fill_bytes(&mut argon2_salt);
    rng.fill_bytes(&mut iv);

    // Derive key
    let composite_key = derive_composite_key(password, &master_seed);
    let transformed_key = derive_key_argon2(
        &composite_key,
        &argon2_salt,
        64 * 1024, // 64MB
        3,
        4,
    );

    // Generate and encrypt payload
    let xml = generate_xml(entries);
    let mut payload = xml.into_bytes();

    // Pad to AES block size (16 bytes)
    let padding_len = 16 - (payload.len() % 16);
    payload.resize(payload.len() + padding_len, padding_len as u8);

    // Encrypt with AES-256-CBC
    let cipher = cbc::Encryptor::<aes::Aes256>::new(&transformed_key.into(), &iv.into());
    let mut encrypted = payload.clone();
    let _ = cipher.encrypt_padded_mut::<NoPadding>(&mut encrypted, payload.len());

    // Build header
    let mut header = Vec::new();

    // Magic bytes
    header.extend_from_slice(KDBX_MAGIC);

    // Version (minor, major) = 0, 4 for KDBX 4.0
    header.extend_from_slice(&0u16.to_le_bytes());
    header.extend_from_slice(&4u16.to_le_bytes());

    // Header size (placeholder, will be updated)
    let header_size_pos = header.len();
    header.extend_from_slice(&0u32.to_le_bytes());

    // Cipher ID
    header.push(0x03); // CipherID
    header.extend_from_slice(&(16u32).to_le_bytes());
    header.extend_from_slice(&CIPHER_AES);

    // Compression flags (0 = none)
    header.push(0x04); // CompressionFlags
    header.extend_from_slice(&(4u32).to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());

    // Master seed
    header.push(0x02); // MasterSeed
    header.extend_from_slice(&(32u32).to_le_bytes());
    header.extend_from_slice(&master_seed);

    // Key derivation parameters
    header.push(0x05); // KdfParameters
    let kdf_params = build_kdf_params(&argon2_salt);
    header.extend_from_slice(&(kdf_params.len() as u32).to_le_bytes());
    header.extend_from_slice(&kdf_params);

    // Header end marker
    header.extend_from_slice(&HEADER_END.to_le_bytes());

    // Update header size
    let header_size = header.len() as u32;
    header[header_size_pos..header_size_pos + 4].copy_from_slice(&header_size.to_le_bytes());

    // Write file
    let mut file = File::create(path)
        .map_err(|e| CoalboxError::Import(format!("Failed to create file: {}", e)))?;

    file.write_all(&header)
        .map_err(|e| CoalboxError::Import(format!("Failed to write header: {}", e)))?;

    file.write_all(&encrypted)
        .map_err(|e| CoalboxError::Import(format!("Failed to write encrypted data: {}", e)))?;

    Ok(())
}

fn derive_composite_key(password: &str, master_seed: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let password_hash = hasher.finalize();

    let mut combined = Vec::new();
    combined.extend_from_slice(&password_hash);
    combined.extend_from_slice(master_seed);

    let mut hasher = Sha256::new();
    hasher.update(&combined);
    hex::encode(hasher.finalize())
}

fn build_kdf_params(salt: &[u8]) -> Vec<u8> {
    let mut params = Vec::new();

    // KDF UUID
    params.extend_from_slice(&KDF_ARGON2);

    // Salt
    params.push(0x01);
    params.extend_from_slice(&(salt.len() as u32).to_le_bytes());
    params.extend_from_slice(salt);

    // Memory (64MB = 65536 KB)
    params.push(0x02);
    params.extend_from_slice(&4u32.to_le_bytes());
    params.extend_from_slice(&(65536u32).to_le_bytes());

    // Iterations
    params.push(0x03);
    params.extend_from_slice(&4u32.to_le_bytes());
    params.extend_from_slice(&3u32.to_le_bytes());

    // Parallelism
    params.push(0x04);
    params.extend_from_slice(&2u32.to_le_bytes());
    params.extend_from_slice(&4u16.to_le_bytes());

    // Version
    params.push(0x05);
    params.extend_from_slice(&4u32.to_le_bytes());
    params.extend_from_slice(&0x13u32.to_le_bytes());

    // Secret (empty for now)
    params.push(0x06);
    params.extend_from_slice(&0u32.to_le_bytes());

    // Memory (again for some readers)
    params.push(0x07);
    params.extend_from_slice(&4u32.to_le_bytes());
    params.extend_from_slice(&32u32.to_le_bytes());

    params
}

mod hex {
    pub fn encode(data: impl AsRef<[u8]>) -> String {
        data.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}
