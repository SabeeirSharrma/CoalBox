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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    Hidden,
    Url,
    Date,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub name: String,
    pub field_type: FieldType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordHistoryEntry {
    pub password: String,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cardholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cvv: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_fields: Vec<CustomField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<CardData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityData>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub password_history: Vec<PasswordHistoryEntry>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
}

impl Entry {
    fn base(id: EntryId, entry_type: EntryType, title: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            entry_type,
            title,
            url: None,
            username: None,
            password: None,
            totp_secret: None,
            notes: None,
            tags: Vec::new(),
            favourite: false,
            custom_fields: Vec::new(),
            card: None,
            identity: None,
            password_history: Vec::new(),
            created: now,
            modified: now,
        }
    }

    pub fn new_login(title: String, username: String, password: String) -> Self {
        let mut entry = Self::base(Uuid::new_v4(), EntryType::Login, title);
        entry.username = Some(username);
        entry.password = Some(password);
        entry
    }

    pub fn new_note(title: String, notes: String) -> Self {
        let mut entry = Self::base(Uuid::new_v4(), EntryType::Note, title);
        entry.notes = Some(notes);
        entry
    }

    pub fn new_card(title: String, card: CardData) -> Self {
        let mut entry = Self::base(Uuid::new_v4(), EntryType::Card, title);
        entry.card = Some(card);
        entry
    }

    pub fn new_identity(title: String, identity: IdentityData) -> Self {
        let mut entry = Self::base(Uuid::new_v4(), EntryType::Identity, title);
        entry.identity = Some(identity);
        entry
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_favourite(mut self, favourite: bool) -> Self {
        self.favourite = favourite;
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    pub fn with_custom_fields(mut self, fields: Vec<CustomField>) -> Self {
        self.custom_fields = fields;
        self
    }

    pub fn with_totp(mut self, totp_secret: String) -> Self {
        self.totp_secret = Some(totp_secret);
        self
    }

    pub fn update_password(&mut self, new_password: String) {
        if let Some(ref old_password) = self.password {
            self.password_history.push(PasswordHistoryEntry {
                password: old_password.clone(),
                changed_at: Utc::now(),
            });
        }
        self.password = Some(new_password);
        self.modified = Utc::now();
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    pub fn add_custom_field(&mut self, field: CustomField) {
        self.custom_fields.push(field);
    }

    pub fn remove_custom_field(&mut self, name: &str) {
        self.custom_fields.retain(|f| f.name != name);
    }

    pub fn get_custom_field(&self, name: &str) -> Option<&CustomField> {
        self.custom_fields.iter().find(|f| f.name == name)
    }

    pub fn display_name(&self) -> String {
        match self.entry_type {
            EntryType::Card => {
                if let Some(ref card) = self.card
                    && let Some(ref number) = card.number
                {
                    let masked = if number.len() > 4 {
                        format!("•••• {}", &number[number.len() - 4..])
                    } else {
                        number.clone()
                    };
                    return format!("{} ({})", self.title, masked);
                }
                self.title.clone()
            }
            EntryType::Identity => {
                if let Some(ref identity) = self.identity {
                    let parts: Vec<&str> = [
                        identity.first_name.as_deref(),
                        identity.last_name.as_deref(),
                    ]
                    .into_iter()
                    .flatten()
                    .collect();
                    if !parts.is_empty() {
                        return format!("{} ({})", self.title, parts.join(" "));
                    }
                }
                self.title.clone()
            }
            _ => self.title.clone(),
        }
    }
}
