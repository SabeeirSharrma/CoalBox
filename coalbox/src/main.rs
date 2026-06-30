use clap::{Parser, Subcommand};
use colored::Colorize;
use serde::Serialize;
use std::path::PathBuf;
use std::process;

use coalbox_core::{
    audit_passwords, check_for_update, check_password, generate_passphrase,
    generate_password, import_file, Entry, ImportFormat, PassphraseConfig, PasswordConfig,
    TotpConfig, Vault,
};

const EXIT_ERROR: i32 = 1;

#[derive(Parser)]
#[command(
    name = "coalbox",
    version = "0.6.3",
    about = "Coalbox password manager",
    after_help = "Run 'coalbox <command> --help' for more information on a specific command."
)]
struct Cli {
    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new vault
    Create {
        /// Path for the new vault file
        #[arg(default_value = "~/.local/share/coalbox/vault.emberkeys")]
        path: String,
    },

    /// Get an entry by title or URL
    Get {
        /// Entry title or URL to search for
        query: String,

        /// Vault file path
        #[arg(short, long)]
        vault: Option<String>,

        /// Output field to return (title, username, password, url, totp, notes, json)
        #[arg(short = 'f', long)]
        field: Option<String>,
    },

    /// List all entries
    List {
        /// Vault file path
        #[arg(short, long)]
        vault: Option<String>,

        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Filter by type (login, note, card, identity)
        #[arg(short = 't', long)]
        entry_type: Option<String>,
    },

    /// Generate a password or passphrase
    Generate {
        /// Generate a passphrase instead of a password
        #[arg(long)]
        passphrase: bool,

        /// Password length (character mode)
        #[arg(short, long, default_value = "20")]
        length: usize,

        /// Word count (passphrase mode)
        #[arg(short = 'w', long, default_value = "6")]
        words: usize,

        /// Separator character (passphrase mode)
        #[arg(short, long, default_value = " ")]
        separator: String,

        /// Capitalize words (passphrase mode)
        #[arg(long, default_value = "true")]
        capitalize: bool,

        /// Include a number at the end (passphrase mode)
        #[arg(long)]
        number: bool,

        /// Include uppercase letters (character mode)
        #[arg(long, default_value = "true")]
        uppercase: bool,

        /// Include lowercase letters (character mode)
        #[arg(long, default_value = "true")]
        lowercase: bool,

        /// Include numbers (character mode)
        #[arg(long, default_value = "true")]
        numbers: bool,

        /// Include symbols (character mode)
        #[arg(long, default_value = "true")]
        symbols: bool,
    },

    /// Lock the vault (daemon mode, not yet implemented)
    Lock,

    /// Show vault info
    Info {
        /// Vault file path
        #[arg(short, long)]
        vault: Option<String>,
    },

    /// Show TOTP code for an entry
    Totp {
        /// Entry title or URL to search for
        query: String,

        /// Vault file path
        #[arg(short, long)]
        vault: Option<String>,
    },

    /// Check passwords against HaveIBeenPwned
    Audit {
        /// Vault file path
        #[arg(short, long)]
        vault: Option<String>,
    },

    /// Check a single password against HaveIBeenPwned
    Check {
        /// Password to check (or - to read from stdin)
        password: Option<String>,
    },

    /// Import entries from a file
    Import {
        /// File to import (csv, json, xml, 1pux)
        file: String,

        /// Import format (csv, bitwarden, keepass, 1password, auto)
        #[arg(short, long, default_value = "auto")]
        format: String,

        /// Vault file path
        #[arg(short, long)]
        vault: Option<String>,
    },

    /// Migrate vault to another encrypted format
    Migrate {
        /// Target format (kdbx, bitwarden)
        #[arg(short = 't', long)]
        to: String,

        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Vault file path
        #[arg(short, long)]
        vault: Option<String>,
    },

