# Coalbox — Password Manager

# The Cinder Project

**Status:** In Development
**Version:** 0.2.0
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
│  │                Coalbox UI                     │  │
│  │  Standalone (GTK), Ember-native panel,        │  │
│  │  or local WebUI (browser-based)               │  │
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

Three modes:

**Standalone** — a GTK application for use outside of Ember. Works on any Linux desktop.

**WebUI** — a local web server that serves a browser-based interface. Binds to `127.0.0.1` only. Access from any modern browser on the same machine. No external network access.

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

Entry types: `login`, `note`, `card`, `identity`, `authenticator`.

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

### 3.7 Migration (Export to Other Formats)

Coalbox does not support plaintext export. Vault encryption (AES-256-GCM + Argon2id) is strong enough that the actual weak point in any export flow is the moment decrypted data touches disk unencrypted — even briefly, even as a "temp file we delete after." That moment is the attack surface, not the vault itself.

Instead, Coalbox provides `coalbox migrate`, a direct migration tool. Decrypted vault data exists only in memory for the duration of the migration and is zeroed immediately after (same zeroize discipline as the rest of Coalbox Core). No intermediate plaintext file is ever written, not even temporarily.

**Targets:**

- **KDBX (KeePass format)** — the primary target. KDBX is itself an encrypted container (AES/ChaCha20 + Argon2), so data leaving Coalbox stays encrypted the entire time, just under a different format. KDBX functions as the universal bridge: KeePass, KeePassXC, 1Password, and most other major password managers can import `.kdbx` directly. This means Coalbox only needs to maintain one migration path to cover nearly every destination.
- **Bitwarden encrypted export** — a secondary target for users moving specifically to Bitwarden, since Bitwarden's importer prefers its own encrypted export format over KDBX.

1Password's native export format (1PUX) is deliberately not supported as a direct target, since it isn't encrypted at rest by default. Anyone migrating to 1Password goes through the KDBX path, which 1Password imports natively.

**Flow:**

```bash
coalbox migrate --to kdbx --output ~/vault.kdbx
coalbox migrate --to bitwarden --output ~/export.json
```

Each command prompts for the Coalbox master password to unlock the vault, then a separate export password to protect the new file — the export password is never the same as the master password, and is never reused or stored. Entries are streamed directly into the target format's encrypted writer as the vault is read, so nothing decrypted is ever held longer than necessary or written anywhere unencrypted.

**WebUI support:**

Migration is also available through the `coalbox-web` interface. The WebUI exposes a Migrate panel where the user selects a target format (KDBX or Bitwarden), is prompted for the Coalbox master password (vault must be unlocked) and a separate export password through the same form, and the resulting file is generated server-side and offered as a direct browser download.

