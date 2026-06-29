# v0.6.1 — WebUI

## What's new

- **WebUI** — beautifully redesigned localhost web interface with TailwindCSS and Lucide icons
- **REST API** — FIXED - full CRUD for entries, search, generate, TOTP
- **WebSocket** — FIXED - real-time lock/unlock/entries notifications
- **Authenticator entries** — separate standalone 2FA entries, with a dedicated tab in the WebUI and animated countdown rings
- **Single binary** — `coalbox-web` with embedded HTML/CSS/JS frontend

## Usage

```bash
# Start the WebUI
coalbox-web --vault ~/vault.emberkeys

# Custom port
coalbox-web --port 3000

# Don't open browser
coalbox-web --no-open
```

## What's not in this release

- Ember Browser integration (v0.7)

## Build from source

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
# binaries at target/release/coalbox and target/release/coalbox-web
```

Requires Rust 1.85+.

## Checksums

Pre-built binaries are for SHA-256 verification only. Do not use them directly.
