# v0.4.0 — Import / Export

## What's new

- **Import from CSV** — flexible column mapping (name/title, username/user/login/email, password/pass/pwd, url/website/site/link, notes/note/comment)
- **Import from Bitwarden JSON** — full vault export support, skips non-login items
- **Import from KeePass XML** — parses exported KeePass XML format
- **Import from 1Password 1PUX** — extracts from 1Password export archives (.zip)
- **Export to plaintext JSON** — full entry data, human-readable
- **Auto-format detection** — picks format from file extension
- **Duplicate detection** — skips entries with existing titles during import
- **CLI commands** — `import`, `export`

## What's not in this release

- WebUI (v0.6)
- Ember Browser integration (v0.7)

## Build from source

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
```

Requires Rust 1.85+.

## Checksums

Pre-built binaries are for SHA-256 verification only. Do not use them directly.
