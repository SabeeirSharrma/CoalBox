use coalbox_core::{Entry, Vault};
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("./test_workspace/test.emberkeys");
    
    // Remove if exists
    let _ = std::fs::remove_file(&path);
    std::fs::create_dir_all("./test_workspace").ok();
    
    // Create vault
    let mut vault = Vault::create(&path, "testpass123").expect("Failed to create vault");
    
    // Login with TOTP
    let login1 = Entry::new_login(
        "GitHub".to_string(),
        "user@example.com".to_string(),
        "gh_s3cur3_p@ss!".to_string(),
    ).with_url("https://github.com".to_string())
     .with_tags(vec!["dev".to_string(), "work".to_string()])
     .with_favourite(true)
     .with_totp("JBSWY3DPEHPK3PXP".to_string());
    
    // Login without TOTP
    let login2 = Entry::new_login(
        "Gmail".to_string(),
        "user@gmail.com".to_string(),
        "gm@1l_p@ssw0rd".to_string(),
    ).with_url("https://mail.google.com".to_string())
     .with_tags(vec!["personal".to_string()]);
    
    let login3 = Entry::new_login(
        "AWS Console".to_string(),
        "admin@company.com".to_string(),
        "Aws!Cl0ud_2024$".to_string(),
    ).with_url("https://console.aws.amazon.com".to_string())
     .with_tags(vec!["work".to_string(), "cloud".to_string()])
     .with_totp("GEZDGNBVGY3TQOJQ".to_string());
    
    let note = Entry::new_note(
        "Recovery Codes".to_string(),
        "Backup codes:\n1. ABCD-1234\n2. EFGH-5678\n3. IJKL-9012".to_string(),
    );
    
    vault.add_entry(login1).unwrap();
    vault.add_entry(login2).unwrap();
    vault.add_entry(login3).unwrap();
    vault.add_entry(note).unwrap();
    vault.save("testpass123").unwrap();
    
    println!("✓ Test vault created at {:?} with {} entries", path, vault.entry_count());
}
