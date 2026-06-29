use coalbox_core::entry::{Entry, EntryType};
use coalbox_core::format::VaultHeader;
use coalbox_core::Vault;
use tempfile::tempdir;

fn temp_vault_path(dir: &tempfile::TempDir) -> std::path::PathBuf {
    dir.path().join("test.emberkeys")
}

#[test]
fn test_vault_create_and_unlock() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);

    let vault = Vault::create(&path, "test_password_123").unwrap();
    assert!(vault.is_unlocked());
    assert_eq!(vault.entry_count(), 0);
    assert!(path.exists());

    drop(vault);

    let vault = Vault::unlock(&path, "test_password_123").unwrap();
    assert!(vault.is_unlocked());
    assert_eq!(vault.entry_count(), 0);
}

#[test]
fn test_wrong_password_fails() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);

    Vault::create(&path, "correct_password").unwrap();

    let result = Vault::unlock(&path, "wrong_password");
    assert!(result.is_err());
}

#[test]
fn test_vault_lock() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);

    let mut vault = Vault::create(&path, "test_password").unwrap();
    assert!(vault.is_unlocked());

    vault.lock();
    assert!(!vault.is_unlocked());
}

#[test]
fn test_add_and_get_entry() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);
    let password = "test_password";

    let mut vault = Vault::create(&path, password).unwrap();

    let entry = Entry::new_login(
        "Example".to_string(),
        "user@example.com".to_string(),
        "secret123".to_string(),
    );
    let entry_id = entry.id;

    vault.add_entry(entry).unwrap();
    vault.save(password).unwrap();

    drop(vault);

    let vault = Vault::unlock(&path, password).unwrap();
    let retrieved = vault.get_entry(&entry_id).unwrap();
    assert_eq!(retrieved.title, "Example");
    assert_eq!(retrieved.username.unwrap(), "user@example.com");
    assert_eq!(retrieved.password.unwrap(), "secret123");
    assert_eq!(retrieved.entry_type, EntryType::Login);
}

#[test]
fn test_add_note_entry() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);
    let password = "test_password";

    let mut vault = Vault::create(&path, password).unwrap();

    let entry = Entry::new_note("My Note".to_string(), "This is a secret note".to_string());
    let entry_id = entry.id;

    vault.add_entry(entry).unwrap();
    vault.save(password).unwrap();

    drop(vault);

    let vault = Vault::unlock(&path, password).unwrap();
    let retrieved = vault.get_entry(&entry_id).unwrap();
    assert_eq!(retrieved.title, "My Note");
    assert_eq!(retrieved.notes.unwrap(), "This is a secret note");
    assert_eq!(retrieved.entry_type, EntryType::Note);
}

#[test]
fn test_delete_entry() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);
    let password = "test_password";

    let mut vault = Vault::create(&path, password).unwrap();

    let entry = Entry::new_login("Delete Me".to_string(), "user".to_string(), "pass".to_string());
    let entry_id = entry.id;

    vault.add_entry(entry).unwrap();
    assert_eq!(vault.entry_count(), 1);

    let deleted = vault.delete_entry(&entry_id).unwrap();
    assert_eq!(deleted.title, "Delete Me");
    assert_eq!(vault.entry_count(), 0);
}

#[test]
fn test_update_entry() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);
    let password = "test_password";

    let mut vault = Vault::create(&path, password).unwrap();

    let entry = Entry::new_login("Old Title".to_string(), "user".to_string(), "pass".to_string());
    let entry_id = entry.id;

    vault.add_entry(entry).unwrap();
    vault.update_entry(&entry_id, |e| {
        e.title = "New Title".to_string();
    }).unwrap();

    let updated = vault.get_entry(&entry_id).unwrap();
    assert_eq!(updated.title, "New Title");
}

#[test]
fn test_search_entries() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);
    let password = "test_password";

    let mut vault = Vault::create(&path, password).unwrap();

    vault.add_entry(Entry::new_login("GitHub".to_string(), "user@gh.com".to_string(), "pass".to_string())).unwrap();
    vault.add_entry(Entry::new_login("GitLab".to_string(), "user@glab.com".to_string(), "pass".to_string())).unwrap();
    vault.add_entry(Entry::new_login("Twitter".to_string(), "user@tw.com".to_string(), "pass".to_string())).unwrap();

    let results = vault.search("git");
    assert_eq!(results.len(), 2);

    let results = vault.search("github");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "GitHub");
}

#[test]
fn test_get_entry_by_title() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);
    let password = "test_password";

    let mut vault = Vault::create(&path, password).unwrap();

    vault.add_entry(Entry::new_login("GitHub".to_string(), "user@gh.com".to_string(), "pass".to_string())).unwrap();

    let entry = vault.get_entry_by_title("GitHub").unwrap();
    assert_eq!(entry.username.unwrap(), "user@gh.com");

    let entry = vault.get_entry_by_title("github").unwrap();
    assert_eq!(entry.username.unwrap(), "user@gh.com");
}

#[test]
fn test_header_serialization() {
    let salt = [1u8; 16];
    let nonce = [2u8; 12];
    let header = VaultHeader::new(salt, nonce, 1024);

    let bytes = header.to_bytes();
    assert_eq!(&bytes[0..4], b"EMBK");

    let parsed = VaultHeader::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.format_version, 1);
    assert_eq!(parsed.salt, salt);
    assert_eq!(parsed.nonce, nonce);
    assert_eq!(parsed.payload_len, 1024);
}

#[test]
fn test_invalid_magic_bytes() {
    let data = vec![0u8; VaultHeader::HEADER_SIZE];
    let result = VaultHeader::from_bytes(&data);
    assert!(result.is_err());
}

#[test]
fn test_vault_entry_count() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);
    let password = "test_password";

    let mut vault = Vault::create(&path, password).unwrap();
    assert_eq!(vault.entry_count(), 0);

    vault.add_entry(Entry::new_login("A".to_string(), "u".to_string(), "p".to_string())).unwrap();
    assert_eq!(vault.entry_count(), 1);

    vault.add_entry(Entry::new_note("B".to_string(), "n".to_string())).unwrap();
    assert_eq!(vault.entry_count(), 2);
}

#[test]
fn test_list_entries() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);
    let password = "test_password";

    let mut vault = Vault::create(&path, password).unwrap();

    vault.add_entry(Entry::new_login("A".to_string(), "u".to_string(), "p".to_string())).unwrap();
    vault.add_entry(Entry::new_login("B".to_string(), "u".to_string(), "p".to_string())).unwrap();

    let entries = vault.list_entries().unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_persistence_across_restarts() {
    let dir = tempdir().unwrap();
    let path = temp_vault_path(&dir);
    let password = "test_password";

    // Create vault and add entries
    {
        let mut vault = Vault::create(&path, password).unwrap();
        vault.add_entry(Entry::new_login("Persist".to_string(), "user".to_string(), "pass".to_string())).unwrap();
        vault.save(password).unwrap();
    }

    // Reopen and verify
    {
        let vault = Vault::unlock(&path, password).unwrap();
        assert_eq!(vault.entry_count(), 1);
        let entry = vault.get_entry_by_title("Persist").unwrap();
        assert_eq!(entry.username.unwrap(), "user");
    }
}

#[test]
fn test_vault_not_found() {
    let result = Vault::unlock(std::path::Path::new("/tmp/nonexistent_vault.emberkeys"), "pass");
    assert!(result.is_err());
}
