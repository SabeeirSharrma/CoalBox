# v0.6.6 — Force Flag

## What's new

- **`--force` flag** on `coalbox update` — reinstall even if already on latest version

## CLI Usage

```bash
# Normal update (only if newer version available)
coalbox update

# Force reinstall (rebuild and reinstall current version)
coalbox update --force

# Force + skip confirmation
coalbox update --force --yes
```

## Build from source

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
```

Requires Rust 1.85+.
