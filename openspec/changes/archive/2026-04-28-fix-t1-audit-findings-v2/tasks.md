## 1. ScrollState sentinel fix

- [x] 1.1 Add `jump_to_bottom: bool` field to `ScrollState`, init `false`. Change `jump_to_bottom()` to set flag instead of `offset = usize::MAX`.
- [x] 1.2 Add `pub fn should_jump_to_bottom(&self) -> bool` and `pub fn clear_jump_flag(&mut self)` accessors to `ScrollState`.
- [x] 1.3 Update render loop in `src/ui/tui/widgets/messages.rs`: before computing `scroll`, check `state.scroll.should_jump_to_bottom()`. If true, use `max_scroll` as offset and call `state.scroll.clear_jump_flag()`.
- [x] 1.4 Verify `offset_u16()` no longer can return `u16::MAX` from a stale sentinel.

## 2. Dead parameter cleanup

- [x] 2.1 Change `TuiState::scroll_messages_down` signature from `(n: u16, _content_height: u16, _visible_height: u16)` to `(n: u16)`. Update body to call `self.scroll.scroll_down_by(n as usize)`.
- [x] 2.2 Update caller in `src/ui/tui/input.rs`: change `state.scroll_messages_down(10, usize::MAX as u16, usize::MAX as u16)` to `state.scroll_messages_down(10)`.

## 3. AppState consistency fixes

- [x] 3.1 Add `#[derive(Debug, Clone)]` to `AppState` in `src/ui/tui/state/app_state.rs`.
- [x] 3.2 Extract inline `std::collections::HashMap` to a `use std::collections::HashMap;` import at top of `app_state.rs`.

## 4. Showcase cleanup

- [x] 4.1 Remove unused `let _idx = i as usize;` line from `examples/tui_showcase.rs`.

## 5. Verify

- [x] 5.1 Run `cargo check` — no compile errors.
- [x] 5.2 Run `cargo fmt --all && cargo clippy -- -W warnings` — clean.
