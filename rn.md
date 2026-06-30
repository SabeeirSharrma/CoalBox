# v0.6.3 — Migration System

## What's new

- **`coalbox migrate` command** — export vault to encrypted KDBX or Bitwarden format
- **No plaintext export** — Coalbox never writes decrypted data to disk
- **KDBX (KeePass)** — primary target, universal bridge to all major password managers
- **Bitwarden encrypted export** — secondary target for Bitwarden users
- **WebUI migrate panel** — migrate from the browser with encrypted download
- **Removed plaintext JSON export** — security: decrypted data never touches disk

## Usage

```bash
# Migrate to KDBX (importable by KeePass, KeePassXC, 1Password, etc.)
coalbox migrate --to kdbx --output ~/vault.kdbx

# Migrate to Bitwarden encrypted export
coalbox migrate --to bitwarden --output ~/export.json
```

Each command prompts for:
1. Master password (to unlock vault)
2. Export password (to protect the new file — never the same as master)

## Build from source

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
```

Requires Rust 1.85+.
