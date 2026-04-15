# Contributing to Clipcat

> **Note**: For detailed coding conventions, see [conventions.md](conventions.md).

## Nix Flake (Recommended)

This project uses [Nix Flakes](https://nixos.wiki/wiki/Flakes) for development environment. We **strongly recommend** using Nix to ensure consistent tooling across all contributors.

### Quick Start

```bash
# Enter the development environment
nix develop

# Build the project
cargo build

# Run tests
cargo nextest run
```

### Using direnv (Recommended for Faster Development)

[direnv](https://direnv.net/) automatically loads the Nix environment when you enter the project directory.

1. Install direnv: `nix profile install nixpkgs#direnv`
2. Add to your shell (bash/zsh/fish)
3. Allow direnv in the project directory: `direnv allow`

Then the development environment will load automatically every time you enter the project.

### Alternative: Without Nix

If you don't use Nix, you need to install the following tools manually:

**Required:**

- Rust (see `rust-toolchain.toml` for version)
- Cargo
- cargo-nextest
- protobuf (protoc)
- pkg-config
- libgit2

**Optional (for formatting/linting):**

- rustfmt
- treefmt
- nixfmt
- shfmt
- taplo
- hclfmt

**Optional (for testing on Linux):**

- xvfb-run

```bash
# Clone the project
git clone https://github.com/xrelkd/clipcat.git
cd clipcat

# Build
cargo build
cargo build --release

# Run
cargo run --package clipcatd
```

## Development Workflow

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run a specific package
cargo run --package clipcatd
```

### Testing

```bash
# Run tests
cargo test

# Or use cargo-nextest (faster, used in CI)
cargo nextest run
cargo nextest run --release
```

### Code Quality

**Always run these before creating a commit:**

```bash
# Format code
cargo fmt

# Or use treefmt
treefmt

# Or nix fmt (if using Nix)
nix fmt

# Run lints (fix all warnings/errors)
cargo clippy
cargo clippy-all --fix
```

**Keep commits clean:**

- Fix all lints and compilation errors in the same commit as your feature/fix
- Do NOT create separate commits for lint fixes
- Keep commit messages concise and meaningful

### Branch Naming

Create feature/fix branches from `develop`:

```bash
# Feature branch
git checkout -b feat/my-new-feature develop

# Fix branch
git checkout -f fix/bug-description develop
```

Supported prefixes: `feat/*`, `feature/*`, `fix/*`, `hotfix/*`, `release/*`, `ci/*`

### Creating a Pull Request

1. Run formatters (`cargo fmt`, `treefmt`, or `nix fmt`)
2. Run lints and fix all warnings/errors (`cargo clippy-all --fix`)
3. Ensure tests pass (`cargo nextest run`)
4. Push your branch to your fork
5. Create a PR targeting the `develop` branch
6. Ensure all CI checks pass
7. Keep commits clean, concise, and follow the commit message convention

---

## Commit Message Convention

This project follows [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Format

```text
<type>(<scope>): <description> (#PR)
```

### Types

| Type       | Description                        |
| ---------- | ---------------------------------- |
| `feat`     | New feature                        |
| `fix`      | Bug fix                            |
| `refactor` | Code refactoring                   |
| `docs`     | Documentation updates              |
| `style`    | Code formatting (no logic changes) |
| `ci`       | CI/CD modifications                |
| `chore`    | Maintenance tasks                  |
| `build`    | Dependency updates                 |

### Scopes

The scope is optional but recommended when the change affects a specific component.

#### One Scope Per Commit

**Each commit should only affect one scope.** If your PR modifies multiple components, create separate commits for each one.

Good:

```bash
# Commit 1
fix(crates/base): fix lints and errors

# Commit 2
fix(clipcatctl): fix lints and errors
```

Bad:

```bash
# Don't do this
fix: fix lints and errors in crates/base and clipcatctl
```

**Lint fixes should be part of the feature commit.** Do not create separate commits for fixing lints or compilation errors.

#### Common Scopes

| Scope              | Description                |
| ------------------ | -------------------------- |
| `clipcatd`         | Daemon component           |
| `clipcatctl`       | CLI control tool           |
| `clipcat-menu`     | Menu component             |
| `clipcat-notify`   | Notification component     |
| `crates/base`      | Base/common crate          |
| `crates/cli`       | CLI utilities crate        |
| `crates/client`    | gRPC client crate          |
| `crates/clipboard` | Clipboard crate            |
| `crates/proto`     | Protocol definitions crate |
| `crates/server`    | gRPC server crate          |
| `actions`          | GitHub Actions workflows   |
| `nix`              | Nix NixOS modules          |

#### Scope-less Commits

Some commits don't require a scope:

- `style: apply formatter`
- `chore: update Rust formatter`
- `build(deps): update Cargo.lock`

### Examples

```bash
# Feature in multiple scopes - create separate commits
feat(crates/clipboard): add new clipboard listener for Wayland
feat(clipcatd): integrate new clipboard listener

# Feature
feat(clipcatd): add history clearing on startup

# Bug Fix
fix(clipcat-menu): pass SkimOptions by value to Skim::run_with

# Refactor
refactor(crates/clipboard): remove clippy suppression and `FIXME`

# Documentation
docs(readme): document rofi delete keybinding

# CI/CD
ci(actions): add `--fix-missing` to `apt install` in release workflow

# Chore
chore(gitignore): add `aider` ignore pattern
chore(nix): extract `name` from the `Cargo.toml` file

# Style (no scope needed)
style: apply formatter

# Dependency Update (usually automated)
build(deps): bump tokio from 1.49.0 to 1.50.0 (#969)
```

### PR References

When submitting a PR that touches multiple scopes, use multiple commits:

```bash
# If your PR modifies both crates/base and clipcatctl
fix(crates/base): fix lints and errors
fix(clipcatctl): fix lints and errors

# Then merge with "Merge pull request #123"
```

Always reference the PR number when applicable:

```text
feat(clipcatd): add new feature (#123)
```
