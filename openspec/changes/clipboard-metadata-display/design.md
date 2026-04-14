## Context

Clipcat stores clipboard items with metadata including:

- `ClipboardKind` enum: `Clipboard`, `Primary`, `Secondary` (X11 selections)
- `timestamp`: When the item was captured
- `id`: Unique identifier

Currently, history list in `clipcat-menu` shows items with previews but without source indicators.

## Goals / Non-Goals

**Goals:**

- Display source prefix `[P]/[S]/[C]` in history list for both clipcat-menu and clipcatctl
- Add config option `show_source_prefix` per app to toggle prefix visibility

**Non-Goals:**

- Add new metadata fields (already exists)
- Change backend storage format
- Add timestamp display beyond existing metadata

## Decisions

1. **UI Prefix Format**: Use `[P]`, `[S]`, `[C]` — short, standard, fits rofi/dmenu columns
2. **Config Location**: Add `show_source_prefix: bool` to both `clipcat-menu` and `clipcatctl` configs
3. **Default**: Disable by default to preserve existing UI behavior

## Risks / Trade-offs

- [Risk] Prefix takes extra column space → [Mitigation] Config toggle lets users disable
- [Risk] Breaking change in list format → [Mitigation] Default off preserves existing behavior
