## Why

6 TUI widget render functions (~200 lines) are fully implemented but marked
`#[allow(dead_code)]` — never called from the render path. The main render loop
in `widgets/mod.rs` bypasses them, using inline implementations instead. This dead
code represents completed work that's invisible to users. Wiring it up is the cheapest
way to close the visual gap with Claude Code before tackling larger refactors (T1).

## What Changes

- Wire `render_tool_spinners()` as an additional line after the activity header in
  the message flow — matching CC's `SpinnerAnimationRow` inline pattern
- Refactor `render_permission_prompt()` to return `Vec<Line>` for inline use in the
  message scroll area, replacing the current inline lines in `messages.rs` — matching
  CC's inline `PermissionPrompt` Select component (NOT an overlay with bordered Block)
- Wire `render_autocomplete()` as a popup overlay above the input area — matching CC's
  absolute-positioned `SuggestionsOverlay` (trigger logic already wired in `input.rs`)
- Wire `render_toasts()` to `state.notifications` (fed from `SessionView.pending_toasts`)
  as a conditional row above input, showing one notification at a time — matching CC's
  single-queue `Notifications.tsx` pattern
- Wire `render_hints()` into the status bar area (internally split) — matching CC's
  footer hint pattern; add Search-mode support
- Wire `render_diff()` into tool result blocks when output contains unified diff
- Expand `mod.rs` layout to use **dynamic conditional chunks** — add rows only when
  their corresponding state is active — matching CC's flexbox behavior
- Remove all 6 `#[allow(dead_code)]` annotations

## Capabilities

### New Capabilities

_None_ — all capabilities already have specs in `openspec/specs/`.

### Modified Capabilities

- `diff-renderer`: activating existing spec — wiring into tool result render path
- `permission-prompt`: activating existing spec — refactoring to inline `Vec<Line>`
  (matching CC's inline permission flow, NOT overlay)
- `command-autocomplete`: activating existing spec — wiring render only (trigger
  already wired in `input.rs:160-193`)
- `notification-toast`: activating existing spec — wiring to `state.notifications`
  with single-queue display
- `command-hints`: activating existing spec — wiring into status bar with Search
  screen support
- `tui-tool-rendering`: activating existing spec — wiring tool spinners after activity
  header in message flow

## Impact

- **Files modified**: `widgets/mod.rs` (dynamic layout chunks), `widgets/messages.rs`
  (refactor permission to use widget function), `widgets/activity_line.rs` (integrate
  tool spinners after header), `widgets/layout.rs` (wire toasts, hints, autocomplete
  + refactor permission widget signature), `widgets/permission.rs` (change signature
  from `frame.render_widget()` to `Vec<Line>` return), `widgets/tool_block.rs`
  (wire diff renderer)
- **No protocol changes** — all data already flows through `SessionView` / `TuiState`
- **No new dependencies** — all rendering code exists
- **Risk**: low–medium — permission refactor touches existing working code; layout
  changes need careful chunk arithmetic. Mitigated by incremental one-at-a-time wiring.
