# v0.6.0 — WebUI

## What's new

- **WebUI** — localhost web interface at `http://127.0.0.1:<port>`
- **axum server** — lightweight async Rust web framework
- **REST API** — full CRUD for entries, search, generate, TOTP
- **WebSocket** — real-time lock/unlock/entries notifications
- **Vanilla frontend** — HTML/CSS/JS, Tailwind CDN, zero build step
- **Single binary** — `coalbox-web` with embedded frontend

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
