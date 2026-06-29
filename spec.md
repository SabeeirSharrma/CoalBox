# Coalbox — Password Manager
# The Cinder Project

**Status:** Planning
**Version:** 0.1-spec
**Maintainer:** The Cinder Project
**Depends on:** Nothing (standalone), optionally integrated into Ember Browser

---

## 1. Philosophy

Coalbox exists because every major password manager today makes one of two compromises: it is closed source, or it locks your vault to a proprietary cloud service you don't control. Both are unacceptable.

Coalbox is:

- **Local-first** — your vault lives on your machine. No account required, no server required, no internet required to use it.
- **Open format** — the `.emberkeys` vault format is fully documented and open. Any tool can implement support for it. You are never locked in.
- **Sync-optional** — if you want your vault on multiple devices, you sync the `.emberkeys` file to any cloud service you already trust (Google Drive, OneDrive, Nextcloud, a USB stick, anything). Coalbox does not operate any sync infrastructure.
- **Auditable** — fully open source, compiled from source, distributed via CPAC.

Coalbox works standalone on any system. It is also integrated natively into Ember Browser as a first-class built-in component.

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                     Coalbox                         │
│                                                     │
│  ┌───────────────────────────────────────────────┐  │
│  │                Coalbox UI                    │  │
│  │  Standalone app (GTK) or Ember-native panel  │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
│  ┌───────────────────────────────────────────────┐  │
│  │              Coalbox Core (Rust)              │  │
│  │  Vault read/write, encryption, autofill API,  │  │
│  │  password generation, breach checking         │  │
│  └───────────────────────────────────────────────┘  │
│                                                     │
│  ┌───────────────────────────────────────────────┐  │
│  │           .emberkeys Vault File               │  │
│  │  AES-256-GCM encrypted, open format,          │  │
│  │  portable, sync-agnostic                      │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### 2.1 Coalbox Core

Written in Rust. Handles all vault operations:

- Vault creation, unlock, lock
- Entry CRUD (create, read, update, delete)
- AES-256-GCM encryption/decryption
- Argon2id key derivation from master password
- Password generation
- Autofill API (exposed to Ember Shell via IPC)
- Breach database lookup (HaveIBeenPwned k-anonymity API — only a hash prefix is sent, never the full password)

### 2.2 Coalbox UI

Two modes:

**Standalone** — a GTK application for use outside of Ember. Works on any Linux desktop.

**Ember-native** — a sidebar panel and toolbar button integrated directly into Ember Shell. No separate window needed when using Ember. Autofill triggers inline without leaving the page.

### 2.3 .emberkeys Vault Format

The vault is a single encrypted file with the `.emberkeys` extension. The format is open, versioned, and fully documented.

```
[4 bytes]   Magic: 0x454D424B ("EMBK")
[2 bytes]   Format version (currently 0x0001)
[16 bytes]  Argon2id salt
[12 bytes]  AES-256-GCM nonce
[4 bytes]   Encrypted payload length
[N bytes]   Encrypted payload (JSON)
[16 bytes]  AES-256-GCM authentication tag
```

The decrypted payload is a JSON object:

```json
{
  "version": 1,
  "created": "2025-01-01T00:00:00Z",
  "modified": "2025-01-01T00:00:00Z",
  "entries": [
    {
      "id": "uuid-v4",
      "type": "login",
      "title": "Example",
      "url": "https://example.com",
      "username": "user@example.com",
      "password": "...",
      "totp_secret": "...",
      "notes": "...",
      "tags": ["work"],
      "created": "2025-01-01T00:00:00Z",
      "modified": "2025-01-01T00:00:00Z"
    }
  ]
}
```

Entry types: `login`, `note`, `card`, `identity`.

The format spec is published separately as `EMBERKEYS_FORMAT.md` in the Coalbox repository. Third-party implementations are encouraged.

---

## 3. Features

### 3.1 Vault Management

- Create a new vault (`.emberkeys` file at any path)
- Open an existing vault from any path
- Lock vault (clears decrypted data from memory)
- Auto-lock after configurable inactivity timeout
- Change master password
- Export vault to plaintext JSON (for migration)
- Import from common formats: KeePass XML, Bitwarden JSON, 1Password 1PUX, CSV

