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

**Non-Goals:**

- Visual polish or pixel-perfect CC matching (that's T2)
- Module restructuring (that's T1)
- New rendering code — all widgets are already written
- Protocol changes — all data already flows through existing types

## Decisions

### D1: Wire incrementally, one widget at a time

Each widget has an independent wiring point. Doing them one at a time means each PR is
small, testable, and easy to revert if something breaks. Order by risk (lowest first):

1. Tool spinners → activity line (additive, no replacement)
2. Keybinding hints → status bar (additive, width-gated)
3. Toast notifications → layout (additive, uses `pending_toasts`)
4. Diff renderer → tool result blocks (additive, conditional on diff detection)
5. Autocomplete popup → input + layout (replaces inline partial implementation)
6. Permission dialog → messages (replaces inline rendering, highest risk)

**Why this order**: First 4 are purely additive — they add new visual elements without
changing existing rendering. Autocomplete and permission replace existing inline code,
so they go last after the render path is validated.

### D2: Keep render signatures unchanged

All 6 functions already have clean signatures taking `&TuiState`, `&Theme`, and frame/area
parameters. No signature changes needed — just call them from the right place.

### D3: Diff detection heuristic

Diff output in tool results is plain text. Detect diff content by checking for unified diff
markers: lines starting with `@@`, `+`, `-`, ` ` after a `@@ ... @@` header. Only apply
`render_diff()` when the output looks like a diff — fall back to plain text otherwise.

### D4: Permission widget replaces inline, not supplements

`messages.rs` has inline permission rendering at lines ~87-104. The wired `permission.rs`
widget replaces this entirely — delete the inline version, call `render_permission_prompt()`
instead. No duplicate rendering paths.

## Risks / Trade-offs

- **[Risk] Permission regression** → Inline permission works today; widget replacement
  must match all edge cases (no permission set, empty description). Mitigation: test with
  `tui_showcase.rs` permission demo section.
- **[Risk] Autocomplete state coupling** → `AutocompleteState` in `TuiState` drives the
  popup, but trigger logic is in `input.rs`. Mitigation: existing `AutocompleteState` already
  handles filter/display — just connect the `/` key to toggle it.
- **[Risk] Layout space contention** → Hints, toasts, and autocomplete all compete for
  space near the bottom of the screen. Mitigation: each has distinct rendering area (hints
  in status bar, toasts above input, autocomplete below input).