    /// Check for updates and optionally update
    Update {
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

// ── JSON output helpers ──────────────────────────────────────────────

fn json_output<T: Serialize>(data: &T) {
    println!("{}", serde_json::to_string(data).unwrap());
}

fn json_error(msg: &str) {
    let obj = serde_json::json!({ "error": msg });
    eprintln!("{}", serde_json::to_string(&obj).unwrap());
}

fn exit_error(cli: &Cli, msg: &str) -> ! {
    if cli.json {
        json_error(msg);
    } else {
        eprintln!("{} {}", "error:".red().bold(), msg);
    }
    process::exit(EXIT_ERROR);
}

// ── Vault helpers ────────────────────────────────────────────────────

fn default_vault_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("coalbox")
        .join("vault.emberkeys")
}

fn prompt_password(prompt: &str) -> String {
    rpassword::prompt_password(prompt).expect("Failed to read password")
}

fn resolve_vault_path(opts: &Option<String>) -> PathBuf {
    opts.as_ref()
        .map(|p| PathBuf::from(shellexpand::tilde(p).to_string()))
        .unwrap_or_else(default_vault_path)
}

fn unlock_vault(cli: &Cli, vault_path: &Option<String>) -> (Vault, String) {
    let path = resolve_vault_path(vault_path);

    if !path.exists() {
        exit_error(cli, &format!("Vault not found at {}", path.display()));
    }

    let password = prompt_password("Enter master password: ");

    match Vault::unlock(&path, &password) {
        Ok(vault) => (vault, password),
        Err(e) => exit_error(cli, &format!("Failed to unlock vault: {}", e)),
    }
}

fn find_entry(vault: &Vault, query: &str) -> Entry {
    vault
        .get_entry_by_title(query)
        .or_else(|_| vault.get_entry_by_url(query))
        .or_else(|_| {
            let results = vault.search(query);
            if results.is_empty() {
                Err(coalbox_core::CoalboxError::EntryNotFound(query.to_string()))
            } else {
                Ok(results.into_iter().next().unwrap())
            }
        })
        .unwrap_or_else(|e| {
            eprintln!("{} {}", "error:".red().bold(), e);
            process::exit(EXIT_ERROR);
        })
}

// ── Command handlers ─────────────────────────────────────────────────

fn create_vault(cli: &Cli, path: &str) {
    let path = PathBuf::from(shellexpand::tilde(path).to_string());
    if path.exists() {
        exit_error(cli, &format!("Vault already exists at {}", path.display()));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create vault directory");
    }

    let password = prompt_password("Enter master password: ");
    let confirm = prompt_password("Confirm master password: ");

    if password != confirm {
        exit_error(cli, "Passwords don't match");
    }

    if password.len() < 8 {
        exit_error(cli, "Master password must be at least 8 characters");
    }

    match Vault::create(&path, &password) {
        Ok(vault) => {
            if cli.json {
                json_output(&serde_json::json!({
                    "ok": true,
                    "path": path.display().to_string(),
                    "entries": vault.entry_count()
                }));
            } else if !cli.quiet {
                println!(
                    "{} Created vault at {}",
                    "✓".green().bold(),
                    path.display()
                );
                println!("  {} entries", vault.entry_count());
            }
        }
        Err(e) => exit_error(cli, &format!("Failed to create vault: {}", e)),
    }
}

fn get_entry(cli: &Cli, query: &str, vault_path_opt: &Option<String>, field: &Option<String>) {
    let (vault, _password) = unlock_vault(cli, vault_path_opt);
    let entry = find_entry(&vault, query);

    if cli.json {
        match field.as_deref() {
            Some("title") => println!("{}", serde_json::to_string(&entry.title).unwrap()),
            Some("username") => {
                println!(
                    "{}",
                    serde_json::to_string(&entry.username.as_deref().unwrap_or("")).unwrap()
                )
            }
            Some("password") => {
                println!(
                    "{}",
                    serde_json::to_string(&entry.password.as_deref().unwrap_or("")).unwrap()
                )
            }
            Some("url") => {
                println!(
                    "{}",
                    serde_json::to_string(&entry.url.as_deref().unwrap_or("")).unwrap()
                )
            }
            Some("totp") => {
                let totp = entry
                    .totp_secret
                    .as_ref()
                    .and_then(|s| TotpConfig::from_secret(s).ok())
                    .map(|c| c.generate_current());
                let code = totp.as_ref().map(|t| t.code.as_str()).unwrap_or("");
                let remaining = totp.as_ref().map(|t| t.remaining).unwrap_or(0);
                json_output(&serde_json::json!({ "code": code, "remaining": remaining }));
            }
            Some("notes") => {
                println!(
                    "{}",
                    serde_json::to_string(&entry.notes.as_deref().unwrap_or("")).unwrap()
                )
            }
            None => json_output(&entry),
            Some(f) => exit_error(cli, &format!("Unknown field: {}", f)),
        }
    } else {
        match field.as_deref() {
            Some("title") => println!("{}", entry.title),
            Some("username") => println!("{}", entry.username.as_deref().unwrap_or("")),
            Some("password") => println!("{}", entry.password.as_deref().unwrap_or("")),
            Some("url") => println!("{}", entry.url.as_deref().unwrap_or("")),
            Some("totp") => {
                if let Some(ref secret) = entry.totp_secret {
                    match TotpConfig::from_secret(secret) {
                        Ok(config) => {
                            let code = config.generate_current();
                            println!("{} ({}s remaining)", code.code, code.remaining);
                        }
                        Err(e) => exit_error(cli, &format!("Invalid TOTP secret: {}", e)),
                    }
                } else {
                    exit_error(cli, "No TOTP configured for this entry");
                }
            }
            Some("notes") => println!("{}", entry.notes.as_deref().unwrap_or("")),
            None => print_entry_pretty(&entry),
            Some(f) => exit_error(cli, &format!("Unknown field: {}", f)),
        }
    }
}

fn list_entries(
    cli: &Cli,
    vault_path_opt: &Option<String>,
    tag: &Option<String>,
    entry_type: &Option<String>,
) {
    let (vault, _password) = unlock_vault(cli, vault_path_opt);

    let entries = vault.list_entries().unwrap_or_default();
    let filtered: Vec<&Entry> = entries
        .iter()
        .filter(|e| {
            if let Some(t) = tag
                && !e.tags.contains(t)
            {
                return false;
            }
            if let Some(t) = entry_type {
                let type_str = match e.entry_type {
                    coalbox_core::entry::EntryType::Login => "login",
                    coalbox_core::entry::EntryType::Note => "note",
                    coalbox_core::entry::EntryType::Card => "card",
                    coalbox_core::entry::EntryType::Identity => "identity",
                    coalbox_core::entry::EntryType::Authenticator => "authenticator",
                };
                if type_str != t.as_str() {
                    return false;
                }
            }
            true
        })
        .collect();

    if cli.json {
        json_output(&filtered);
    } else if !cli.quiet {
        if filtered.is_empty() {
            println!("No entries in vault.");
            return;
        }

        println!("{} entries in vault:\n", filtered.len().to_string().cyan());
        for entry in &filtered {
            print_entry_summary(entry);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_generate(cli: &Cli, passphrase: bool, length: usize, words: usize, separator: String, capitalize: bool, number: bool, uppercase: bool, lowercase: bool, numbers: bool, symbols: bool) {
    if passphrase {
        let config = PassphraseConfig {
            word_count: words,
            separator,
            capitalize,
            include_number: number,
        };
        let result = generate_passphrase(&config);
        if cli.json {
            json_output(&serde_json::json!({ "passphrase": result }));
        } else {
            println!("{}", result);
        }
    } else {
        let config = PasswordConfig {
            length,
            uppercase,
            lowercase,
            numbers,
            symbols,
            custom_symbols: None,
            exclude_chars: None,
        };
        let result = generate_password(&config);
        if cli.json {
            json_output(&serde_json::json!({ "password": result }));
        } else {
            println!("{}", result);
        }
    }
}

fn show_info(cli: &Cli, vault_path_opt: &Option<String>) {
    let path = resolve_vault_path(vault_path_opt);

    if !path.exists() {
        exit_error(cli, &format!("Vault not found at {}", path.display()));
    }

    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

    if cli.json {
        json_output(&serde_json::json!({
            "path": path.display().to_string(),
            "size_bytes": size,
            "format": ".emberkeys",
            "cipher": "AES-256-GCM",
            "kdf": "Argon2id",
            "kdf_params": {
                "memory_kb": 65536,
                "iterations": 3,
                "parallelism": 4
            }
        }));
    } else if !cli.quiet {
        println!("{}", "═".repeat(50).dimmed());
        println!("Vault: {}", path.display().to_string().bold());
        println!("Size:  {} bytes", size);
        println!("{}", "═".repeat(50).dimmed());
        println!("Format: .emberkeys (EMBK v1)");
        println!("Cipher: AES-256-GCM");
        println!("KDF:    Argon2id (64MB, 3 iter, 4 parallel)");
    }
}

fn show_totp(cli: &Cli, query: &str, vault_path_opt: &Option<String>) {
    let (vault, _password) = unlock_vault(cli, vault_path_opt);
    let entry = find_entry(&vault, query);

    match entry.totp_secret {
        Some(ref secret) => match TotpConfig::from_secret(secret) {
            Ok(config) => {
                let code = config.generate_current();
                if cli.json {
                    json_output(&serde_json::json!({
                        "title": entry.title,
                        "code": code.code,
                        "remaining": code.remaining
                    }));
                } else {
                    println!("{}", entry.title.bold());
                    println!();
                    println!(
                        "  TOTP: {} ({}s remaining)",
                        code.code.bold().green(),
                        code.remaining
                    );
                }
            }
            Err(e) => exit_error(cli, &format!("Invalid TOTP secret: {}", e)),
        },
        None => exit_error(cli, "No TOTP configured for this entry"),
    }
}

fn audit_vault(cli: &Cli, vault_path_opt: &Option<String>) {
    let (vault, _password) = unlock_vault(cli, vault_path_opt);

    if !cli.quiet && !cli.json {
        println!("{}", "Checking passwords against HaveIBeenPwned...".dimmed());
        println!();
    }

    let entries = vault.list_entries().unwrap_or_default();

    match audit_passwords(&entries) {
        Ok(result) => {
            if cli.json {
                json_output(&serde_json::json!({
                    "total_entries": result.total_entries,
                    "entries_with_passwords": result.entries_with_passwords,
                    "breached_entries": result.breached_entries.iter().map(|e| {
                        serde_json::json!({
                            "title": e.title,
                            "entry_id": e.entry_id,
                            "breach_count": e.breach_count
                        })
                    }).collect::<Vec<_>>()
                }));
            } else if !cli.quiet {
                println!("Vault audit complete:");
                println!("  Total entries:       {}", result.total_entries);
                println!(
                    "  Entries with pass:   {}",
                    result.entries_with_passwords
                );
                println!();

                if result.breached_entries.is_empty() {
                    println!(
                        "{} No breached passwords found!",
                        "✓".green().bold()
                    );
                } else {
                    println!(
                        "{} {} breached passwords found:",
                        "⚠".red().bold(),
                        result.breached_entries.len()
                    );
                    println!();
                    for entry in &result.breached_entries {
                        println!(
                            "  {} {} — seen {} times in data breaches",
                            entry.title.bold(),
                            entry.entry_id.to_string()[..8].dimmed(),
                            entry.breach_count.to_string().red()
                        );
                    }
                }
            }
        }
        Err(e) => exit_error(cli, &format!("Audit failed: {}", e)),
    }
}

fn check_single_password(cli: &Cli, password: Option<String>) {
    let pass = match password {
        Some(p) => {
            if p == "-" {
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read from stdin");
                input.trim().to_string()
            } else {
                p
            }
        }
        None => prompt_password("Enter password to check: "),
    };

    match check_password(&pass) {
        Ok(result) => {
            if cli.json {
                json_output(&serde_json::json!({
                    "breached": result.breached,
                    "count": result.count
                }));
            } else if result.breached {
                println!(
                    "{} Password found in {} data breaches!",
                    "⚠".red().bold(),
                    result.count.to_string().red().bold()
                );
                println!("  Do not use this password.");
            } else {
                println!(
                    "{} Password not found in any data breaches.",
                    "✓".green().bold()
                );
            }
        }
        Err(e) => exit_error(cli, &format!("Breach check failed: {}", e)),
    }
}

fn parse_import_format(format: &str) -> ImportFormat {
    match format.to_lowercase().as_str() {
        "csv" => ImportFormat::Csv,
        "bitwarden" | "bw" => ImportFormat::BitwardenJson,
        "keepass" | "kpx" => ImportFormat::KeePassXml,
        "1password" | "1pux" => ImportFormat::OnePassword1Pux,
        "auto" => ImportFormat::Auto,
        _ => ImportFormat::Auto,
    }
}

fn import_entries(cli: &Cli, file: &str, format: &str, vault_path_opt: &Option<String>) {
    let path = PathBuf::from(shellexpand::tilde(file).to_string());

    if !path.exists() {
        exit_error(cli, &format!("File not found: {}", path.display()));
    }

    let fmt = parse_import_format(format);
    let detected_fmt = if matches!(fmt, ImportFormat::Auto) {
        ImportFormat::detect(&path)
    } else {
        fmt.clone()
    };

    if !cli.quiet && !cli.json {
        println!(
            "{} Importing from {} (format: {:?})...",
            "→".cyan(),
            path.display(),
            detected_fmt
        );
    }

    let result = match import_file(&path, fmt) {
        Ok(r) => r,
        Err(e) => exit_error(cli, &format!("Import failed: {}", e)),
    };

    if result.entries.is_empty() {
        if cli.json {
            json_output(&serde_json::json!({
                "ok": true,
                "imported": 0,
                "skipped": result.skipped,
                "errors": result.errors
            }));
        } else if !cli.quiet {
            eprintln!(
                "{} No entries found in import file",
                "warning:".yellow().bold()
            );
        }
        return;
    }

    let (mut vault, password) = unlock_vault(cli, vault_path_opt);

    let existing = vault
        .list_entries()
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.title.to_lowercase())
        .collect::<Vec<_>>();

    let mut imported = 0;
    let mut skipped_dupes = 0;

    for entry in result.entries {
        if existing.contains(&entry.title.to_lowercase()) {
            skipped_dupes += 1;
            if !cli.quiet && !cli.json {
                println!(
                    "  {} {} (duplicate, skipped)",
                    "⊘".yellow(),
                    entry.title
                );
            }
            continue;
        }
        match vault.add_entry(entry.clone()) {
            Ok(_) => {
                imported += 1;
                if !cli.quiet && !cli.json {
                    println!("  {} {}", "✓".green(), entry.title);
                }
            }
            Err(e) => {
                if !cli.quiet && !cli.json {
                    println!("  {} {} — {}", "✗".red(), entry.title, e);
                }
            }
        }
    }

    if imported > 0
        && let Err(e) = vault.save(&password)
    {
        exit_error(cli, &format!("Failed to save vault: {}", e));
    }

    if cli.json {
        json_output(&serde_json::json!({
            "ok": true,
            "imported": imported,
            "skipped_duplicates": skipped_dupes,
            "parse_errors": result.errors.len(),
            "skipped": result.skipped
        }));
    } else if !cli.quiet {
        println!(
            "\n{} Imported {} entries",
            "✓".green().bold(),
            imported.to_string().green().bold()
        );
    }
}

fn migrate_vault(cli: &Cli, target: &str, output: &str, vault_path_opt: &Option<String>) {
    let (vault, _password) = unlock_vault(cli, vault_path_opt);

    let entries = match vault.list_entries() {
        Ok(e) => e,
        Err(e) => exit_error(cli, &format!("Failed to list entries: {}", e)),
    };

    let path = PathBuf::from(shellexpand::tilde(output).to_string());

    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).expect("Failed to create output directory");
    }

    let export_password = prompt_password("Enter export password: ");
    let confirm = prompt_password("Confirm export password: ");

    if export_password != confirm {
        exit_error(cli, "Export passwords don't match");
    }

    if export_password.len() < 8 {
        exit_error(cli, "Export password must be at least 8 characters");
    }

    if !cli.quiet && !cli.json {
        println!(
            "{} Migrating {} entries to {}...",
            "→".cyan(),
            entries.len(),
            target
        );
    }

    let result = match target.to_lowercase().as_str() {
        "kdbx" => coalbox_core::migrate::export_kdbx(&entries, &path, &export_password),
        "bitwarden" => coalbox_core::migrate::export_bitwarden_encrypted(&entries, &path, &export_password),
        _ => exit_error(cli, &format!("Unsupported target format: {}. Use 'kdbx' or 'bitwarden'.", target)),
    };

    match result {
        Ok(()) => {
            if cli.json {
                json_output(&serde_json::json!({
                    "ok": true,
                    "exported": entries.len(),
                    "format": target,
                    "path": path.display().to_string()
                }));
            } else if !cli.quiet {
                println!(
                    "{} Migrated {} entries to {} ({})",
                    "✓".green().bold(),
                    entries.len().to_string().green().bold(),
                    path.display(),
                    target
                );
            }
        }
        Err(e) => exit_error(cli, &format!("Migration failed: {}", e)),
    }
}

fn check_and_update(cli: &Cli, skip_confirm: bool) {
    if !cli.quiet && !cli.json {
        println!("{}", "Checking for updates...".dimmed());
    }

    let check = match check_for_update() {
        Ok(c) => c,
        Err(e) => exit_error(cli, &format!("Failed to check for updates: {}", e)),
    };

    if !check.update_available {
        if cli.json {
            json_output(&serde_json::json!({
                "ok": true,
                "update_available": false,
                "current_version": check.current_version,
                "latest_version": check.latest_version,
            }));
        } else if !cli.quiet {
            println!(
                "{} You're running the latest version (v{})",
                "✓".green().bold(),
                check.current_version
            );
        }
        return;
    }

    if cli.json {
        json_output(&serde_json::json!({
            "ok": true,
            "update_available": true,
            "current_version": check.current_version,
            "latest_version": check.latest_version,
        }));
    } else if !cli.quiet {
        println!(
            "{} New update available: {} -> {}",
            "↑".cyan().bold(),
            check.current_version,
            check.latest_version.green().bold()
        );
        if let Some(ref release) = check.release
            && !release.body.is_empty()
        {
            println!();
            println!("{}", release.body);
        }
        println!();
    }

    if !skip_confirm && !cli.quiet {
        print!("Update now? [y/N] ");
        use std::io::Write;
        std::io::stdout().flush().ok();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        if !input.trim().eq_ignore_ascii_case("y") && !input.trim().eq_ignore_ascii_case("yes") {
            if !cli.quiet {
                println!("Update cancelled.");
            }
            return;
        }
    }

    run_update(cli);
}

fn run_update(cli: &Cli) {
    if !cli.quiet {
        println!("{}", "Building and installing update...".dimmed());
    }

    let update_script = include_str!("../../update.sh");

    let tmp_script = std::env::temp_dir().join("coalbox_update.sh");
    std::fs::write(&tmp_script, update_script).expect("Failed to write update script");

    let status = std::process::Command::new("bash")
        .arg(&tmp_script)
        .status();

    let _ = std::fs::remove_file(&tmp_script);

    match status {
        Ok(s) if s.success() => {
            if cli.json {
                json_output(&serde_json::json!({
                    "ok": true,
                    "message": "Update completed successfully"
                }));
            } else if !cli.quiet {
                println!(
                    "{} Update completed! Restart coalbox to use the new version.",
                    "✓".green().bold()
                );
            }
        }
        Ok(s) => exit_error(cli, &format!("Update script exited with status: {}", s)),
        Err(e) => exit_error(cli, &format!("Failed to run update script: {}", e)),
    }
}

// ── Pretty printers ──────────────────────────────────────────────────

fn print_entry_summary(entry: &Entry) {
    let type_str = match entry.entry_type {
        coalbox_core::entry::EntryType::Login => "login".blue(),
        coalbox_core::entry::EntryType::Note => "note".yellow(),
        coalbox_core::entry::EntryType::Card => "card".magenta(),
        coalbox_core::entry::EntryType::Identity => "identity".green(),
        coalbox_core::entry::EntryType::Authenticator => "authenticator".cyan(),
    };

    let fav = if entry.favourite { " ★" } else { "" };
    let tags = if entry.tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", entry.tags.join(", ")).dimmed().to_string()
    };

