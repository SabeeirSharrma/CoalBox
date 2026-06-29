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
- **Secure Note** — freeform text
- **Payment Card** — cardholder, number, expiry, CVV, PIN
- **Identity** — name, email, phone, address

### Password Generator

- **Character mode** — configurable length, uppercase, lowercase, numbers, symbols
- **Passphrase mode** — EFF large wordlist (7776 words), configurable word count, separator, capitalization

### Entry Management

- Custom fields (text, hidden, URL, date)
- Tags and favourites
- Search across all entries
- Password history (previous versions retained)

## Quick Start

```bash
# Create a vault
coalbox create ~/vault.emberkeys

# Generate a password
coalbox generate -l 32

# Generate a passphrase
coalbox generate --passphrase --words 6

# List entries
coalbox list -v ~/vault.emberkeys
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