The same in-memory-only guarantee applies regardless of entry point — whether triggered via CLI or WebUI, decrypted vault data is never written to disk unencrypted, and the generated file is only ever the final encrypted output (`.kdbx` or Bitwarden's encrypted export), streamed straight to the response/download rather than staged as a temp file on the server.

### 3.8 Vault Sync

Coalbox does not operate any sync server. Sync is the user's responsibility.

The recommended workflow:

1. Store the `.emberkeys` file in a synced folder named `CoalBox_sync` (Google Drive, OneDrive, Nextcloud, Syncthing, etc.)
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
coalbox migrate --to kdbx --output ~/vault.kdbx  # migrate to KDBX
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

## 7. WebUI (Ease of Access)

For users who want a graphical interface without installing Ember or using the CLI, Coalbox includes an optional WebUI. As lite as possible — no build step, no bundler, no framework.

### 7.1 Tech Stack

- **Server:** [axum](https://github.com/tokio-rs/axum) — lightweight async Rust web framework
- **Frontend:** Vanilla HTML/CSS/JS — zero build step, zero dependencies
- **Styling:** Tailwind CSS (via CDN) + Lucide Icons — no local install, no purge step
- **State:** WebSocket for real-time vault state
- **Binary:** Single `coalbox-web` executable, frontend served from memory

### 7.2 Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Coalbox WebUI                     │
│                                                     │
│  ┌───────────────────────────────────────────────┐  │
│  │           axum (Rust, localhost)              │  │
│  │  Serves HTML/JS, WebSocket, REST endpoints    │  │
│  └───────────────────────────────────────────────┘  │
│                        │                            │
│                        ▼                            │
│  ┌───────────────────────────────────────────────┐  │
│  │              Coalbox Core (Rust)              │  │
│  │  Same library used by CLI and Ember           │  │
│  └───────────────────────────────────────────────┘  │
│                        │                            │
│                        ▼                            │
│  ┌───────────────────────────────────────────────┐  │
│  │           .emberkeys Vault File               │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### 7.3 Design

- **Local only** — binds to `127.0.0.1` with a random port. No external network access.
- **No cloud, no accounts** — same principles as the CLI.
- **No build step** — frontend is raw HTML/CSS/JS files. Tailwind via `<script src="https://cdn.tailwindcss.com">`. No npm, no webpack, no vite.
- **Minimal JS** — vanilla `fetch()` for API calls, native `WebSocket` for state. No React, no Alpine, no framework.
- **Embedded assets** — HTML/JS served from the Rust binary via `axum::Router`. No filesystem serving.
- **Single binary** — `coalbox-web` contains the server, frontend, and coalbox-core.

### 7.4 API

```
POST   /api/unlock          # unlock vault with master password
POST   /api/lock            # lock vault
GET    /api/status          # lock state, entry count
GET    /api/entries         # list all entries
GET    /api/entries/:id     # get single entry
POST   /api/entries         # create entry
PUT    /api/entries/:id     # update entry
DELETE /api/entries/:id     # delete entry
GET    /api/search?q=       # search entries
POST   /api/generate        # generate password
WS     /ws                  # real-time state (lock/unlock notifications)
```

### 7.5 Features

- Full vault management (add, edit, delete entries)
- Search across all entries
- Password generator with live preview
- TOTP code display with countdown timer
- Copy password/TOTP to clipboard (auto-clear after 30s)
- Vault lock/unlock from the browser
- Entry import/export

### 7.6 Usage

```bash
# Start the WebUI
coalbox-web --vault ~/vault.emberkeys

# Opens at http://127.0.0.1:<random-port>
# Vault unlocks via the web interface
# Close browser tab or Ctrl+C to stop
```

### 7.7 Security

- Server binds to localhost only — no remote access possible
- No CORS headers for external origins
- Vault data never leaves the process memory
- Auto-locks when the WebUI process is stopped
- Same encryption and key derivation as CLI/Ember
- Tailwind loaded from CDN is styling-only, no JS execution

---

## 8. Standalone Distribution

Coalbox is distributed two ways:

- **Via CPAC** — source build, same as all Cinder Project software
- **Standalone** — source tarball, buildable independently of Ember

The Coalbox Core (Rust) and CLI build without any Ember dependency. The GTK standalone UI and WebUI build without any Ember dependency. Only the Ember Shell integration component requires Ember.

---

## 9. Versioned Roadmap

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

### v0.6 — WebUI

- Local web server (localhost only)
- Full vault management via browser
- Password generator with live preview
- TOTP display with countdown
- WebSocket real-time state
- Single `coalbox-web` binary

### v0.7 — Ember Integration

- Toolbar button and sidebar panel
- Autofill (username + password + TOTP)
- New credential save prompt
- Address bar indicator
- Generator inline from password fields

### v0.8 — Security Hardening

- mlock for vault memory
- Clipboard auto-clear
- Lock on suspend
- Security audit pass

### v0.9 — Polish + Docs

- Full user documentation
- `.emberkeys` format spec published
- Bug fix sprint

### v1.0 — Stable Release

- All features stable and documented
- CPAC package stable
- Format spec v1.0 frozen

### v1.1 — Android

- Native Android app (Kotlin + coalbox-core via FFI)
- Vault access on mobile
- Autofill via Android Autofill Framework
- Biometric unlock (fingerprint/face)
- QR code scanning for TOTP setup
- Same `.emberkeys` format — sync via any cloud service

---

## 10. Future Considerations

- **Browser extension** — a standalone Coalbox extension (`.crx`) for use in non-Ember browsers, sideloadable
- **SSH key storage** — store SSH private keys in the vault, integrate with ssh-agent
- **Hardware key support** — YubiKey / FIDO2 as second factor to unlock vault (alongside or instead of master password)
- **Ember Sync integration** — when Ember Sync ships post-v1.0, Coalbox vault sync can be optionally routed through it

---

## 11. Project Info

**Repository:** TBD (The Cinder Project org)
**Language:** Rust (core, CLI, WebUI), TypeScript (Ember panel UI), C/GTK (standalone UI), Kotlin (Android)
**License:** TBD (likely GPL-3.0 or MIT)
**Part of:** The Cinder Project
**Integrated into:** Ember Browser (v0.6+)
**Related projects:** CPAC (distribution), Ember Browser
