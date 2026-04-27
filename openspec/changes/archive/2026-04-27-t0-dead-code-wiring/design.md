## Context

6 widget render functions exist in `src/ui/tui/widgets/` but are never called. Each is
annotated `#[allow(dead_code)]`. The top-level `widgets::render()` in `mod.rs` orchestrates
all rendering but doesn't invoke these functions. Some features (permission, autocomplete)
have duplicate inline implementations in other files that the wired widgets should replace.

All 6 widgets already have corresponding specs in `openspec/specs/`. This change activates
them — no new rendering logic needed.

## Goals / Non-Goals

**Goals:**

- Eliminate all 6 `#[allow(dead_code)]` annotations
- Each widget's output visible in the running TUI
- No behavior regressions in existing render path
- Perfectly replicate original CC terminal UI behavior for each wired widget

**Non-Goals:**

- Visual polish beyond CC parity (that's T2)
- Module restructuring (that's T1)
- New rendering code — all widgets are already written
- Protocol changes — all data already flows through existing types
- Syntax-highlighted diffs (CC uses NAPI ColorDiff; simplified color-only is fine)

## Decisions

### D1: Wire incrementally, one widget at a time (CORRECTED)

Each widget has an independent wiring point. Doing them one at a time means each step is
small, testable, and easy to revert if something breaks. Order by risk (lowest first):

1. Tool spinners → after activity header in message flow (additive)
2. Keybinding hints → status bar area (additive, width-gated)
3. Toast notifications → conditional row above input (additive, single-queue)
4. Diff renderer → tool result blocks (additive, conditional on diff detection)
5. Autocomplete popup → overlay above input (render wiring only; trigger already wired)
6. Permission prompt → inline in message flow (replaces inline rendering, highest risk)
7. Layout → dynamic conditional chunks in `mod.rs` (structural change, after all wired)

**Why this order**: Steps 1-4 are purely additive — they add new visual elements without
changing existing rendering. Step 5 is render-only (trigger already wired in `input.rs:160-193`).
Step 6 replaces existing inline code. Step 7 restructures the layout to accommodate all
new elements with dynamic chunks matching CC's flexbox behavior.

### D2: Signature changes — permission only (CORRECTED)

5 of 6 functions have clean signatures that need no changes — just call them from the
right place. One exception:

- **`permission.rs::render_permission_prompt()`** — **MUST change** from
  `fn(frame, theme, area, perm)` (rendering a bordered `Paragraph` into a frame)
  to `fn(theme, perm) -> Vec<Line<'static>>` (returning lines for inline use).
  This matches CC's inline `PermissionPrompt` component which renders in the scrollable
  message flow, NOT as a bordered overlay. Current `Borders::ALL` usage is incorrect per
  CC's behavior.

All other functions keep their existing signatures.

### D3: Diff detection function contract (CORRECTED)

Diff output in tool results is plain text. Detect diff content with `looks_like_diff()`:

```rust
fn looks_like_diff(text: &str) -> bool
```

**Contract:**
- Must find at least one `@@ ... @@` hunk header line
- May optionally check for preceding `diff --git a/ b/` header
- Must handle empty string → `false`
- Must handle false positives: `@mention` is NOT a diff; require `@@` (double at-sign)
  with content between them, matching the pattern `@@ -\d+(?:,\d+)? \+\d+(?:,\d+)? @@`
- Only apply `render_diff()` when `true` — fall back to plain text otherwise
- Width check is the caller's responsibility (not inside this function)

### D4: Permission inline in message flow, NOT overlay (CORRECTED)

CC's `PermissionPrompt.tsx` renders **inline in the scrollable message area** as a Select
component — it is NOT a bordered overlay or modal. The current `permission.rs` widget uses
`Borders::ALL` which creates a bordered box, diverging from CC's behavior.

**Corrected approach:**
1. Refactor `permission.rs::render_permission_prompt()` to return `Vec<Line<'static>>`
   instead of rendering into a frame
2. Remove the `Block::default().borders(Borders::ALL)` wrapper
3. Replace the inline permission code in `messages.rs:88-103` with a call to the
   refactored function
4. Result: identical visual output to current inline code, but from a single source

The existing inline code in `messages.rs:88-103` already matches CC's behavior — it renders
label, description, and `[y]/[n]` action buttons as plain lines in the scroll area. The
widget function just needs to produce the same lines.

### D5: Dynamic conditional chunks for layout (NEW)

CC's `FullscreenLayout.tsx` uses flexbox with `flex: 1` (grow) and `flexShrink: 0` (fixed).
Some overlay elements use absolute positioning. Our ratatui equivalent is **dynamic
conditional chunks**:

**Current `mod.rs` layout** (4 fixed chunks):
```
[title bar: 1] [messages: Min] [input: 3] [status: 1]
```

**Corrected layout** — compute chunk list dynamically based on state:
```
[title bar: 1]              (optional, when show_title_bar)
[messages: Min]              (always)
[notification: 1]            (conditional, when state.notifications is non-empty)
[input: 3]                   (always)
[status bar: 1]              (always, internally split for hints)
```

**Autocomplete** renders as an **overlay above the input area** using absolute positioning
(compute `Rect` with `y.saturating_sub()`), consuming **0 layout rows** — matching CC's
`SuggestionsOverlay` which uses `bottom: "100%"` positioning.

**Notification row**: Only present when `!state.notifications.is_empty()`. Shows **one
notification at a time** (matching CC's `Notifications.tsx` single-queue pattern). Current
`render_toasts()` iterates up to 2 — change to show only `queue.front()`.

**Hints**: Rendered by **internally splitting the status bar area** (1 row split into
left/right spans), NOT by adding a separate layout row. This matches CC's footer where
hints appear as part of the status line.

### D6: Tool spinners after activity header (NEW)

CC renders tool loading indicators inline in the message flow. Our `render_tool_spinners()`
produces `Option<Line>` with `● tool-name X.Xs` per active tool.

**Integration point**: After the activity header in `messages.rs`, **before** the permission
prompt. The line is appended to the message `Vec<Line>` in the scroll area. This matches
CC's `SpinnerAnimationRow` which renders as a row in the scrollable message list, not in
a fixed position outside the scroll area.

Specifically in `messages.rs:build_lines()`:
1. Render activity header (existing, line 83-85)
2. Render tool spinners (new, after line 85)
3. Render permission prompt (existing, line 88-104)

## Risks / Trade-offs

- **[Risk] Permission regression** → Inline permission works today; widget replacement
  must produce identical output. Mitigation: refactored `Vec<Line>` function produces
  same lines as current inline code at `messages.rs:88-103`. Test with `tui_showcase.rs`.
- **[Risk] Layout chunk arithmetic** → Dynamic conditional chunks mean index offsets change
  depending on which elements are active. Mitigation: compute indices programmatically
  after building constraint list, don't hardcode index values.
- **[Risk] Autocomplete overlay positioning** → Must compute Rect above input area
  correctly. Mitigation: existing `render_autocomplete()` already uses `y.saturating_sub()`
  positioning — verify it works with new dynamic layout.
- **[Trade-off] Diff detection is heuristic** → `looks_like_diff()` may false-positive on
  text containing `@@`. Mitigation: require full hunk header pattern `@@ -\d+ \+\d+ @@`
  to reduce false positives.
- **[Trade-off] PendingPermissionView lacks diff field** → CC shows file diffs inside
  permission prompts, but our `PendingPermissionView` only has `label` + `description`.
  Wire what we have now, note diff-in-permission as aspirational for future spec expansion.
