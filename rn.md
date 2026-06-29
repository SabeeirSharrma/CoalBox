# v0.5.0 — CLI

## What's new

- **JSON output** — `--json` flag for all commands, machine-readable format
- **Quiet mode** — `--quiet` / `-q` suppresses non-essential output
- **Field extraction** — `get --field password` to grab a single value
- **Entry filtering** — `list --tag work --type login`
- **Consistent exit codes** — 0 on success, 1 on error

## What's not in this release

- WebUI (v0.6)
- Ember Browser integration (v0.7)

## Build from source

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
```

Requires Rust 1.85+.

## Checksums

Pre-built binaries are for SHA-256 verification only. Do not use them directly.