    let display = entry.display_name();
    println!(
        "  {} {} {} {}{}",
        entry.id.to_string()[..8].dimmed(),
        display.bold(),
        format!("[{}]", type_str).dimmed(),
        fav.yellow(),
        tags
    );

    if let Some(ref url) = entry.url {
        println!("    url: {}", url.dimmed());
    }
    if let Some(ref username) = entry.username {
        println!("    user: {}", username.dimmed());
    }
}

fn print_entry_pretty(entry: &Entry) {
    let type_str = match entry.entry_type {
        coalbox_core::entry::EntryType::Login => "Login",
        coalbox_core::entry::EntryType::Note => "Secure Note",
        coalbox_core::entry::EntryType::Card => "Payment Card",
        coalbox_core::entry::EntryType::Identity => "Identity",
        coalbox_core::entry::EntryType::Authenticator => "Authenticator",
    };

    println!("{}", "═".repeat(50).dimmed());
    println!("{}", entry.title.bold());
    println!("{}", type_str.dimmed());
    if entry.favourite {
        println!("{}", "★ Favourite".yellow());
    }
    println!("{}", "═".repeat(50).dimmed());

    if let Some(ref url) = entry.url {
        println!("URL:      {}", url);
    }
    if let Some(ref username) = entry.username {
        println!("Username: {}", username);
    }
    if let Some(ref password) = entry.password {
        println!("Password: {}", "*".repeat(password.len()));
        if !entry.password_history.is_empty() {
            println!(
                "          ({} previous passwords in history)",
                entry.password_history.len()
            );
        }
    }
    if let Some(ref notes) = entry.notes {
        println!("Notes:    {}", notes);
    }

    if let Some(ref card) = entry.card {
        println!();
        println!("Card Details:");
        if let Some(ref holder) = card.cardholder {
            println!("  Cardholder: {}", holder);
        }
        if let Some(ref number) = card.number {
            let masked = if number.len() > 4 {
                format!("•••• {}", &number[number.len() - 4..])
            } else {
                number.clone()
            };
            println!("  Number:     {}", masked);
        }
        if let Some(ref expiry) = card.expiry {
            println!("  Expires:    {}", expiry);
        }
        if let Some(ref cvv) = card.cvv {
            println!("  CVV:        {}", "*".repeat(cvv.len()));
        }
        if let Some(ref pin) = card.pin {
            println!("  PIN:        {}", "*".repeat(pin.len()));
        }
    }

    if let Some(ref identity) = entry.identity {
        println!();
        println!("Identity:");
        if let Some(ref name) = identity.first_name {
            let full = match (&identity.middle_name, &identity.last_name) {
                (Some(mid), Some(last)) => format!("{} {} {}", name, mid, last),
                (None, Some(last)) => format!("{} {}", name, last),
                _ => name.clone(),
            };
            println!("  Name:   {}", full);
        }
        if let Some(ref email) = identity.email {
            println!("  Email:  {}", email);
        }
        if let Some(ref phone) = identity.phone {
            println!("  Phone:  {}", phone);
        }
        if let Some(ref addr) = identity.address_line1 {
            print!("  Address: {}", addr);
            if let Some(ref city) = identity.city {
                print!(", {}", city);
            }
            if let Some(ref state) = identity.state {
                print!(", {}", state);
            }
            if let Some(ref zip) = identity.postal_code {
                print!(" {}", zip);
            }
            println!();
        }
    }

    if !entry.custom_fields.is_empty() {
        println!();
        println!("Custom Fields:");
        for field in &entry.custom_fields {
            let value = match field.field_type {
                coalbox_core::entry::FieldType::Hidden => "*".repeat(field.value.len()),
                _ => field.value.clone(),
            };
            println!("  {}: {}", field.name, value);
        }
    }

    if !entry.tags.is_empty() {
        println!();
        println!("Tags: {}", entry.tags.join(", "));
    }

    println!();
    println!(
        "Created:  {}",
        entry.created.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!(
        "Modified: {}",
        entry.modified.format("%Y-%m-%d %H:%M:%S UTC")
    );
}

// ── Main ─────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let json = cli.json;
    let quiet = cli.quiet;

    match cli.command {
        Commands::Create { path } => {
            let c = Cli { json, quiet, command: Commands::Create { path: path.clone() } };
            create_vault(&c, &path);
        }
        Commands::Get {
            query,
            vault,
            field,
        } => {
            let c = Cli { json, quiet, command: Commands::Get { query: query.clone(), vault: vault.clone(), field: field.clone() } };
            get_entry(&c, &query, &vault, &field);
        }
        Commands::List {
            vault,
            tag,
            entry_type,
        } => {
            let c = Cli { json, quiet, command: Commands::List { vault: vault.clone(), tag: tag.clone(), entry_type: entry_type.clone() } };
            list_entries(&c, &vault, &tag, &entry_type);
        }
        Commands::Generate {
            passphrase,
            length,
            words,
            separator,
            capitalize,
            number,
            uppercase,
            lowercase,
            numbers,
            symbols,
        } => {
            let c = Cli { json, quiet, command: Commands::Generate { passphrase, length, words, separator: separator.clone(), capitalize, number, uppercase, lowercase, numbers, symbols } };
            handle_generate(&c, passphrase, length, words, separator, capitalize, number, uppercase, lowercase, numbers, symbols);
        }
        Commands::Lock => {
            let c = Cli { json, quiet, command: Commands::Lock };
            exit_error(&c, "Daemon mode not yet implemented");
        }
        Commands::Info { vault } => {
            let c = Cli { json, quiet, command: Commands::Info { vault: vault.clone() } };
            show_info(&c, &vault);
        }
        Commands::Totp { query, vault } => {
            let c = Cli { json, quiet, command: Commands::Totp { query: query.clone(), vault: vault.clone() } };
            show_totp(&c, &query, &vault);
        }
        Commands::Audit { vault } => {
            let c = Cli { json, quiet, command: Commands::Audit { vault: vault.clone() } };
            audit_vault(&c, &vault);
        }
        Commands::Check { password } => {
            let c = Cli { json, quiet, command: Commands::Check { password: password.clone() } };
            check_single_password(&c, password);
        }
        Commands::Import { file, format, vault } => {
            let c = Cli { json, quiet, command: Commands::Import { file: file.clone(), format: format.clone(), vault: vault.clone() } };
            import_entries(&c, &file, &format, &vault);
        }
        Commands::Migrate { to, output, vault } => {
            let c = Cli { json, quiet, command: Commands::Migrate { to: to.clone(), output: output.clone(), vault: vault.clone() } };
            migrate_vault(&c, &to, &output, &vault);
        }
        Commands::Update { yes } => {
            let c = Cli { json, quiet, command: Commands::Update { yes } };
            check_and_update(&c, yes);
        }
    }
}
