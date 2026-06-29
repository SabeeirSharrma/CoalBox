use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

use coalbox_core::{Entry, Vault};

#[derive(Parser)]
#[command(name = "coalbox", version = "0.1.0", about = "Coalbox password manager")]
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

    /// Generate a password
    Generate {
        /// Password length
        #[arg(short, long, default_value = "20")]
        length: usize,

        /// Include uppercase letters
        #[arg(long, default_value = "true")]
        uppercase: bool,

        /// Include lowercase letters
        #[arg(long, default_value = "true")]
        lowercase: bool,

        /// Include numbers
        #[arg(long, default_value = "true")]
        numbers: bool,

        /// Include symbols
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
        eprintln!("{} Vault already exists at {}", "error:".red().bold(), path.display());
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
        eprintln!("{} Master password must be at least 8 characters", "error:".red().bold());
        std::process::exit(1);
    }

    match Vault::create(&path, &password) {
        Ok(vault) => {
            println!("{} Created vault at {}", "✓".green().bold(), path.display());
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
        eprintln!("{} Vault not found at {}", "error:".red().bold(), path.display());
        eprintln!("  Run {} to create a vault", "coalbox create".cyan());
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

    // Try exact title match first, then URL, then search
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

                println!(
                    "  {} {} {} {}",
                    entry.id.to_string()[..8].dimmed(),
                    entry.title.bold(),
                    format!("[{}]", type_str).dimmed(),
                    fav.yellow()
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
    println!("{}", "═".repeat(50).dimmed());

    if let Some(ref url) = entry.url {
        println!("URL:      {}", url);
    }
    if let Some(ref username) = entry.username {
        println!("Username: {}", username);
    }
    if let Some(ref password) = entry.password {
        println!("Password: {}", "*".repeat(password.len()));
    }
    if let Some(ref notes) = entry.notes {
        println!("Notes:    {}", notes);
    }
    if !entry.tags.is_empty() {
        println!("Tags:     {}", entry.tags.join(", "));
    }
    println!("Created:  {}", entry.created.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Modified: {}", entry.modified.format("%Y-%m-%d %H:%M:%S UTC"));
}

fn generate_password(length: usize, uppercase: bool, lowercase: bool, numbers: bool, symbols: bool) {
    let mut charset = Vec::new();

    if uppercase {
        charset.extend_from_slice(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if lowercase {
        charset.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz");
    }
    if numbers {
        charset.extend_from_slice(b"0123456789");
    }
    if symbols {
        charset.extend_from_slice(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
    }

    if charset.is_empty() {
        eprintln!("{} At least one character set must be enabled", "error:".red().bold());
        std::process::exit(1);
    }

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect();

    println!("{}", password);
}

fn show_info(vault_path: &Option<String>) {
    let path = vault_path
        .as_ref()
        .map(|p| PathBuf::from(shellexpand::tilde(p).to_string()))
        .unwrap_or_else(default_vault_path);

    if !path.exists() {
        eprintln!("{} Vault not found at {}", "error:".red().bold(), path.display());
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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { path } => create_vault(&path),
        Commands::Get { query, vault } => get_entry(&query, &vault),
        Commands::List { vault } => list_entries(&vault),
        Commands::Generate {
            length,
            uppercase,
            lowercase,
            numbers,
            symbols,
        } => generate_password(length, uppercase, lowercase, numbers, symbols),
        Commands::Lock => {
            eprintln!("{} Daemon mode not yet implemented", "todo:".yellow().bold());
            std::process::exit(1);
        }
        Commands::Info { vault } => show_info(&vault),
    }
}
