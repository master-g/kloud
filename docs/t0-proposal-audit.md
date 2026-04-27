# T0 Dead Code Wiring — Proposal Audit

Audit date: 2026-04-27

## Sources checked

- `openspec/changes/t0-dead-code-wiring/` (proposal, design, tasks)
- `src/ui/tui/widgets/*.rs` (all 10 widget files)
- `src/ui/tui/input.rs` (key handling)
- `src/ui/tui/state.rs` (TuiState, AutocompleteState, Notification, etc.)
- `src/agent/view.rs` (SessionView, PendingPermissionView, PendingToast)
- `openspec/specs/*/` (6 capability specs)

## Confirmed correct

- 6 `#[allow(dead_code)]` annotations all at correct locations:
  - `activity_line.rs:182` — `render_tool_spinners`
  - `layout.rs:157` — `render_autocomplete`
  - `layout.rs:207` — `render_toasts`
  - `layout.rs:230` — `render_hints`
  - `permission.rs:12` — `render_permission_prompt`
  - `diff.rs:9` — `render_diff`
- All 6 widget functions have clean signatures taking `&TuiState` / `&Theme` / `Frame` / `Rect` as needed
- All required data fields exist in `TuiState` (`active_tools`, `autocomplete`, `notifications`, `pending_permission`, `tool_start_times`)
- No new dependencies, data types, or protocol changes needed
- Incremental wiring order (additive → conditional → replacement) is a sensible risk gradient
- Specs for all 6 capabilities exist under `openspec/specs/`

## Blockers (design gaps — must resolve before implementation)

### B1: Permission rendering strategy mismatch

The inline permission in `messages.rs:88-103` and the `render_permission_prompt()` widget in `permission.rs:13` use fundamentally different rendering models:

| Aspect | Inline (current) | Widget (proposed) |
|--------|------------------|-------------------|
| Method | Line pushed into scrollable Paragraph | `frame.render_widget()` with bordered Block |
| Position | Scrolls with message text | Fixed position on screen |
| Border | None | `Borders::ALL` |
| Area | Part of message content flow | Needs a dedicated `Rect` |

D4 says "replace inline entirely" but doesn't specify where the widget's `Rect` comes from. The current layout in `mod.rs:render()` has 4 fixed chunks; a permission overlay requires either:

- A new layout chunk (reshuffles all constraints)
- A floating overlay rendered on top of the messages area

Also, `PendingPermissionView` (view.rs:29) has only `label` and `description` — no `diff` field. The permission-prompt spec's requirement for diff previews is aspirational relative to the current data model.

### B2: Layout chunk constraints not addressed

Adding toasts (above input), hints (below input/status), and autocomplete (below input) all require changing the fixed `Layout::vertical()` constraints in `mod.rs:28-43`. The current 4-chunk layout has no room for these new UI elements:

```
Current:  [title 1] [messages *] [input 3] [status 1]
Needed:   [title 1] [messages *] [toasts 1-2] [input 3] [autocomplete 0-4] [hints 1] [status 1]
```

Tasks reference wiring into `layout.rs` functions but never include layout constraint changes as explicit steps. Decide:

- How many rows each new element gets
- Whether toasts/autocomplete are conditional chunks or overlays
- How this interacts with the transcript screen layout

## Warnings (implementation risk — can be resolved during implementation)

### W1: Task 3.1 ("Wire `/` key in `input.rs`") mostly redundant

The `/` autocomplete trigger is already wired in `input.rs`:

- `input.rs:160-169` — char insert updates `autocomplete.update_filter()` and sets `autocomplete.visible`
- `input.rs:171-180` — backspace does the same update
- `input.rs:66-72` — Tab completion already works
- `input.rs:182-193` — Arrow keys and Esc navigation already work

Only `render_autocomplete()` call remains unwired. Task 3.1 should be scoped down to verifying the existing trigger path, not implementing it from scratch.

Note: bare `/` on empty input triggers search (`input.rs:61-63`), not autocomplete. Autocomplete activates only after the first character following `/`. This is likely intentional but should be confirmed.

### W2: Diff detection heuristic underspecified

D3 says "lines starting with `@@`, `+`, `-`, ` ` after a `@@ ... @@` header" but doesn't define:

- What threshold constitutes "looks like a diff"? (one `@@` line? Multiple?)
- Does it check all lines or just early ones?
- How to handle empty output (shouldn't crash)
- How to handle non-diff text that coincidentally starts with `@` (e.g. `@mention`)
- The `render_diff` function also handles `diff --git` header lines — should detection match?

Define a function contract: signature, input, return type, and edge case behavior.

### W3: Tool spinner integration point ambiguous

`render_live_assistant_header()` in `activity_line.rs` is called from `messages.rs:84` as part of the scrollable message flow. `render_tool_spinners()` also returns `Option<Line>` and lives in the same file. Two equally valid integration points:

1. Modify `render_live_assistant_header()` to call `render_tool_spinners` internally when `active_tools` is non-empty (fall-through behavior)
2. Add a separate call to `render_tool_spinners()` in `messages.rs:render_messages()` before or after the activity line

Choice affects ordering: do spinners replace the thinking glimmer or supplement it?

### W4: `render_hints` doesn't match full spec requirements

The command-hints spec requires hints for all screen modes (Prompt, Search, Transcript). But `render_hints:231-244` only checks `state.status` (idle/streaming/cancelling) and `state.text_area.mode()` — it never checks `state.screen`. Search-mode hints are missing. This is a pre-existing gap between spec and function, not introduced by this proposal.

## Minor notes

### M1: Proposal wording "pending_toasts" vs code "notifications"

Proposal says "Wire `render_toasts()` to `pending_toasts` from `SessionView`". The actual data flow is: `SessionView.pending_toasts` → `sync_from_active_view()` → `Notification` structs → `state.notifications`. `render_toasts()` reads `state.notifications`. The data path is correct; just the wording is imprecise.

### M2: `render_hints` position ambiguous

Proposal says "status bar area" but `render_status()` already consumes the full 1-row status bar chunk. Hints need either a separate row below the status bar or an extension of the status bar chunk with internal splitting. Tasks don't pick.

### M3: Permission spec-datamodel mismatch

Permission-prompt spec requires diff preview on file writes. But `PendingPermissionView` has only `label` and `description` fields — no `diff` field. `render_permission_prompt()` also doesn't render diffs. The spec describes a capability the data model doesn't yet support.

### M4: `render_hints` already has access to width via `area.width`

The design says "width-gated" and tasks say "add width check". `render_hints` receives a `Rect` area from the caller — width check is the caller's responsibility (only allocate a chunk when the terminal is wide enough).

## Verdict

Proposal is structurally sound: correct inventory of dead code, honest risk assessment, sensible ordering. Two blocking design gaps (B1, B2) need resolution before implementation starts. The 3 additive widgets (spinners, hints, toasts) are straightforward. Diff renderer needs a detection heuristic spec (W2). Permission widget replacement has the most open questions (B1 + M3).
