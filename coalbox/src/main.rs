use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

use coalbox_core::{
    audit_passwords, check_password, generate_passphrase, generate_password, Entry,
    PassphraseConfig, PasswordConfig, TotpConfig, Vault,
};

#[derive(Parser)]
#[command(name = "coalbox", version = "0.3.0", about = "Coalbox password manager")]
struct Cli {
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
    },

    /// List all entries
    List {
        /// Vault file path
        #[arg(short, long)]
        vault: Option<String>,
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
}

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

fn create_vault(path: &str) {
    let path = PathBuf::from(shellexpand::tilde(path).to_string());
    if path.exists() {
        eprintln!(
            "{} Vault already exists at {}",
            "error:".red().bold(),
            path.display()
        );
        std::process::exit(1);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create vault directory");
    }

    let password = prompt_password("Enter master password: ");
    let confirm = prompt_password("Confirm master password: ");

    if password != confirm {
        eprintln!("{} Passwords don't match", "error:".red().bold());
        std::process::exit(1);
    }

    if password.len() < 8 {
        eprintln!(
            "{} Master password must be at least 8 characters",
            "error:".red().bold()
        );
        std::process::exit(1);
    }

    match Vault::create(&path, &password) {
        Ok(vault) => {
            println!(
                "{} Created vault at {}",
                "✓".green().bold(),
                path.display()
            );
            println!("  {} entries", vault.entry_count());
        }
        Err(e) => {
            eprintln!("{} Failed to create vault: {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn unlock_vault(vault_path: &Option<String>) -> (Vault, String) {
    let path = vault_path
        .as_ref()
        .map(|p| PathBuf::from(shellexpand::tilde(p).to_string()))
        .unwrap_or_else(default_vault_path);

    if !path.exists() {
        eprintln!(
            "{} Vault not found at {}",
            "error:".red().bold(),
            path.display()
        );
        eprintln!(
            "  Run {} to create a vault",
            "coalbox create".cyan()
        );
        std::process::exit(1);
    }

    let password = prompt_password("Enter master password: ");

    match Vault::unlock(&path, &password) {
        Ok(vault) => (vault, password),
        Err(e) => {
            eprintln!("{} Failed to unlock vault: {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn get_entry(query: &str, vault_path: &Option<String>) {
    let (vault, _password) = unlock_vault(vault_path);

    let entry = vault
        .get_entry_by_title(query)
        .or_else(|_| vault.get_entry_by_url(query))
        .or_else(|_| {
            let results = vault.search(query);
            if results.is_empty() {
                Err(coalbox_core::CoalboxError::EntryNotFound(query.to_string()))
            } else {
                Ok(results.into_iter().next().unwrap())
            }
        });

    match entry {
        Ok(entry) => {
            print_entry(&entry);
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn list_entries(vault_path: &Option<String>) {
    let (vault, _password) = unlock_vault(vault_path);

    match vault.list_entries() {
        Ok(entries) => {
            if entries.is_empty() {
                println!("No entries in vault.");
                return;
            }

            println!("{} entries in vault:\n", entries.len().to_string().cyan());
            for entry in &entries {
                let type_str = match entry.entry_type {
                    coalbox_core::entry::EntryType::Login => "login".blue(),
                    coalbox_core::entry::EntryType::Note => "note".yellow(),
                    coalbox_core::entry::EntryType::Card => "card".magenta(),
                    coalbox_core::entry::EntryType::Identity => "identity".green(),
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
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn print_entry(entry: &Entry) {
    let type_str = match entry.entry_type {
        coalbox_core::entry::EntryType::Login => "Login",
        coalbox_core::entry::EntryType::Note => "Secure Note",
        coalbox_core::entry::EntryType::Card => "Payment Card",
        coalbox_core::entry::EntryType::Identity => "Identity",
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

#[allow(clippy::too_many_arguments)]
fn handle_generate(
    passphrase: bool,
    length: usize,
    words: usize,
    separator: String,
    capitalize: bool,
    number: bool,
    uppercase: bool,
    lowercase: bool,
    numbers: bool,
    symbols: bool,
) {
    if passphrase {
        let config = PassphraseConfig {
            word_count: words,
            separator,
            capitalize,
            include_number: number,
        };
        println!("{}", generate_passphrase(&config));
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
        println!("{}", generate_password(&config));
    }
}

fn show_info(vault_path: &Option<String>) {
    let path = vault_path
        .as_ref()
        .map(|p| PathBuf::from(shellexpand::tilde(p).to_string()))
        .unwrap_or_else(default_vault_path);

    if !path.exists() {
        eprintln!(
            "{} Vault not found at {}",
            "error:".red().bold(),
            path.display()
        );
        std::process::exit(1);
    }

    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    println!("{}", "═".repeat(50).dimmed());
    println!("Vault: {}", path.display().to_string().bold());
    println!("Size:  {} bytes", size);
    println!("{}", "═".repeat(50).dimmed());
    println!("Format: .emberkeys (EMBK v1)");
    println!("Cipher: AES-256-GCM");
    println!("KDF:    Argon2id (64MB, 3 iter, 4 parallel)");
}

fn show_totp(query: &str, vault_path: &Option<String>) {
    let (vault, _password) = unlock_vault(vault_path);

    let entry = vault
        .get_entry_by_title(query)
        .or_else(|_| vault.get_entry_by_url(query))
        .or_else(|_| {
            let results = vault.search(query);
            if results.is_empty() {
                Err(coalbox_core::CoalboxError::EntryNotFound(query.to_string()))
            } else {
                Ok(results.into_iter().next().unwrap())
            }
        });

    match entry {
        Ok(entry) => {
            if let Some(ref totp_secret) = entry.totp_secret {
                match TotpConfig::from_secret(totp_secret) {
                    Ok(config) => {
                        let code = config.generate_current();
                        println!("{}", entry.title.bold());
                        println!();
                        println!(
                            "  TOTP: {} ({}s remaining)",
                            code.code.bold().green(),
                            code.remaining
                        );
                    }
                    Err(e) => {
                        eprintln!("{} Invalid TOTP secret: {}", "error:".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("{} No TOTP configured for this entry", "error:".red().bold());
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn audit_vault(vault_path: &Option<String>) {
    let (vault, _password) = unlock_vault(vault_path);

    println!("{}", "Checking passwords against HaveIBeenPwned...".dimmed());
    println!();

    match vault.list_entries() {
        Ok(entries) => {
            match audit_passwords(&entries) {
                Ok(result) => {
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
                Err(e) => {
                    eprintln!("{} Audit failed: {}", "error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn check_single_password(password: Option<String>) {
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
            if result.breached {
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
        Err(e) => {
            eprintln!("{} Breach check failed: {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { path } => create_vault(&path),
        Commands::Get { query, vault } => get_entry(&query, &vault),
        Commands::List { vault } => list_entries(&vault),
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
        } => handle_generate(
            passphrase, length, words, separator, capitalize, number, uppercase, lowercase,
            numbers, symbols,
        ),
        Commands::Lock => {
            eprintln!(
                "{} Daemon mode not yet implemented",
                "todo:".yellow().bold()
            );
            std::process::exit(1);
        }
        Commands::Info { vault } => show_info(&vault),
        Commands::Totp { query, vault } => show_totp(&query, &vault),
        Commands::Audit { vault } => audit_vault(&vault),
        Commands::Check { password } => check_single_password(password),
    }
}
