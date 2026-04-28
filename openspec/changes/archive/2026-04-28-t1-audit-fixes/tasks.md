## 1. Bug Fixes

- [x] 1.1 Fix `scroll_state.rs:scroll_up()` — change `saturating_add` to `saturating_sub` so offset decreases
- [x] 1.2 Guard `state/mod.rs:input_height()` — add `.max(3)` to prevent underflow on tiny terminals

## 2. Dead Code Removal

- [x] 2.1 Remove unused accessors from `TuiState` (`messages()`, `messages_mut()`, `screen()`, `pending_permission()`, `active_tools()`, `recent_activity()`, `live_activity()`, `activity_snapshot()`, `collapsed_tools()`, `notifications()`) — verify each has zero callers
- [x] 2.2 Remove dead `last_seen_message_count.min(new_count)` line in `sync_from_active_view`
- [x] 2.3 Delete `src/ui/tui/state/theme_state.rs` and remove `mod theme_state` + `pub use ThemeState` from `state/mod.rs`

## 3. Visibility Fix

- [x] 3.1 Change `SLASH_COMMAND_HINTS` from `pub(super)` to `pub(crate)` in `input_state.rs`

## 4. Deduplication

- [x] 4.1 Replace second `let now = Instant::now();` in `sync_from_active_view` with reuse of first binding

## 5. Verification

- [x] 5.1 Run `cargo fmt --all && cargo clippy -- -W warnings`
- [x] 5.2 Run `cargo test` — all 50 tests pass