### 3.2 Entry Management

- Add, edit, delete entries
- Entry types: Login, Secure Note, Payment Card, Identity
- Custom fields per entry (text, hidden, URL, date)
- Tags for organisation
- Favourites
- Search across all entries (title, URL, username, notes)
- Entry history — previous versions of a password are retained

### 3.3 Password Generator

Configurable password generator:

```
Length:        8 – 128 characters
Uppercase:     on/off
Lowercase:     on/off
Numbers:       on/off
Symbols:       on/off, custom symbol set
Exclude chars: user-defined exclusion list
Passphrase:    wordlist-based (EFF large wordlist)
  - Word count: 3 – 10
  - Separator:  custom character
  - Capitalise: on/off
  - Include number: on/off
```

### 3.4 TOTP (Two-Factor Authentication)

Coalbox stores TOTP secrets and generates 2FA codes natively. No separate authenticator app needed.

- Store TOTP secret per entry (manual entry or QR code scan)
- Generate current TOTP code with countdown timer
- Autofill TOTP code in Ember alongside username/password

### 3.5 Autofill (Ember Integration)

When integrated into Ember Browser:

- Detects login forms on page load
- Suggests matching entries from the vault via a non-intrusive toolbar prompt
- Fills username + password + TOTP in one action
- Keyboard shortcut to trigger autofill (remappable in `ember.toml`)
- Per-site autofill disable option
- Never autofills on HTTP (HTTPS only)

### 3.6 Breach Checking

Checks passwords against the HaveIBeenPwned Pwned Passwords database using the k-anonymity model:

- Only the first 5 characters of the SHA-1 hash of the password are sent to the API
- The full password never leaves the device
- Can be triggered manually per-entry or as a full vault audit
- Breached entries are flagged in the UI with a warning

Breach checking requires internet. It can be disabled entirely in settings for air-gapped use.

### 3.7 Vault Sync

Coalbox does not operate any sync server. Sync is the user's responsibility.

The recommended workflow:

1. Store the `.emberkeys` file in a synced folder (Google Drive, OneDrive, Nextcloud, Syncthing, etc.)
2. Point Coalbox at that path
3. The cloud service handles sync across devices

Coalbox handles **sync conflicts** gracefully: if two versions of the vault file exist (e.g. from simultaneous edits on two devices), Coalbox detects the conflict on open and presents a merge UI showing differing entries, letting the user choose which version to keep per-entry.

### 3.8 Standalone CLI

A command-line interface for power users and scripting:

```bash
coalbox unlock ~/vault.emberkeys        # unlock vault, keep unlocked in daemon mode
coalbox get example.com                 # retrieve entry by URL or title
coalbox add                             # interactive entry creation
coalbox generate --length 32 --symbols  # generate a password
coalbox audit                           # run breach check on all entries
coalbox export --format json            # export to plaintext JSON
coalbox lock                            # lock the vault daemon
```

---

## 4. Security Model

### 4.1 Encryption

- **Cipher:** AES-256-GCM (authenticated encryption)
- **Key derivation:** Argon2id with tuned parameters (memory: 64MB, iterations: 3, parallelism: 4)
- **Salt:** 16 bytes, randomly generated per vault
- **Nonce:** 12 bytes, randomly generated per save operation
- **Master password:** never stored, never written to disk, zeroed from memory on lock

### 4.2 Memory Security

- Decrypted vault data is held in memory only while unlocked
- Memory is explicitly zeroed on lock using Rust's `zeroize` crate
- The process does not swap vault data to disk (mlock where supported)

### 4.3 Auto-lock

Vault auto-locks after configurable inactivity:

```toml
[coalbox]
auto_lock_minutes = 5    # 0 = never auto-lock
lock_on_suspend = true   # lock when system suspends
```

### 4.4 Clipboard Security

When a password is copied to clipboard:

- Clipboard is cleared automatically after 30 seconds (configurable)
- Coalbox does not retain clipboard history

### 4.5 What Coalbox Never Does

