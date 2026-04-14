## 1. clipcat-menu Config

- [ ] 1.1 Add `show_source_prefix: bool` to `clipcat-menu` Config struct in `src/config.rs`
- [ ] 1.2 Set default to `false` in Config::default()

## 2. clipcat-menu UI Display

- [ ] 2.1 Find where history list items are rendered in clipcat-menu
- [ ] 2.2 Add source prefix formatting (use Kind::as_str() first char or custom)
- [ ] 2.3 Prefix item preview with `[P]`, `[S]`, `[C]` when config enabled

## 3. clipcatctl Config

- [ ] 3.1 Add `show_source_prefix: bool` to `clipcatctl` Config struct in `src/config.rs`
- [ ] 3.2 Set default to `false` in Config::default()

## 4. clipcatctl List Output

- [ ] 4.1 Update `print_list()` in `cli.rs` to include source prefix
- [ ] 4.2 Check config option before prefixing

## 5. Verify

- [ ] 5.1 Build clipcat-menu and clipcatctl
- [ ] 5.2 Test each with config enabled/disabled
