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

## Global Options

| Option | Description |
| --- | --- |
| `-h, --help` | Print help |
| `-V, --version` | Print version |
