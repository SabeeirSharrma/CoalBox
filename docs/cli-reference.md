---
title: CLI Reference
description: Coalbox command-line interface reference.
order: 5
---

# CLI Reference

## Usage

```bash
coalbox <COMMAND> [OPTIONS]
```

## Commands

### create

Create a new vault.

```bash
coalbox create [PATH]
```

| Argument | Default | Description |
| --- | --- | --- |
| `PATH` | `~/.local/share/coalbox/vault.emberkeys` | Path for the new vault file |

**Example:**

```bash
coalbox create ~/my-vault.emberkeys
```

---

### get

Get an entry by title or URL.

```bash
coalbox get <QUERY> [-v <VAULT>] [-f <FIELD>]
```

| Argument | Description |
| --- | --- |
| `QUERY` | Entry title, URL, or search term |
| `-v, --vault` | Vault file path |
| `-f, --field` | Extract a single field (title, username, password, url, totp, notes) |

**Examples:**

```bash
# Full entry display
coalbox get github -v ~/vault.emberkeys

# Just the password (great for scripting)
coalbox get github -f password

# JSON output for a single field
coalbox get github --field password --json
```

---

### list

List all entries in the vault.

```bash
coalbox list [-v <VAULT>] [-t <TAG>] [-T <TYPE>]
```

| Option | Description |
| --- | --- |
| `-v, --vault` | Vault file path |
| `-t, --tag` | Filter by tag |
| `-T, --type` | Filter by type (login, note, card, identity, authenticator) |

**Examples:**

```bash
# List all entries
coalbox list -v ~/vault.emberkeys

# List only login entries
coalbox list --type login

# List entries with a specific tag
coalbox list --tag work

# JSON output for scripting
coalbox list --json | jq '.[].title'
```

---

### generate

Generate a password or passphrase.

```bash
coalbox generate [OPTIONS]
```

**Character mode (default):**

| Option | Default | Description |
| --- | --- | --- |
| `-l, --length` | `20` | Password length |
| `--uppercase` | `true` | Include uppercase letters |
| `--lowercase` | `true` | Include lowercase letters |
| `--numbers` | `true` | Include numbers |
| `--symbols` | `true` | Include symbols |

**Passphrase mode:**

| Option | Default | Description |
| --- | --- | --- |
| `--passphrase` | `false` | Generate passphrase instead of password |
| `-w, --words` | `6` | Number of words |
| `-s, --separator` | `" "` | Separator between words |
| `--capitalize` | `true` | Capitalize first letter of each word |
| `--number` | `false` | Append a random number |

**Examples:**

```bash
# Generate a 32-character password
coalbox generate -l 32

# Generate a 6-word passphrase
coalbox generate --passphrase

# Generate a 4-word passphrase with dashes, no caps
coalbox generate --passphrase -w 4 -s "-" --no-capitalize

# JSON output for scripting
coalbox generate -l 32 --json
# {"password":"aB3$kL9#mN2@pQ5&rS8!"}

coalbox generate --passphrase --json
# {"passphrase":"correct horse battery staple"}
```

---

### info

Show vault information.

```bash
coalbox info [-v <VAULT>]
```

| Option | Description |
| --- | --- |
| `-v, --vault` | Vault file path |

**Example:**

```bash
coalbox info -v ~/vault.emberkeys
```

---

### lock

Lock the vault (daemon mode). Not yet implemented.

```bash
coalbox lock
```

---

### totp

Show TOTP code for an entry.

```bash
coalbox totp <QUERY> [-v <VAULT>]
```

| Argument | Description |
| --- | --- |
| `QUERY` | Entry title, URL, or search term |
| `-v, --vault` | Vault file path |

**Example:**

```bash
coalbox totp github -v ~/vault.emberkeys
# GitHub
#   TOTP: 482903 (12s remaining)
```

---

### audit

Check all passwords in the vault against HaveIBeenPwned.

```bash
coalbox audit [-v <VAULT>]
```

| Option | Description |
| --- | --- |
| `-v, --vault` | Vault file path |

**Example:**

```bash
coalbox audit -v ~/vault.emberkeys
# Checking passwords against HaveIBeenPwned...
#
# Vault audit complete:
#   Total entries:       15
#   Entries with pass:   8
#
# ✓ No breached passwords found!
```

---

### check

Check a single password against HaveIBeenPwned.

```bash
coalbox check [PASSWORD]
```

| Argument | Description |
| --- | --- |
| `PASSWORD` | Password to check (or `-` to read from stdin) |

**Example:**

```bash
coalbox check "password123"
# ⚠ Password found in 12345 data breaches!
#   Do not use this password.
```

---

### import

Import entries from an external file.

```bash
coalbox import <FILE> [-f <FORMAT>] [-v <VAULT>]
```

| Argument | Default | Description |
| --- | --- | --- |
| `FILE` | — | File to import (required) |
| `-f, --format` | `auto` | Format: `csv`, `bitwarden`, `keepass`, `1password`, `auto` |
| `-v, --vault` | default | Vault file path |

**Supported formats:**

| Format | Extension | Description |
| --- | --- | --- |
| CSV | `.csv` | Flexible column mapping (name/title, user/username/login/email, pass/password/pwd, url/website/site, notes/note/comment) |
| Bitwarden JSON | `.json` | Full Bitwarden vault export |
| KeePass XML | `.xml` | KeePass XML export |
| 1Password 1PUX | `.1pux` | 1Password export archive |

**Examples:**

```bash
# Import from CSV (auto-detect format)
coalbox import ~/export.csv

# Import from Bitwarden JSON
coalbox import ~/bitwarden.json -f bitwarden

# Import into a specific vault
coalbox import ~/keepass.xml -f keepass -v ~/other-vault.emberkeys
```

---

### export

Export all entries to a plaintext JSON file.

```bash
coalbox export <FILE> [-v <VAULT>]
```

| Argument | Description |
| --- | --- |
| `FILE` | Output file path |
| `-v, --vault` | Vault file path |

**Example:**

```bash
# Export vault to JSON
coalbox export ~/backup.json

# Export from a specific vault
coalbox export ~/backup.json -v ~/other-vault.emberkeys
```

---

## Global Options

| Option | Description |
| --- | --- |
| `--json` | Output in JSON format (all commands) |
| `-q, --quiet` | Suppress non-essential output |
| `-h, --help` | Print help |
| `-V, --version` | Print version |
