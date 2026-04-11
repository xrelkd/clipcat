# Coding Conventions

This file describes the Rust coding conventions used in this project.

## Project Structure

```text
clipcat/
├── Cargo.toml            # Workspace manifest
├── clipcatd/             # Daemon (server)
├── clipcatctl/           # CLI client tool
├── clipcat-menu/         # Menu client (with finders)
├── clipcat-notify/       # Clipboard monitoring tool
├── crates/
│   ├── base/             # Common types and utilities
│   ├── cli/              # CLI utilities
│   ├── client/           # gRPC client
│   ├── clipboard/        # Clipboard operations
│   ├── dbus-variant/     # D-Bus variant definitions
│   ├── external-editor/  # External editor integration
│   ├── metrics/          # Metrics utilities
│   ├── proto/            # Protocol buffer definitions
│   └── server/           # gRPC server
└── docs/                 # Documentation
```

## Import Ordering

Imports should be ordered using the 4-group convention:

1. `std` crate
2. Third-party crates (external dependencies)
3. `self::` paths (child submodules)
4. `crate::` paths (siblings and ancestors)

Example from `crates/base/src/entry.rs`:

```rust
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

use image::ImageEncoder as _;
use sha2::{Digest, Sha256};
use snafu::{ResultExt, Snafu};
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

use crate::{ClipboardContent, ClipboardKind};
```

### No Glob Imports

Never use glob (wildcard) imports (`use foo::*`). Always import items explicitly:

```rust
// GOOD
use std::collections::HashMap;
use crate::config::Config;

// BAD
use std::collections::*;
use crate::config::*;
```

## Attribute Ordering

One principle for all items — **Gate → Transform → Derive → Config → Contract → Lint → Physical**:

| #   | Category      | Applies to | Examples                                                            |
| --- | ------------- | ---------- | ------------------------------------------------------------------- |
| 1   | **Gate**      | All        | `#[cfg]`, `#[cfg_attr]`, `#[test]`, `#[tokio::test]`                |
| 2   | **Transform** | All        | `#[serde_as]`, `#[skip_serializing_none]`, `#[async_trait]`         |
| 3   | **Derive**    | Types      | `#[derive(...)]`                                                    |
| 4   | **Config**    | Types      | Derive helpers: `#[serde(...)]`                                     |
| 5   | **Contract**  | All        | `#[must_use]`, `#[deprecated]`, `#[non_exhaustive]`                 |
| 6   | **Lint**      | All        | `#[expect(clippy::...)]`                                            |
| 7   | **Physical**  | All        | Types: `#[repr]`. Functions: `#[inline]`, `#[cold]`, `#[no_mangle]` |

Example from `crates/cli/src/config/log.rs`:

```rust
#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogConfig {
    #[serde(default = "LogConfig::default_file_path")]
    pub file_path: Option<PathBuf>,
    // ...
}
```

Example with Transform + Derive + Config + Physical:

```rust
// Gate → Transform → Derive → Config → Physical
#[cfg(feature = "image")]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum ImageFormat {
    Png,
    Jpeg,
}
```

## Derive Macro Ordering

Order derive macros: **standard library → third-party**, alphabetical within each group:

```rust
// GOOD — std (Clone, Debug, Eq) → third-party (Deserialize, Serialize)
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]

// BAD — random order
#[derive(Serialize, Hash, Clone, Debug, Default)]
```

For structures with many derives, use the 5-tier priority system:

### Tier 1: Basics

`Clone`, `Copy`, `Debug`, `Default`, `Eq`, `Hash`, `Ord`, `PartialEq`, `PartialOrd`

### Tier 2: Comparison

`BitwiseCast`, `Symbolic`

### Tier 3: Data/Database

`Deserialize`, `Serialize`

### Tier 4: Validation

`Constrained`, `Validate`

### Tier 5: Custom

Any other derive macros (e.g., `Snafu`, `Type`)

## Error Handling

