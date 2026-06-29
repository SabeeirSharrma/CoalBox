---
title: Installation
description: How to install Coalbox.
order: 2
---

# Installation

Coalbox runs on any system with Rust installed. No external dependencies required.

## Quick Install (Recommended)

One command to build and install coalbox from source:

```bash
curl -sSf https://thecinderproject.qd.je/coalbox/install.sh | bash
```

This script will:
- Detect if Rust is already installed on your system
- If not, install Rust temporarily just for building
- Clone and build coalbox from source
- Install the binary to `/usr/local/bin`
- If Rust was not present before, **automatically remove it** after installation

No dependencies are left behind. The script is fully transparent — you can review it before running:

```bash
curl -sSf https://thecinderproject.qd.je/coalbox/install.sh -o install.sh
less install.sh
bash install.sh
```

## Build from Source

If you prefer to build manually:

```bash
git clone https://github.com/SabeeirSharrma/coalbox.git
cd coalbox
cargo build --release
```

The binary will be at `target/release/coalbox`.

### Optional: Install to PATH

```bash
sudo cp target/release/coalbox /usr/local/bin/coalbox
```

## Verify Installation

```bash
coalbox --version
# coalbox 0.3.0
```

## Pre-built Binaries

GitHub Releases include pre-built binaries for x86_64 and aarch64 Linux.

> **These binaries are provided for SHA-256 verification only — do not download or use them directly.**
> Coalbox must be built from source to ensure transparency and reproducibility.

## Requirements

- Rust 1.85+ (edition 2024)
- No runtime dependencies beyond glibc

## Platform Support

| Platform | Status |
| --- | --- |
| x86_64 Linux | Supported |
| aarch64 Linux | Supported |
| Other platforms | Build from source |
