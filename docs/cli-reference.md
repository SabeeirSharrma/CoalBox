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
coalbox get <QUERY> [-v <VAULT>]
```

| Argument | Description |
| --- | --- |
| `QUERY` | Entry title, URL, or search term |
| `-v, --vault` | Vault file path |

**Example:**

```bash
coalbox get github -v ~/vault.emberkeys
```

---

### list

List all entries in the vault.

```bash
coalbox list [-v <VAULT>]
```

| Option | Description |
| --- | --- |
| `-v, --vault` | Vault file path |

**Example:**

```bash
coalbox list -v ~/vault.emberkeys
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

## Global Options

| Option | Description |
| --- | --- |
| `-h, --help` | Print help |
| `-V, --version` | Print version |
