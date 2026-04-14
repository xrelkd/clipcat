## Why

Users want visibility into clipboard item provenance — knowing whether an item came from PRIMARY, SECONDARY, or CLIPBOARD selection, and when it was captured. This helps distinguish recently copied items and understand data flow between X11 selections.

## What Changes

- Add optional metadata display prefix in history list UI (e.g., `[P]`, `[S]`, `[C]` for Primary/Secondary/Clipboard)
- Add config option `show_source_prefix` to toggle prefix display
- Timestamp display is already available in the metadata but not prominently shown in list view

## Capabilities

### New Capabilities

- `clipboard-source-display`: Display clipboard source type (Primary/Secondary/Clipboard) as prefix in history list items with optional config toggle

### Modified Capabilities

- (none — this extends existing display behavior, not changes requirements)

## Impact

- `clipcat-menu`: Updates to history list rendering to show source prefixes
- `clipcat-menu` config: New `show_source_prefix` option
- `clipcatctl`: Updates to list output to show source prefixes
- `clipcatctl` config: New `show_source_prefix` option
- No backend changes required — data already exists in `Entry` struct
