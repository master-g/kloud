## Why

6 TUI widgets are fully implemented (~200 lines) but marked `#[allow(dead_code)]` —
never called from the render path. The main render loop in `widgets/mod.rs` bypasses
them, using inline implementations instead. This dead code represents completed work
that's invisible to users. Wiring it up is the cheapest way to close the visual gap
with Claude Code before tackling larger refactors (T1).

## What Changes

- Wire `render_diff()` into tool result blocks when output contains unified diff
- Replace inline permission rendering in `messages.rs` with `render_permission_prompt()` widget
- Wire `render_autocomplete()` to `/` key trigger in `input.rs` using existing `AutocompleteState`
- Wire `render_toasts()` to `pending_toasts` from `SessionView`
- Wire `render_hints()` into status bar area, width-gated
- Wire `render_tool_spinners()` into activity line during tool execution
- Remove all 6 `#[allow(dead_code)]` annotations

## Capabilities

### New Capabilities

_None_ — all capabilities already have specs in `openspec/specs/`.

### Modified Capabilities

- `diff-renderer`: activating existing spec — wiring into tool result render path
- `permission-prompt`: activating existing spec — replacing inline rendering with widget
- `command-autocomplete`: activating existing spec — wiring render + `/` key trigger
- `notification-toast`: activating existing spec — wiring to `pending_toasts`
- `command-hints`: activating existing spec — wiring into status bar
- `tui-tool-rendering`: activating existing spec — wiring tool spinners into activity line

## Impact

- **Files modified**: `widgets/mod.rs` (render dispatch), `widgets/messages.rs` (remove inline
  permission), `widgets/activity_line.rs` (integrate tool spinners), `widgets/layout.rs`
  (wire toasts, hints, autocomplete), `input.rs` (autocomplete trigger)
- **No protocol changes** — all data already flows through `SessionView` / `TuiState`
- **No new dependencies** — all rendering code exists
- **Risk**: low — no new logic, only connecting existing functions to existing data
