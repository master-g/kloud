## Why

Code review of the T1 module refactor found 4 issues: a direct field mutation bypassing ScrollState encapsulation, `scroll_down` degrading PageDown to jump-to-bottom when called with dummy params, an unused variable warning, and stale task entries in tasks.md.

## What Changes

- Add `jump_to_bottom()` method to `ScrollState` for the End-key case, replacing direct `offset = usize::MAX` mutation
- Add `scroll_down_by(n)` method to `ScrollState` for incremental scroll without content-height clamping
- Fix unused variable in `tui_showcase.rs`
- Update `tasks.md` section 5 to reflect that theme_state extraction was intentionally skipped (no theme fields in TuiState)

## Capabilities

### New Capabilities

None. These are implementation-level fixes within existing T1 scope.

### Modified Capabilities

None. No spec-level requirement changes — all fixes are internal to the T1 module refactor implementation.

## Impact

- `src/ui/tui/state/scroll_state.rs` — two new methods
- `src/ui/tui/input.rs` — update End key handler and PageDown/scroll call sites
- `examples/tui_showcase.rs` — unused variable fix
- `openspec/changes/t1-module-refactor/tasks.md` — correct section 5 status
