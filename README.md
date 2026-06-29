# Coalbox

A local-first, open-format password manager. Part of [The Cinder Project](https://thecinderproject.qd.je/).

Your vault lives on your machine. No account, no server, no internet required. The `.emberkeys` format is fully documented and open — you are never locked in.

## Principles

- **No trackers. No telemetry. No accounts. No BS.** Coalbox never phones home. Ever.
- **Local-first.** Your vault is a single encrypted file you control. Sync it however you want.
- **Open format.** `.emberkeys` is fully documented. Any tool can read it.
- **Build from source.** Transparency means you verify what you run.

## Install

**Recommended — build from source:**

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
# binary at target/release/coalbox
```

Requires Rust 1.85+ (edition 2024).

## Releases

GitHub Releases include pre-built binaries for x86_64 and aarch64 Linux alongside SHA-256 checksums.

> **These binaries are provided for checksum verification only — do not download or use them directly.**
> Coalbox must be built from source to ensure transparency and reproducibility.

## Usage

```bash
# Create a new vault
coalbox create ~/vault.emberkeys

# List all entries
coalbox list -v ~/vault.emberkeys

# Get an entry by title or URL
coalbox get github -v ~/vault.emberkeys

# Generate a password
coalbox generate -l 32

# Generate a passphrase
coalbox generate --passphrase --words 6

# Show vault info
coalbox info -v ~/vault.emberkeys
```

## Features

- **Login entries** — username, password, URL, TOTP
- **Secure Notes** — freeform encrypted text
- **Payment Cards** — cardholder, number (masked), expiry, CVV, PIN
- **Identity entries** — name, email, phone, full address
- **Custom fields** — text, hidden, URL, date types
- **Tags & favourites** — organize and highlight entries
- **Password generator** — character mode with configurable options
- **Passphrase generator** — EFF wordlist (7776 words), configurable word count
- **Password history** — previous passwords retained on update

## Vault Format

The `.emberkeys` format is a single encrypted file:

```
[4 bytes]   Magic: EMBK
[2 bytes]   Format version
[16 bytes]  Argon2id salt
[12 bytes]  AES-256-GCM nonce
[4 bytes]   Encrypted payload length
[N bytes]   Encrypted JSON payload
[16 bytes]  AES-256-GCM auth tag
```

Encryption: AES-256-GCM. Key derivation: Argon2id (64MB, 3 iterations, 4 parallelism).

See [docs/vault-format.md](docs/vault-format.md) for the full specification. Third-party implementations are encouraged.

## Documentation

- [Installation](docs/installation.md)
- [Configuration](docs/configuration.md)
- [Vault Format](docs/vault-format.md)
- [CLI Reference](docs/cli-reference.md)
- [Security](docs/security.md)

## Security

- Master password is never stored or written to disk
- Decrypted data is zeroed from memory on lock (`zeroize` crate)
- Only the first 5 chars of a SHA-1 hash are sent for breach checks (k-anonymity) — full password never leaves your machine
- No analytics, no telemetry, no phone-home

If you discover a security vulnerability, please **do not** open a public issue. Contact the maintainers directly (see [CONTRIBUTING.md](CONTRIBUTING.md)).

## Requirements

- Rust 1.85+ (for building)
- No runtime dependencies beyond glibc

## Made By

**Developer/Maintainer: [Sabeeir Sharrma](https://github.com/SabeeirSharrma)**

**Made under [The Cinder Project — Burn all the Blind Spots](https://thecinderproject.qd.je/)**