This project uses [SNAFU](https://github.com/rtsuru/snafu) for error handling.

### Error Enum Pattern

Example from `crates/server/src/error.rs`:

```rust
use std::path::PathBuf;

use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Error occurs while starting tonic server, error: {source}"))]
    StartTonicServer { source: tonic::TransportError },

    #[snafu(display("Error occurs while creating Unix domain socket listener on `{}`, error: {source}", socket_path.display()))]
    CreateUnixListener { socket_path: PathBuf, source: std::io::Error },

    #[snafu(display("{source}"))]
    Metrics { source: crate::metrics::Error },
}
```

Example from `clipcatd/src/error.rs`:

```rust
use snafu::Snafu;

use crate::{config, pid_file};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not initialize tokio runtime, error: {source}"))]
    InitializeTokioRuntime { source: tokio::io::Error },

    #[snafu(display("{source}"))]
    Application { source: Box<clipcat_server::Error> },

    #[snafu(display("Failed to daemonize, error: {source}"))]
    Daemonize { source: daemonize::Error },
}

impl From<daemonize::Error> for Error {
    fn from(source: daemonize::Error) -> Self { Self::Daemonize { source } }
}

impl From<clipcat_server::Error> for Error {
    fn from(source: clipcat_server::Error) -> Self {
        Self::Application { source: Box::new(source) }
    }
}
```

### Error Propagation

Prefer `.with_context()` over `.context()` to avoid unnecessary allocations on the success path:

```rust
use snafu::ResultExt;

// GOOD — with_context: no clone on success path
let data = std::fs::read(&path)
    .with_context(|_| error::ReadFileSnafu { path: path.clone() })?;

// OK — context when no extra fields needed
signal::sigprocmask(SigmaskHow::SIG_BLOCK, Some(&mask), None)
    .context(error::SetSignalMaskSnafu)?;
```

### Avoid unwrap()/expect()

- **`unwrap()` is prohibited** in production code
- Use `?` operator with proper error types:

```rust
// GOOD
let content = std::fs::read(&path).context(error::ReadFileSnafu)?;

// BAD
let content = self.read_content().unwrap();
```

- `expect()` is only permitted in post-fork single-threaded context where inputs are already validated.

## Async Patterns

This project uses [Tokio](https://tokio.rs/) for async runtime.

### tokio::select

Format with proper alignment:

```rust
tokio::select! {
    Some(item) = receiver.recv() => {
        handle_item(item).await;
    }
    _ = tokio::time::sleep(Duration::from_secs(1)) => {
        timeout_handler().await;
    }
}
```

### Spawning Tasks

Use `tokio::spawn` for running tasks concurrently:

```rust
let handle = tokio::spawn(async move {
    some_async_operation().await
});

let result = handle.await.context(TaskJoin)?;
```

## Path Arguments Handling

Use `AsRef<Path>` for file path parameters:

```rust
// GOOD
pub fn load_config(path: impl AsRef<Path>) -> std::io::Result<String> {
    let path = path.as_ref();
    std::fs::read_to_string(path)
}

// BAD
fn load_config(path: &str)  // requires manual conversion
fn load_config(path: PathBuf)  // forces ownership transfer
```

## Testing

This project uses [cargo-nextest](https://nexte.st/) for running tests.

### Running Tests

```bash
# Run all tests
cargo nextest run

# Run tests in a specific crate
cargo nextest run -p clipcatd

# Run tests with output
cargo nextest run --no-capture
```

### Async Tests

Use `#[tokio::test]` for async tests:

```rust
#[tokio::test]
async fn test_clipboard_listen() {
    let listener = ClipboardListener::new().await;
    // test code...
}
```

### Test Assertions

Choose the assertion macro that gives the best failure message:

```rust
// GOOD — assert_eq shows both values on failure
assert_eq!(result.status, 200);

// GOOD — assert_matches for enum variant checks
assert_matches!(result, Ok(Response { status: 200, .. }));

// BAD — assert! with equality gives poor failure messages
assert!(result.status == 200);
```

## Code Quality

### Formatting

```bash
cargo fmt
treefmt
nix fmt  # if using Nix
```

### Linting

Run clippy before committing. Treat all warnings as errors:

```bash
cargo clippy
cargo clippy-all --fix
```

### Lint Suppressions

`#[allow(...)]` is **prohibited**. Use `#[expect(...)]` with a reason:

```rust
// GOOD
#[expect(clippy::too_many_arguments, reason = "builder pattern not worth it for internal-only fn")]
fn create_session(/* ... */) {}

// BAD
#[allow(clippy::too_many_arguments)]
fn create_session(/* ... */) {}
```

### Unsafe Code

```rust
// GOOD
#[expect(unsafe_code, reason = "FFI call requires raw pointer; lifetime is bound to `self`")]
unsafe { ffi_init(ptr) }

// BAD
unsafe { ffi_init(ptr) }
```

### Keep Commits Clean

- Fix all lints and compilation errors in the same commit as your feature/fix
- Do NOT create separate commits for lint fixes
- Keep commit messages concise and meaningful

## Naming

- Prefer full words over abbreviations: `quantity` not `qty`, `configuration` not `cfg`
- Exceptions: widely understood domain abbreviations and Rust conventions (`ctx`, `rx`/`tx`, `fn`, `impl`)
