## 1. Create state/ directory and extract animation types

- [x] 1.1 Create `src/ui/tui/state/` directory with `mod.rs` (re-export sub-modules). After all content migrated from `state.rs`, delete `src/ui/tui/state.rs` (Rust forbids coexisting `state.rs` and `state/`)
- [x] 1.2 Move `ActivityClock`, `StalledState`, `TokenCounter`, `ThinkingStatus` from `state.rs:28-231` to `state/anim_state.rs`
- [x] 1.3 Move `ActivityEntry`, `LiveActivity`, `ActivitySnapshot` to `state/anim_state.rs`
- [x] 1.4 Add `pub use anim_state::*` in `state/mod.rs`; update imports in `state.rs` and `mod.rs`
- [x] 1.5 `cargo check` passes with animation types in new location

## 2. Extract input state

- [x] 2.1 Create `state/input_state.rs` with `InputState` struct (fields: text_area, history, history_index, search, autocomplete)
- [x] 2.2 Move `SearchState`, `AutocompleteState` types to `input_state.rs`
- [x] 2.3 Update `TuiState` to hold `input: InputState` instead of individual fields
- [x] 2.4 Update all `state.text_area` → `state.input.text_area`, `state.history` → `state.input.history`, etc. in `input.rs`, `widgets/*.rs`, `mod.rs`
- [x] 2.5 `cargo check` passes with input state extracted

## 3. Extract app state

- [x] 3.1 Create `state/app_state.rs` with `AppState` struct (fields: messages, screen, pending_permission, notifications, collapsed_tools, active_tools, tool_start_times, recent_activity, live_activity, activity_snapshot)
- [x] 3.2 Move `Notification` type to `app_state.rs`
- [x] 3.3 Update `TuiState` to hold `app: AppState` instead of individual fields
- [x] 3.4 Update all `state.messages` → `state.app.messages`, `state.screen` → `state.app.screen`, etc.
- [x] 3.5 `cargo check` passes with app state extracted

## 4. Extract scroll state and eliminate redundant scroll field

- [x] 4.1 Create `state/scroll_state.rs` with `ScrollState` struct wrapping `VirtualScroll` and `auto_scroll_paused`
- [x] 4.2 Delete `scroll: u16` field from `TuiState`
- [x] 4.3 Update scroll references in `input.rs` (lines 233,237,243,348,352,356,360,364,368) AND `messages.rs` (lines 108,120) to use VirtualScroll methods
- [x] 4.4 Update `state.rs` `scroll_messages_up()` and `scroll_messages_down()` to use VirtualScroll methods
- [x] 4.5 Update render code in `messages.rs` that reads `state.scroll` to use `state.scroll.virtual_scroll.scroll_offset`
- [x] 4.6 `cargo test` passes with scroll consolidation

## 5. Extract theme state — SKIPPED

_No theme fields in `TuiState`. Theme switching lives in `RatatuiBackend.theme` (mod.rs:149) and `AppEvent::ThemeChanged`. Nothing to extract into a `ThemeState` struct._

- [x] 5.1 Create `state/theme_state.rs` with `ThemeState` struct
- [x] 5.2 Move theme switching fields to `ThemeState`
- [x] 5.3 Update theme toggle code to use `state.theme`
- [x] 5.4 `cargo check` passes

## 6. Relocate constants

- [x] 6.1 Move `src/ui/constants.rs` → `src/ui/tui/constants.rs`
- [x] 6.2 Update all imports from `crate::ui::constants` to `crate::ui::tui::constants`
- [x] 6.3 Delete old `src/ui/constants.rs`
- [x] 6.4 `cargo check` passes

## 7. Dynamic input height

- [x] 7.1 Add `input_height()` method to `TuiState` that returns `text_area.line_count().clamp(3, max/2) + 2` as `u16`
- [x] 7.2 Update `widgets/mod.rs:48` from `Constraint::Length(3)` to `Constraint::Length(state.input_height())`
- [x] 7.3 Ensure message area uses `Constraint::Min(1)` to absorb remaining space
- [x] 7.4 Test: multi-line input expands area, single-line stays at 3

## 7.5. Update tests and examples

- [x] 7.5.1 Update 5 tests in `state.rs` — field paths to sub-structs (e.g., `state.messages` → `state.app.messages`)
- [x] 7.5.2 Update `examples/tui_showcase.rs` — all TuiState field access paths
- [x] 7.5.3 `cargo test` all passes

## 8. Final cleanup and verification

- [x] 8.1 Verify `state/mod.rs` facade is < 200 lines (apply_view + sync_from_active_view alone are ~110 lines)
- [x] 8.2 `cargo fmt --all && cargo clippy -- -W warnings` clean
- [x] 8.3 `cargo test` all passes
- [x] 8.4 `cargo run --example tui_showcase` visual check — no regressions
