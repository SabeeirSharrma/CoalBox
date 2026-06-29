# Contributing to Coalbox

Thank you for your interest in Coalbox.

## Contribution Policy

**At this time, we are not accepting unsolicited pull requests or direct commits to this repository.**

This repository is maintained strictly by the core developers/maintainers. Any commits, pull requests, or code modifications by individuals who have not been explicitly invited as collaborators or granted prior written permission will be **rejected and closed immediately**.

### Why this policy?

Coalbox is a security-focused password manager — its encryption, vault format, and key derivation are the parts of the codebase that users rely on to keep their credentials safe. An unreviewed external change to those areas isn't just a process risk, it's a real attack surface. Until Coalbox's contribution review process is more established, we're keeping the codebase strictly maintainer-controlled to protect that trust.

### How to get involved

If you're interested in contributing code, reach out to the project maintainers first — contact options are listed on our GitHub Pages site:

- **Owner / Maintainer / Main Developer:** [Sabeeir Sharrma](https://github.com/sabeeirsharrma)

Only after being officially invited as a collaborator, joining our organization, or receiving explicit permission from the core team may you begin submitting changes.

### Bug reports and suggestions

You don't need prior permission for this part — bug reports, feature suggestions, and general feedback are welcome through either:

- Opening a new [issue](https://github.com/SabeeirSharrma/coalbox/issues) on this repository
- Our [Discord server](https://discord.com/invite/3ZMtEgJjFT)

## Development

### Prerequisites

- Rust 1.85+ (stable, edition 2024)
- `cargo`, `rustc`

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Running tests

```bash
cargo test
```

### Linting

```bash
cargo clippy -- -W clippy::all
```

### Project structure

Coalbox uses a Cargo workspace with two crates:

```
coalbox/
├── Cargo.toml            # Workspace root
├── coalbox-core/         # Core library crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs        # Public API
│       ├── crypto.rs     # AES-256-GCM + Argon2id
│       ├── entry.rs      # Entry types (Login, Note, Card, Identity)
│       ├── error.rs      # Error types
│       ├── format.rs     # .emberkeys binary format
│       ├── generator.rs  # Password/passphrase generator
│       └── vault.rs      # Vault operations
├── coalbox/              # CLI binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs       # CLI entry point
├── docs/                 # Documentation
│   ├── index.md          # Overview
│   ├── installation.md   # Build instructions
│   ├── configuration.md  # Config options
│   ├── vault-format.md   # .emberkeys spec
│   ├── cli-reference.md  # CLI commands
│   └── security.md       # Security model
└── .github/workflows/    # CI/CD
```

**Why a workspace?** `coalbox-core` is designed as a standalone library. When Ember Browser integration ships at v0.7, it will depend on `coalbox-core` directly — without pulling in the CLI. Third-party tools can also implement `.emberkeys` support by depending on just the core crate.

## Security

If you discover a security vulnerability, please **do not** open a public issue. Instead, contact the maintainers directly via email or Discord. We will coordinate a responsible disclosure.

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (see LICENSE).
