use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type EntryId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    Login,
    Note,
    Card,
    Identity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: EntryId,
    pub entry_type: EntryType,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default)]
    pub favourite: bool,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

impl Entry {
    pub fn new_login(title: String, username: String, password: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            entry_type: EntryType::Login,
            title,
            url: None,
            username: Some(username),
            password: Some(password),
            totp_secret: None,
            notes: None,
            tags: Vec::new(),
            favourite: false,
            created: now,
            modified: now,
        }
    }

    pub fn new_note(title: String, notes: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            entry_type: EntryType::Note,
            title,
            url: None,
            username: None,
            password: None,
            totp_secret: None,
            notes: Some(notes),
            tags: Vec::new(),
            favourite: false,
            created: now,
            modified: now,
        }
    }

    pub fn new_card(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            entry_type: EntryType::Card,
            title,
            url: None,
            username: None,
            password: None,
            totp_secret: None,
            notes: None,
            tags: Vec::new(),
            favourite: false,
            created: now,
            modified: now,
        }
    }

    pub fn new_identity(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            entry_type: EntryType::Identity,
            title,
            url: None,
            username: None,
            password: None,
            totp_secret: None,
            notes: None,
            tags: Vec::new(),
            favourite: false,
            created: now,
            modified: now,
        }
    }
}
