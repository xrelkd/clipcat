## Why

Users want visibility into clipboard item provenance — knowing whether an item came from PRIMARY, SECONDARY, or CLIPBOARD selection. This helps distinguish recently copied items and understand data flow between X11 selections.

## What Changes

- Add optional metadata display prefix in history list UI (e.g., `[P]`, `[S]`, `[C]` for Primary/Secondary/Clipboard)
- Add `show_source_prefix: bool` field to each finder implementation (set via setter methods like `set_extra_arguments`, `set_line_length`, `set_menu_length`)
- Add `fn show_prefix(&self) -> &'static str` method to `Kind` enum for generating prefix string
- Use existing `generate_input` method to generate the input with prefix

## Capabilities

### New Capabilities

- `clipboard-source-display`: Display clipboard source type as prefix in history list items with per-finder config toggle

### Modified Capabilities

- `generate_input`: Extended to optionally include source prefix based on `show_source_prefix` setting

## Implementation

### Finder Configuration Pattern

Each finder implements setter methods following existing pattern:

```rust
fn set_extra_arguments(&mut self, _arguments: &[String]) {}

fn set_line_length(&mut self, _line_length: usize) {}

fn set_menu_length(&mut self, _menu_length: usize) {}

fn set_show_source_prefix(&mut self, _show: bool) {}
```

### Kind Prefix Method

Add method to `Kind` enum in `crates/base/src/kind.rs`:

```rust
impl Kind {
    pub const fn show_prefix(&self) -> &'static str {
        match self {
            Self::Clipboard => "[C]",
            Self::Primary => "[P]",
            Self::Secondary => "[S]",
        }
    }
}
```

### Input Generation

Update `generate_input` in `FinderStream` trait to include prefix when enabled:

```rust
fn generate_input(&self, clips: &[ClipEntryMetadata]) -> String {
    clips
        .iter()
        .enumerate()
        .map(|(i, ClipEntryMetadata { preview, kind, .. })| {
            let prefix = if self.show_source_prefix() { kind.show_prefix() } else { "" };
            format!("{i}{INDEX_SEPARATOR} {prefix}{preview}")
        })
        .collect::<Vec<_>>()
        .join(ENTRY_SEPARATOR)
}
```

## Impact

- `clipcat-menu`: Updates to finders (rofi, choose) to support source prefix
- `clipcatctl`: Updates to list output to show source prefixes
- `crates/base`: Add `show_prefix()` method to `Kind` enum
- No backend changes required — data already exists in `Entry` struct
