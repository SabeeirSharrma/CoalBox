# v0.6.5 — Uninstall + Vault Destroy

## What's new

- **`uninstall.sh`** — removes coalbox binaries, preserves vault data
- **`coalbox destroy`** — permanently delete a vault file
- **WebUI vault delete** — delete vault from the Vault Info modal
- **Confirmation prompts** — both CLI and WebUI require confirmation before destroying vaults

## CLI Usage

```bash
# Uninstall coalbox (keeps vault)
bash uninstall.sh

# Delete a vault (requires typing vault name to confirm)
coalbox destroy

# Delete without confirmation (dangerous!)
coalbox destroy --yes
```

## WebUI

- "Delete Vault" button in Vault Info modal
- Requires typing vault name to confirm
- Vault is permanently deleted, all entries lost

## Build from source

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
```

Requires Rust 1.85+.
