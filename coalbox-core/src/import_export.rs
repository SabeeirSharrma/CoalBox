use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::entry::Entry;
use crate::error::CoalboxError;

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub entries: Vec<Entry>,
    pub skipped: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportFormat {
    Csv,
    BitwardenJson,
    KeePassXml,
    OnePassword1Pux,
    Auto,
}

impl ImportFormat {
    pub fn detect(path: &Path) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "csv" => Self::Csv,
            "json" => Self::BitwardenJson,
            "xml" | "kdbx" => Self::KeePassXml,
            "1pux" => Self::OnePassword1Pux,
            _ => Self::Auto,
        }
    }
}

pub fn import_file(path: &Path, format: ImportFormat) -> Result<ImportResult, CoalboxError> {
    match format {
        ImportFormat::OnePassword1Pux => import_1password_1pux(path),
        _ => {
            let data = std::fs::read_to_string(path)?;
            match format {
                ImportFormat::Csv => import_csv(&data),
                ImportFormat::BitwardenJson => import_bitwarden_json(&data),
                ImportFormat::KeePassXml => import_keepass_xml(&data),
                ImportFormat::Auto => {
                    let fmt = ImportFormat::detect(path);
                    import_file(path, fmt)
                }
                ImportFormat::OnePassword1Pux => unreachable!(),
            }
        }
    }
}

fn import_csv(data: &str) -> Result<ImportResult, CoalboxError> {
    let mut reader = csv::Reader::from_reader(data.as_bytes());
    let mut entries = Vec::new();
    let mut skipped = 0;
    let mut errors = Vec::new();

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| CoalboxError::Import(e.to_string()))?
        .iter()
        .map(|h| h.to_lowercase())
        .collect();

    for result in reader.records() {
        match result {
            Ok(record) => {
                let row: HashMap<&str, &str> = headers
                    .iter()
                    .zip(record.iter())
                    .map(|(h, v)| (h.as_str(), v))
                    .collect();

                let title = get_csv_field(&row, &["name", "title", "entry", "label"]);
                let username = get_csv_field(&row, &["username", "user", "login", "email"]);
                let password = get_csv_field(&row, &["password", "pass", "pwd"]);
                let url = get_csv_field(&row, &["url", "website", "site", "link"]);
                let notes = get_csv_field(&row, &["notes", "note", "comment"]);

                if title.is_none() && username.is_none() && password.is_none() {
                    skipped += 1;
                    continue;
                }

                let mut entry = Entry::new_login(
                    title.unwrap_or_default(),
                    username.unwrap_or_default(),
                    password.unwrap_or_default(),
                );

                if let Some(u) = url {
                    entry = entry.with_url(u);
                }
                if let Some(n) = notes {
                    entry = entry.with_notes(n);
                }

                entries.push(entry);
            }
            Err(e) => {
                errors.push(format!("CSV row error: {}", e));
            }
        }
    }

    Ok(ImportResult {
        entries,
        skipped,
        errors,
    })
}

