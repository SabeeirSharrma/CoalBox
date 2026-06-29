---
title: Introduction
description: Overview of the Coalbox password manager.
order: 1
---

# Coalbox

**Local-first, open-format password manager.** Part of [The Cinder Project](https://thecinderproject.qd.je/).

Your vault lives on your machine. No account, no server, no internet required. The `.emberkeys` format is fully documented and open — you are never locked in.

## What is Coalbox?

Coalbox is a password manager that runs entirely on your local machine. It stores your credentials in a single encrypted `.emberkeys` file that you control. No cloud service, no subscription, no vendor lock-in.

## Principles

- **No trackers. No telemetry. No accounts. No BS.** Coalbox never phones home. Ever.
- **Local-first.** Your vault is a single encrypted file you control. Sync it however you want.
- **Open format.** `.emberkeys` is fully documented. Any tool can read it.
- **Build from source.** Transparency means you verify what you run.

## Features

### Vault Management

- Create, unlock, lock vaults
- AES-256-GCM encryption with Argon2id key derivation
- Auto-lock after configurable inactivity

### Entry Types

- **Login** — username, password, URL, TOTP
- **Authenticator** — standalone 2FA codes, separated from passwords
- **Secure Note** — freeform text
- **Payment Card** — cardholder, number, expiry, CVV, PIN
- **Identity** — name, email, phone, address

### Password Generator

- **Character mode** — configurable length, uppercase, lowercase, numbers, symbols
- **Passphrase mode** — EFF large wordlist (7776 words), configurable word count, separator, capitalization

### Import / Export

- Import from CSV, Bitwarden JSON, KeePass XML, 1Password 1PUX
- Export to plaintext JSON
- Auto-format detection, duplicate detection

### Entry Management

- Custom fields (text, hidden, URL, date)
- Tags and favourites
- Search across all entries
- Password history (previous versions retained)

### CLI & Scripting

- JSON output for all commands (`--json`)
- Quiet mode (`--quiet`) for minimal output
- Field extraction (`get --field password`)
- Entry filtering (`list --tag work --type login`)
- Consistent exit codes (0 success, 1 error)

### TOTP & Security

- TOTP code generation (RFC 6238, SHA-1/SHA-256)
- Breach checking via HaveIBeenPwned (k-anonymity)
- Vault audit for compromised passwords

### WebUI

- beautifully redesigned localhost web interface (axum, vanilla JS, Tailwind CSS, Lucide icons)
- Full vault management via browser
- Dedicated tab for Authenticator codes
- Password generator with live preview
- TOTP display with auto-refreshing animated countdown timers
- WebSocket real-time state

## Quick Start

```bash
# Create a vault
coalbox create ~/vault.emberkeys

# Generate a password
coalbox generate -l 32

# Generate a passphrase
coalbox generate --passphrase --words 6

# Import from Bitwarden
coalbox import ~/bitwarden.json -f bitwarden -v ~/vault.emberkeys

# List entries
coalbox list -v ~/vault.emberkeys

# Export vault
coalbox export ~/backup.json -v ~/vault.emberkeys

# Start the WebUI
coalbox-web --vault ~/vault.emberkeys
```

## Requirements

- Rust 1.85+ (for building)
- No runtime dependencies beyond glibc

## Related Projects

| Project | Role |
| --- | --- |
| [cpac](https://github.com/SabeeirSharrma/cpac) | Package trust layer (distributes Coalbox) |
| [ember-browser](https://github.com/SabeeirSharrma/ember-browser) | Browser with native Coalbox integration |

## Made By

**Developer/Maintainer:** [Sabeeir Sharrma](https://github.com/SabeeirSharrma)

**Made under [The Cinder Project](https://thecinderproject.qd.je/)** — *Burn all the Blind Spots*
