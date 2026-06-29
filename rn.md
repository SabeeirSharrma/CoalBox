# v0.2.0 — Full Entry Types + Generator

## What's new

- **Payment Card entries** — cardholder, number (masked), expiry, CVV, PIN
- **Identity entries** — name, email, phone, full address
- **Custom fields** — text, hidden, URL, date types per entry
- **Tags** — add, remove, search by tag
- **Favourites** — mark entries as favourite
- **Password generator** — character mode with configurable length, uppercase, lowercase, numbers, symbols
- **Passphrase generator** — EFF large wordlist (7776 words), configurable word count, separator, capitalization
- **Entry history** — previous passwords retained when updated
- **Display name** — cards show masked number, identities show full name
- **CLI updated** — `--passphrase` flag, `--words`, `--separator`, `--capitalize`, `--number` options

## What's not in this release

- TOTP (v0.3)
- Breach checking (v0.3)
- Import/export (v0.4)
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
