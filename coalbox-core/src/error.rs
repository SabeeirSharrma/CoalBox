use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoalboxError {
    #[error("Vault file not found: {0}")]
    VaultNotFound(String),

    #[error("Invalid vault magic bytes (expected EMBK)")]
    InvalidMagic,

    #[error("Unsupported vault format version: {0}")]
    UnsupportedVersion(u16),

    #[error("Invalid vault file: {0}")]
    InvalidFormat(String),

    #[error("Decryption failed — wrong master password or corrupted vault")]
    DecryptionFailed,

    #[error("Vault is locked — unlock first")]
    Locked,

    #[error("Vault is already unlocked")]
    AlreadyUnlocked,

    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Encryption error: {0}")]
    Crypto(String),

    #[error("Argon2 error: {0}")]
    Argon2(String),
}
