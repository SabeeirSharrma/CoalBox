# v0.6.4 — Update System

## What's new

- **`coalbox update`** — check for updates and build from source
- **Update checker** — checks GitHub releases API on CLI and WebUI
- **WebUI update button** — shows when update is available, one-click update
- **CLI update prompt** — `coalbox update` shows release notes and prompts before updating
- **`coalbox update --yes`** — skip confirmation prompt

## CLI Usage

```bash
# Check for updates
coalbox update

# Update without confirmation
coalbox update --yes
```

## WebUI

- Green arrow icon appears in header when update is available
- Click to see release notes and confirm update
- Update builds from source and installs to same location

## Build from source

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
```

Requires Rust 1.85+.
