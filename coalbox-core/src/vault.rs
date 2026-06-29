use std::fs;
use std::path::{Path, PathBuf};

use crate::crypto;
use crate::entry::{Entry, EntryId};
use crate::error::CoalboxError;
use crate::format::{VaultHeader, TAG_LEN};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VaultData {
    pub version: u32,
    pub created: String,
    pub modified: String,
    pub entries: Vec<Entry>,
}

impl Default for VaultData {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultData {
    pub fn new() -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            version: 1,
            created: now.clone(),
            modified: now,
            entries: Vec::new(),
        }
    }
}

pub struct Vault {
    path: PathBuf,
    header: Option<VaultHeader>,
    data: Option<VaultData>,
}

impl Vault {
    pub fn create(path: &Path, master_password: &str) -> Result<Self, CoalboxError> {
        let salt = crypto::generate_salt();
        let nonce = crypto::generate_nonce();
        let key = crypto::derive_key(master_password, &salt)?;

        let vault_data = VaultData::new();
        let plaintext = serde_json::to_vec(&vault_data)?;

        let (encrypted, tag) = crypto::encrypt(&plaintext, &key, &nonce)?;

        let header = VaultHeader::new(salt, nonce, encrypted.len() as u32);

        let mut file_data = Vec::new();
        file_data.extend_from_slice(&header.to_bytes());
        file_data.extend_from_slice(&encrypted);
        file_data.extend_from_slice(&tag);

        fs::write(path, file_data)?;

        Ok(Self {
            path: path.to_path_buf(),
            header: Some(header),
            data: Some(vault_data),
        })
    }

    pub fn unlock(path: &Path, master_password: &str) -> Result<Self, CoalboxError> {
        if !path.exists() {
            return Err(CoalboxError::VaultNotFound(path.display().to_string()));
        }

        let file_data = fs::read(path)?;
        let header = VaultHeader::from_bytes(&file_data)?;

        let header_size = VaultHeader::HEADER_SIZE;
        let expected_size = header_size + header.payload_len as usize + TAG_LEN;

        if file_data.len() < expected_size {
            return Err(CoalboxError::InvalidFormat(
                "file size doesn't match header".into(),
            ));
        }

        let key = crypto::derive_key(master_password, &header.salt)?;

        let encrypted = &file_data[header_size..header_size + header.payload_len as usize];
        let tag = &file_data[header_size + header.payload_len as usize..expected_size];

        let mut tag_bytes = [0u8; TAG_LEN];
        tag_bytes.copy_from_slice(tag);

        let plaintext = crypto::decrypt(encrypted, &key, &header.nonce, &tag_bytes)?;

        let vault_data: VaultData = serde_json::from_slice(&plaintext)?;

        Ok(Self {
            path: path.to_path_buf(),
            header: Some(header),
            data: Some(vault_data),
        })
    }

    pub fn lock(&mut self) {
        if let Some(ref mut data) = self.data {
            let mut json = serde_json::to_vec(data).unwrap_or_default();
            crypto::secure_zero(&mut json);
        }
        self.data = None;
        self.header = None;
    }

    pub fn is_unlocked(&self) -> bool {
        self.data.is_some()
    }

    pub fn save(&mut self, master_password: &str) -> Result<(), CoalboxError> {
        let data = self
            .data
            .as_mut()
            .ok_or(CoalboxError::Locked)?;

        let now = chrono::Utc::now().to_rfc3339();
        data.modified = now;

        let salt = crypto::generate_salt();
        let nonce = crypto::generate_nonce();
        let key = crypto::derive_key(master_password, &salt)?;

        let plaintext = serde_json::to_vec(&data)?;
        let (encrypted, tag) = crypto::encrypt(&plaintext, &key, &nonce)?;

        let header = VaultHeader::new(salt, nonce, encrypted.len() as u32);

        let mut file_data = Vec::new();
        file_data.extend_from_slice(&header.to_bytes());
        file_data.extend_from_slice(&encrypted);
        file_data.extend_from_slice(&tag);

        fs::write(&self.path, file_data)?;

        self.header = Some(header);

        Ok(())
    }

    pub fn data(&self) -> Result<&VaultData, CoalboxError> {
        self.data.as_ref().ok_or(CoalboxError::Locked)
    }

    pub fn data_mut(&mut self) -> Result<&mut VaultData, CoalboxError> {
        self.data.as_mut().ok_or(CoalboxError::Locked)
    }

    pub fn add_entry(&mut self, entry: Entry) -> Result<(), CoalboxError> {
        let data = self.data_mut()?;
        data.entries.push(entry);
        Ok(())
    }

    pub fn get_entry(&self, id: &EntryId) -> Result<Entry, CoalboxError> {
        let data = self.data()?;
        data.entries
            .iter()
            .find(|e| e.id == *id)
            .cloned()
            .ok_or_else(|| CoalboxError::EntryNotFound(id.to_string()))
    }

    pub fn get_entry_by_title(&self, title: &str) -> Result<Entry, CoalboxError> {
        let data = self.data()?;
        data.entries
            .iter()
            .find(|e| e.title.eq_ignore_ascii_case(title))
            .cloned()
            .ok_or_else(|| CoalboxError::EntryNotFound(title.to_string()))
    }

    pub fn get_entry_by_url(&self, url: &str) -> Result<Entry, CoalboxError> {
        let data = self.data()?;
        data.entries
            .iter()
            .find(|e| {
                e.url
                    .as_ref()
                    .map(|u| u.contains(url))
                    .unwrap_or(false)
            })
            .cloned()
            .ok_or_else(|| CoalboxError::EntryNotFound(url.to_string()))
    }

    pub fn search(&self, query: &str) -> Vec<Entry> {
        let data = match self.data() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        let q = query.to_lowercase();
        data.entries
            .iter()
            .filter(|e| {
                e.title.to_lowercase().contains(&q)
                    || e.url.as_ref().map(|u| u.to_lowercase().contains(&q)).unwrap_or(false)
                    || e.username.as_ref().map(|u| u.to_lowercase().contains(&q)).unwrap_or(false)
                    || e.notes.as_ref().map(|n| n.to_lowercase().contains(&q)).unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn update_entry(&mut self, id: &EntryId, update_fn: impl FnOnce(&mut Entry)) -> Result<(), CoalboxError> {
        let data = self.data_mut()?;
        let entry = data
            .entries
            .iter_mut()
            .find(|e| e.id == *id)
            .ok_or_else(|| CoalboxError::EntryNotFound(id.to_string()))?;

        update_fn(entry);
        entry.modified = chrono::Utc::now();
        Ok(())
    }

    pub fn delete_entry(&mut self, id: &EntryId) -> Result<Entry, CoalboxError> {
        let data = self.data_mut()?;
        let idx = data
            .entries
            .iter()
            .position(|e| e.id == *id)
            .ok_or_else(|| CoalboxError::EntryNotFound(id.to_string()))?;

        Ok(data.entries.remove(idx))
    }

    pub fn list_entries(&self) -> Result<Vec<Entry>, CoalboxError> {
        let data = self.data()?;
        Ok(data.entries.clone())
    }

    pub fn entry_count(&self) -> usize {
        self.data
            .as_ref()
            .map(|d| d.entries.len())
            .unwrap_or(0)
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        self.lock();
    }
}
