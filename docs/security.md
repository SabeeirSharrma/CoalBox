---
title: Security
description: Coalbox security model and design decisions.
order: 6
---

# Security Model

Coalbox is designed with security as a core principle. This document explains the security model and design decisions.

## Encryption

- **Cipher:** AES-256-GCM (authenticated encryption)
- **Key derivation:** Argon2id with tuned parameters:
  - Memory: 64MB
  - Iterations: 3
  - Parallelism: 4
  - Output: 32 bytes
- **Salt:** 16 bytes, randomly generated per vault
- **Nonce:** 12 bytes, randomly generated per save operation
- **Master password:** never stored, never written to disk, zeroed from memory on lock

## Memory Security

- Decrypted vault data is held in memory only while unlocked
- Memory is explicitly zeroed on lock using Rust's `zeroize` crate
- The process does not swap vault data to disk (mlock where supported)

## Auto-lock

Vault auto-locks after configurable inactivity:

```toml
[coalbox]
auto_lock_minutes = 5    # 0 = never auto-lock
lock_on_suspend = true   # lock when system suspends
```

## Clipboard Security

When a password is copied to clipboard:

- Clipboard is cleared automatically after 30 seconds (configurable)
- Coalbox does not retain clipboard history

## Breach Checking

Checks passwords against the HaveIBeenPwned Pwned Passwords database using the k-anonymity model:

- Only the first 5 characters of the SHA-1 hash of the password are sent to the API
- The full password never leaves the device
- Can be disabled entirely in settings for air-gapped use

## What Coalbox Never Does

- Never sends vault data, passwords, or master password to any server
- Never phones home for analytics, telemetry, or licensing
- Never requires an account or registration
- Never stores the master password or a derivative of it

## Build Verification

Coalbox is open source and distributed via source. The recommended way to install is building from source:

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
```

Pre-built binaries are provided for checksum verification only. See [Installation](installation.md) for details.