fn get_csv_field(row: &HashMap<&str, &str>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(val) = row.get(*key) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn import_bitwarden_json(data: &str) -> Result<ImportResult, CoalboxError> {
    let parsed: serde_json::Value =
        serde_json::from_str(data).map_err(CoalboxError::Json)?;

    let items = parsed
        .get("items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| CoalboxError::Import("Invalid Bitwarden JSON: missing 'items'".into()))?;

    let mut entries = Vec::new();
    let errors = Vec::new();

    for item in items {
        let item_type = item.get("type").and_then(|v| v.as_i64()).unwrap_or(1);

        if item_type != 1 {
            continue;
        }

        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let login = item.get("login");
        let username = login
            .and_then(|l| l.get("username"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let password = login
            .and_then(|l| l.get("password"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let uris = login
            .and_then(|l| l.get("uris"))
            .and_then(|v| v.as_array());
        let url = uris
            .and_then(|u| u.first())
            .and_then(|u| u.get("uri"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let notes = item
            .get("notes")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut entry = Entry::new_login(name, username, password);

        if let Some(u) = url {
            entry = entry.with_url(u);
        }
        if let Some(n) = notes {
            entry = entry.with_notes(n);
        }

        let tags = item.get("fields").and_then(|v| v.as_array());
        if let Some(tags_arr) = tags {
            let tag_names: Vec<String> = tags_arr
                .iter()
                .filter_map(|t| t.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .collect();
            if !tag_names.is_empty() {
                entry = entry.with_tags(tag_names);
            }
        }

        entries.push(entry);
    }

    Ok(ImportResult {
        entries,
        skipped: 0,
        errors,
    })
}

fn import_keepass_xml(data: &str) -> Result<ImportResult, CoalboxError> {
    let mut reader = quick_xml::Reader::from_str(data);
    let mut entries = Vec::new();
    let mut errors = Vec::new();

    let mut buf = Vec::new();
    let mut in_entry = false;
    let mut in_key = false;
    let mut in_value = false;
    let mut title = String::new();
    let mut username = String::new();
    let mut password = String::new();
    let mut url = String::new();
    let mut notes = String::new();
    let mut current_field = String::new();
    let mut collecting_value = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"Entry" if !in_entry => {
                        in_entry = true;
                        title.clear();
                        username.clear();
                        password.clear();
                        url.clear();
                        notes.clear();
                        current_field.clear();
                    }
                    b"Key" if in_entry => {
                        in_key = true;
                        current_field.clear();
                    }
                    b"Value" if in_entry && in_key => {
                        in_value = true;
                    }
                    b"Value" if in_entry && !in_key && !collecting_value => {
                        collecting_value = true;
                    }
                    _ => {}
                }
            }
            Ok(quick_xml::events::Event::Text(ref e)) => {
                if in_entry {
                    let text = e
                        .unescape()
                        .map_err(|e| CoalboxError::Import(e.to_string()))?
                        .to_string();
                    if in_key && in_value && current_field.is_empty() {
                        current_field = text;
                    } else if collecting_value {
                        match current_field.as_str() {
                            "Title" => title = text,
                            "UserName" => username = text,
                            "Password" => password = text,
                            "URL" => url = text,
                            "Notes" => notes = text,
                            _ => {}
                        }
                    }
                }
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"Value" if in_entry && in_key => {
                        in_value = false;
                    }
                    b"Key" if in_entry => {
                        in_key = false;
                    }
                    b"Value" if in_entry && collecting_value => {
                        collecting_value = false;
                    }
                    b"Entry" => {
                        in_entry = false;
                        in_key = false;
                        in_value = false;
                        collecting_value = false;
                        if !title.is_empty() || !username.is_empty() {
                            let mut entry =
                                Entry::new_login(title.clone(), username.clone(), password.clone());
                            if !url.is_empty() {
                                entry = entry.with_url(url.clone());
                            }
                            if !notes.is_empty() {
                                entry = entry.with_notes(notes.clone());
                            }
                            entries.push(entry);
                        }
                    }
                    _ => {}
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => {
                errors.push(format!("XML parse error: {}", e));
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(ImportResult {
        entries,
        skipped: 0,
        errors,
    })
}

fn import_1password_1pux(path: &Path) -> Result<ImportResult, CoalboxError> {
    let file = std::fs::File::open(path)
        .map_err(|e| CoalboxError::Import(format!("Failed to open 1PUX file: {}", e)))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| CoalboxError::Import(format!("Failed to read 1PUX archive: {}", e)))?;

    let mut json_content = String::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| CoalboxError::Import(e.to_string()))?;

        if file.name().ends_with("export.json") {
            file.read_to_string(&mut json_content)
                .map_err(|e| CoalboxError::Import(e.to_string()))?;
            break;
        }
    }

    if json_content.is_empty() {
        return Err(CoalboxError::Import(
            "No export.json found in 1PUX archive".into(),
        ));
    }

    let parsed: serde_json::Value =
        serde_json::from_str(&json_content).map_err(CoalboxError::Json)?;

    let items = parsed
        .get("items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| CoalboxError::Import("Invalid 1PUX format: missing 'items'".into()))?;

    let mut entries = Vec::new();

    for item in items {
        let overview = item.get("overview");
        let title = overview
            .and_then(|o| o.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let urls = overview
            .and_then(|o| o.get("urls"))
            .and_then(|v| v.as_array());
        let url = urls
            .and_then(|u| u.first())
            .and_then(|u| u.get("url"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let details = item.get("details");
        let login = details.and_then(|d| d.get("loginFields")).and_then(|v| v.as_array());

        let username = login
            .and_then(|l| {
                l.iter().find(|f| {
                    f.get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s == "username")
                        .unwrap_or(false)
                })
            })
            .and_then(|f| f.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let password = login
            .and_then(|l| {
                l.iter().find(|f| {
                    f.get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s == "password")
                        .unwrap_or(false)
                })
            })
            .and_then(|f| f.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let notes = details
            .and_then(|d| d.get("notesPlain"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut entry = Entry::new_login(title, username, password);
        if let Some(u) = url {
            entry = entry.with_url(u);
        }
        if let Some(n) = notes {
            entry = entry.with_notes(n);
        }

        entries.push(entry);
    }

    Ok(ImportResult {
        entries,
        skipped: 0,
        errors: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_csv() {
        let csv_data = "name,username,password,url,notes\nGitHub,user@example.com,pass123,https://github.com,My GitHub\nGitLab,dev@example.com,secret456,https://gitlab.com,\n";

        let result = import_csv(csv_data).unwrap();
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].title, "GitHub");
        assert_eq!(
            result.entries[0].username.as_deref(),
            Some("user@example.com")
        );
        assert_eq!(result.entries[0].password.as_deref(), Some("pass123"));
        assert_eq!(
            result.entries[0].url.as_deref(),
            Some("https://github.com")
        );
    }

    #[test]
    fn test_import_csv_variants() {
        let csv_data = "title,user,pwd,website\nNetflix,user,pass,https://netflix.com\n";

        let result = import_csv(csv_data).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].title, "Netflix");
        assert_eq!(
            result.entries[0].username.as_deref(),
            Some("user")
        );
    }

    #[test]
    fn test_import_csv_skips_empty_rows() {
        let csv_data = "name,username,password\nGitHub,user,pass\n\nGitLab,user2,pass2\n";

        let result = import_csv(csv_data).unwrap();
        assert_eq!(result.entries.len(), 2);
    }

    #[test]
    fn test_import_bitwarden_json() {
        let json = r#"{
            "items": [
                {
                    "type": 1,
                    "name": "GitHub",
                    "login": {
                        "username": "user@example.com",
                        "password": "pass123",
                        "uris": [{"uri": "https://github.com"}]
                    },
                    "notes": "My notes"
                }
            ]
        }"#;

        let result = import_bitwarden_json(json).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].title, "GitHub");
        assert_eq!(result.entries[0].notes.as_deref(), Some("My notes"));
    }

    #[test]
    fn test_import_bitwarden_json_skips_non_login() {
        let json = r#"{
            "items": [
                {
                    "type": 1,
                    "name": "Login",
                    "login": {"username": "u", "password": "p"}
                },
                {
                    "type": 2,
                    "name": "Secure Note"
                }
            ]
        }"#;

        let result = import_bitwarden_json(json).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].title, "Login");
    }

    #[test]
    fn test_import_keepass_xml() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Database>
<Group>
<Entry>
<Key><Value>Title</Value></Key>
<Value><Value>GitHub</Value></Value>
</Entry>
<Entry>
<Key><Value>UserName</Value></Key>
<Value><Value>user@example.com</Value></Value>
</Entry>
<Entry>
<Key><Value>Password</Value></Key>
<Value><Value>pass123</Value></Value>
</Entry>
</Group>
</Database>"#;

        let result = import_keepass_xml(xml).unwrap();
        assert!(!result.entries.is_empty());
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(
            ImportFormat::detect(Path::new("vault.csv")),
            ImportFormat::Csv
        );
        assert_eq!(
            ImportFormat::detect(Path::new("export.json")),
            ImportFormat::BitwardenJson
        );
        assert_eq!(
            ImportFormat::detect(Path::new("database.xml")),
            ImportFormat::KeePassXml
        );
        assert_eq!(
            ImportFormat::detect(Path::new("export.1pux")),
            ImportFormat::OnePassword1Pux
        );
    }
}
