use crate::error::CoalboxError;

pub const MAGIC: [u8; 4] = [0x45, 0x4D, 0x42, 0x4B]; // "EMBK"
pub const FORMAT_VERSION: u16 = 0x0001;
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;

pub struct VaultHeader {
    pub format_version: u16,
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
    pub payload_len: u32,
}

impl VaultHeader {
    pub const HEADER_SIZE: usize = 4 + 2 + SALT_LEN + NONCE_LEN + 4; // magic + version + salt + nonce + len

    pub fn new(salt: [u8; SALT_LEN], nonce: [u8; NONCE_LEN], payload_len: u32) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            salt,
            nonce,
            payload_len,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(Self::HEADER_SIZE);
        buf.extend_from_slice(&MAGIC);
        buf.extend_from_slice(&self.format_version.to_le_bytes());
        buf.extend_from_slice(&self.salt);
        buf.extend_from_slice(&self.nonce);
        buf.extend_from_slice(&self.payload_len.to_le_bytes());
        buf
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, CoalboxError> {
        if data.len() < Self::HEADER_SIZE {
            return Err(CoalboxError::InvalidFormat("file too short".into()));
        }

        if data[0..4] != MAGIC {
            return Err(CoalboxError::InvalidMagic);
        }

        let version = u16::from_le_bytes([data[4], data[5]]);
        if version != FORMAT_VERSION {
            return Err(CoalboxError::UnsupportedVersion(version));
        }

        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&data[6..22]);

        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&data[22..34]);

        let payload_len = u32::from_le_bytes([data[34], data[35], data[36], data[37]]);

        Ok(Self {
            format_version: version,
            salt,
            nonce,
            payload_len,
        })
    }
}
