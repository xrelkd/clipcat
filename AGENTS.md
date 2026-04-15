# AGENTS.md

This file provides guidance for agentic coding assistants working in the Clipcat repository.

## Before Making Any Changes

Always read these files first:

- **[conventions.md](conventions.md)** — Rust coding standards
- **[CONTRIBUTING.md](CONTRIBUTING.md)** — Development workflow, commit conventions

## Build, Test, and Quality Commands

```bash
# Build
cargo build                    # Debug
cargo build --release         # Release
cargo build -p <package>      # Specific package

# Test (cargo-nextest is faster, used in CI)
cargo nextest run
cargo nextest run --release

# Code quality (order matters!)
cargo fmt                      # Format first
cargo clippy --fix            # Fix warnings
cargo check                   # Type check
cargo nextest run             # Tests last
```

> **Tip**: Always run format → lint → typecheck → test. Fix lints in the same commit as your changes.

## Critical Conventions

- **Error handling**: Use SNAFU. No `unwrap()` in production.
- **Lint suppression**: Use `#[expect(...)]` with a reason, NOT `#[allow(...)]`
- **Import ordering**: std → third-party → self:: → crate:: (4-group)
- **Derive ordering**: std types → third-party, alphabetical within each group
- **No glob imports**: Never use `use foo::*`
- **unsafe**: Requires `#[expect(unsafe_code, reason = "...")]`

## Project Structure

| Component          | Type   | Description                 |
| ------------------ | ------ | --------------------------- |
| `clipcatd`         | Binary | Daemon (server)             |
| `clipcatctl`       | Binary | CLI client                  |
| `clipcat-menu`     | Binary | Menu client with finders    |
| `clipcat-notify`   | Binary | Clipboard monitor tool      |
| `crates/server`    | Lib    | gRPC server                 |
| `crates/client`    | Lib    | gRPC client                 |
| `crates/clipboard` | Lib    | Clipboard operations        |
| `crates/proto`     | Lib    | Protocol buffer definitions |
| `crates/base`      | Lib    | Common types                |
| `crates/cli`       | Lib    | CLI utilities               |

## Branch & Commit Convention

- **Base branch**: `develop` (not `main` for features)
- **Branch prefixes**: `feat/*`, `fix/*`, `release/*`, `ci/*`
- **Commits**: One scope per commit. Format: `<type>(<scope>): <description>`
- **Scopes**: `clipcatd`, `clipcatctl`, `clipcat-menu`, `clipcat-notify`, `crates/base`, `crates/cli`, `crates/client`, `crates/clipboard`, `crates/proto`, `crates/server`, `actions`, `nix`

## Required Tools (Non-Nix)

If not using Nix, install manually:

- Rust 1.93+ (check `rust-toolchain.toml`)
- cargo-nextest
- protobuf compiler (protoc)
- For tests on Linux: xvfb-run
