# v0.1.0 — Core Vault

First release of Coalbox.

## What's included

- `.emberkeys` vault format (EMBK magic, versioned, open spec)
- AES-256-GCM encryption with Argon2id key derivation (64MB, 3 iter, 4 parallelism)
- Vault create, unlock, lock, save
- Entry CRUD — Login, Secure Note, Payment Card, Identity types
- Search across title, URL, username, notes
- CLI: `create`, `get`, `list`, `generate`, `info`
- Password generator (configurable length, character sets)
- 15 passing tests

## What's not in this release

- TOTP (v0.3)
- Breach checking (v0.3)
- Import/export (v0.4)
- Ember Browser integration (v0.6)
- Daemon/lock mode (v0.5)

## Build from source

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
```

Requires Rust 1.85+.

## Checksums

Pre-built binaries are for SHA-256 verification only. Do not use them directly.
