---
title: Configuration
description: Coalbox configuration options.
order: 3
---

# Configuration

Coalbox uses a TOML configuration file.

## Config Location

- **Standalone:** `~/.config/coalbox/coalbox.toml`
- **Ember integration:** `ember.toml` (under `[coalbox]` section)

## Options

```toml
[coalbox]
# Path to vault file
vault_path = "~/.local/share/coalbox/vault.emberkeys"

# Auto-lock after N minutes of inactivity (0 = never)
auto_lock_minutes = 5

# Lock when system suspends
lock_on_suspend = true

# Clear clipboard after N seconds
clipboard_clear_seconds = 30

# Enable breach checking (requires internet)
breach_check_enabled = true

# Enable autofill (Ember integration only)
autofill_enabled = true

# Only autofill on HTTPS (Ember integration only)
autofill_https_only = true

# Default password generator length
generator_default_length = 20

# Default: include symbols in generator
generator_default_symbols = true
```

## Defaults

If no config file exists, Coalbox uses these defaults:

| Setting | Default |
| --- | --- |
| `vault_path` | `~/.local/share/coalbox/vault.emberkeys` |
| `auto_lock_minutes` | `5` |
| `lock_on_suspend` | `true` |
| `clipboard_clear_seconds` | `30` |
| `breach_check_enabled` | `true` |
| `autofill_enabled` | `true` |
| `autofill_https_only` | `true` |
| `generator_default_length` | `20` |
| `generator_default_symbols` | `true` |
