## 1. Kind Prefix Method

- [x] 1.1 Add `fn show_prefix(&self) -> &'static str` method to `Kind` enum in `crates/base/src/kind.rs`

## 2. FinderStream Trait

- [x] 2.1 Add `fn show_source_prefix(&self) -> bool` method to `FinderStream` trait
- [x] 2.2 Add `fn set_show_source_prefix(&mut self, show: bool)` setter to `FinderStream` trait
- [x] 2.3 Update `generate_input` to include prefix when enabled (use `kind.show_prefix()`)

## 3. Rofi Finder Implementation

- [x] 3.1 Implement `show_source_prefix()` returning stored value
- [x] 3.2 Implement `set_show_source_prefix()` to store the value

## 4. Choose Finder Implementation

- [x] 4.1 Implement `show_source_prefix()` returning stored value
- [x] 4.2 Implement `set_show_source_prefix()` to store the value

## 5. clipcat-menu Config

- [x] 5.1 Add `show_source_prefix: bool` to clipcat-menu Config struct in `src/config.rs`
- [x] 5.2 Set default to `false` in Config::default()

## 6. clipcatctl Config

- [x] 6.1 Add `show_source_prefix: bool` to clipcatctl Config struct in `src/config.rs`
- [x] 6.2 Set default to `false` in Config::default()

## 7. clipcatctl List Output

- [x] 7.1 Update list output to include source prefix when config enabled

## 8. Verify

- [x] 8.1 Build clipcat-menu and clipcatctl
- [x] 8.2 Test each with config enabled/disabled
