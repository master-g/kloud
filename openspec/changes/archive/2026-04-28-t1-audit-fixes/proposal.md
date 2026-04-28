## Why

Code review of the T1 module refactor (`state.rs` → `state/` directory) found a scroll direction bug, dead code, a misleading `pub(super)` visibility, and minor logic issues. Fix now before T2+ layers build on these foundations.

## What Changes

- Fix `scroll_state.rs:scroll_up()` — uses `saturating_add` instead of `saturating_sub`, scrolls wrong direction
- Guard `input_height()` against `max_height < 2` edge case
- Deduplicate `Instant::now()` in `sync_from_active_view`
- Remove dead `last_seen_message_count.min()` line (overwritten immediately)
- Remove unused accessor methods on `TuiState` (callers already use `state.app.*`)
- Fix `SLASH_COMMAND_HINTS` visibility: `pub(super)` → `pub(crate)` (accessed cross-module via `input_state::`)
- Remove dead `theme_state.rs` (no `theme` field on `TuiState`, theme passed as parameter)

## Capabilities

### New Capabilities

(none — bug fixes only)

### Modified Capabilities

- `sticky-scroll`: `scroll_up()` direction fix changes scroll behavior contract
- `tui-input`: `SLASH_COMMAND_HINTS` visibility change; `input_height()` guard

## Impact

- `src/ui/tui/state/scroll_state.rs` — bug fix, 1 line
- `src/ui/tui/state/mod.rs` — remove dead code, dedupe `Instant::now()`, guard `input_height()`
- `src/ui/tui/state/input_state.rs` — visibility change
- `src/ui/tui/state/theme_state.rs` — delete file
- No API or dependency changes
