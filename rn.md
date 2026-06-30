# v0.6.2 — WebUI Polish

## What's new

- **Create vault from UI** — form appears when no vault exists
- **Enter key submits all forms** — unlock, create vault, create/edit entry
- **Copy from generator** — paste generated password directly into password field
- **TOTP countdown bar** — visual timer with progress bar
- **Keyboard shortcuts** — `Ctrl+K` search, `Ctrl+N` new entry, `Esc` close modals
- **Favourites** — star/unstar entries, filter sidebar by favourites
- **Tags in sidebar** — filter by tag, badge counts
- **Import/Export from UI** — import CSV/Bitwarden/KeePass/1Password, export JSON
- **Clipboard auto-clear** — copy clears after 30 seconds
- **Vault info panel** — vault stats (path, entries, format, cipher, KDF)
- **Empty state** — better UX when vault has no entries
- **Entry type icons** — visual distinction between login, note, card, identity

## Build from source

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
```

Requires Rust 1.85+.

## Checksums

Pre-built binaries are for SHA-256 verification only. Do not use them directly.