- Never sends vault data, passwords, or master password to any server
- Never phones home for analytics, telemetry, or licensing
- Never requires an account or registration
- Never stores the master password or a derivative of it

---

## 5. Configuration

Coalbox configuration lives in `ember.toml` when running inside Ember, or in its own config file when standalone:

```
~/.config/coalbox/coalbox.toml
```

```toml
[coalbox]
vault_path = "~/.local/share/coalbox/vault.emberkeys"
auto_lock_minutes = 5
lock_on_suspend = true
clipboard_clear_seconds = 30
breach_check_enabled = true
autofill_enabled = true          # Ember integration only
autofill_https_only = true       # Ember integration only
generator_default_length = 20
generator_default_symbols = true
```

---

## 6. Ember Shell Integration

When Coalbox is running inside Ember:

- **Toolbar button** — lock/unlock indicator, click to open vault panel
- **Sidebar panel** — full vault UI accessible without leaving the browser
- **Autofill prompt** — non-intrusive bar below the address bar when a login form is detected
- **Address bar indicator** — padlock icon indicates saved credentials exist for current site
- **New credential prompt** — offers to save credentials after a successful login is detected
- **TOTP autofill** — fills 2FA code alongside username/password in one action
- **Generator inline** — password generator accessible from any password field via right-click

Coalbox in Ember communicates with Coalbox Core via IPC. The vault process is separate from the browser process — a browser crash does not corrupt the vault.

---

## 7. Standalone Distribution

Coalbox is distributed two ways:

- **Via CPAC** — source build, same as all Cinder Project software
- **Standalone** — source tarball, buildable independently of Ember

The Coalbox Core (Rust) and CLI build without any Ember dependency. The GTK standalone UI builds without any Ember dependency. Only the Ember Shell integration component requires Ember.

---

## 8. Versioned Roadmap

### v0.1 — Core Vault
- `.emberkeys` format defined and documented
- Vault create, unlock, lock
- Entry CRUD (login, note)
- AES-256-GCM encryption
- Argon2id key derivation
- Basic GTK standalone UI
- CPAC package definition

### v0.2 — Full Entry Types + Generator
- Payment card and identity entry types
- Custom fields
- Tags and favourites
- Password generator (character + passphrase modes)
- Entry history

### v0.3 — TOTP + Breach Check
- TOTP secret storage and code generation
- HaveIBeenPwned breach checking (k-anonymity)
- Full vault audit

### v0.4 — Import / Export
- Import: KeePass XML, Bitwarden JSON, 1Password 1PUX, CSV
- Export: plaintext JSON
- Conflict merge UI for sync conflicts

### v0.5 — CLI
- Full `coalbox` CLI
- Daemon mode for unlock persistence
- Scripting-friendly output (JSON flags)

### v0.6 — Ember Integration
- Toolbar button and sidebar panel
- Autofill (username + password + TOTP)
- New credential save prompt
- Address bar indicator
- Generator inline from password fields

### v0.7 — Security Hardening
- mlock for vault memory
- Clipboard auto-clear
- Lock on suspend
- Security audit pass

### v0.8 — Polish + Docs
- Full user documentation
- `.emberkeys` format spec published
- Bug fix sprint

### v1.0 — Stable Release
- All features stable and documented
- CPAC package stable
- Format spec v1.0 frozen

---

## 9. Future Considerations

- **Browser extension** — a standalone Coalbox extension (`.crx`) for use in non-Ember browsers, sideloadable
- **Android app** — mobile vault access, post-v1.0
- **SSH key storage** — store SSH private keys in the vault, integrate with ssh-agent
- **Hardware key support** — YubiKey / FIDO2 as second factor to unlock vault (alongside or instead of master password)
- **Ember Sync integration** — when Ember Sync ships post-v1.0, Coalbox vault sync can be optionally routed through it

---

## 10. Project Info

**Repository:** TBD (The Cinder Project org)
**Language:** Rust (core, CLI), TypeScript (Ember panel UI), C/GTK (standalone UI)
**License:** TBD (likely GPL-3.0 or MIT)
**Part of:** The Cinder Project
**Integrated into:** Ember Browser (v0.6+)
**Related projects:** CPAC (distribution), Ember Browser
