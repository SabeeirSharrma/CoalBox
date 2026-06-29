# v0.3.0 — TOTP + Breach Check

## What's new

- **TOTP code generation** — RFC 6238, SHA-1/SHA-256 support, 6-digit codes
- **Base32 secret decoding** — import TOTP secrets from authenticator apps
- **Breach checking** — HaveIBeenPwned k-anonymity (only hash prefix leaves device)
- **Vault audit** — scan all entries for breached passwords
- **Single password check** — check any password against HIBP
- **CLI commands** — `totp`, `audit`, `check`

## What's not in this release

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
