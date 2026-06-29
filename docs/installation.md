---
title: Installation
description: How to install Coalbox.
order: 2
---

# Installation

Coalbox runs on any system with Rust installed. No external dependencies required.

## Build from Source (Recommended)

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
# coalbox 0.2.0
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
