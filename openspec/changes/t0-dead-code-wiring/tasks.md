## 1. Additive Widgets (low risk)

- [x] 1.1 Wire `render_tool_spinners()` into `messages.rs` — call after activity header
  render (line ~85), before permission prompt, append returned `Line` to message lines,
  remove `#[allow(dead_code)]` from `activity_line.rs:182`
- [x] 1.2 Wire `render_hints()` into status bar in `layout.rs` — split status bar area
  internally (left: token count/cost, right: hints), add `Screen::Search` case showing
  "Esc back · / search · n next · N prev", add width gate (hide when area < 40 cols),
  remove `#[allow(dead_code)]` from `layout.rs:230`
- [x] 1.3 Wire `render_toasts()` into `layout.rs` — read `state.notifications` (renamed
  from `pending_toasts` per M1), show **only the first notification** (single-queue,
  matching CC's `Notifications.tsx`), remove `#[allow(dead_code)]` from `layout.rs:207`

## 2. Diff Renderer (conditional activation)

- [x] 2.1 Add `looks_like_diff(text: &str) -> bool` helper in `diff.rs` — require at
  least one `@@ -\d+(?:,\d+)? \+\d+(?:,\d+)? @@` hunk header, return `false` for empty
  input, handle `@mention` false positive (must be double `@@`)
- [x] 2.2 Wire `render_diff()` into tool result rendering — in `tool_block.rs` (or
  `messages.rs` tool block path), call `looks_like_diff()` on tool output, use
  `render_diff()` for diff output, plain text fallback otherwise, remove
  `#[allow(dead_code)]` from `diff.rs:12`

## 3. Autocomplete Popup (render wiring only)

- [x] 3.1 Verify autocomplete trigger is wired — confirm `input.rs:160-193` already
  activates `AutocompleteState` on `/` key (this is verification only per W1, no code
  change expected)
- [x] 3.2 Wire `render_autocomplete()` into `mod.rs` — render as overlay above input area
  using absolute positioning (`y.saturating_sub()`), NOT as a layout row. Call when
  `state.autocomplete.is_active`, remove `#[allow(dead_code)]` from `layout.rs:157`
- [x] 3.3 Verify Tab/Arrow/Escape navigation works through existing `AutocompleteState`
  logic — manual test in running TUI

## 4. Permission Prompt (refactor + replacement)

- [x] 4.1 Refactor `permission.rs::render_permission_prompt()` signature — change from
  `fn(frame, theme, area, perm)` to `fn(theme, perm) -> Vec<Line<'static>>`, remove
  `Block::default().borders(Borders::ALL)` wrapper, return the same lines as current
  inline code in `messages.rs:89-103`
- [x] 4.2 Replace inline permission in `messages.rs:88-103` — call refactored
  `render_permission_prompt(theme, perm)` and extend lines, delete the inline
  `if let Some(perm) = &state.pending_permission { ... }` block
- [x] 4.3 Verify y/n key handling still sends `UiAction::PermissionResponse` — key
  handling is in `input.rs`, unaffected by rendering change, but verify manually
- [x] 4.4 Remove `#[allow(dead_code)]` from `permission.rs:12`

## 5. Layout Restructure (after all widgets wired)

- [x] 5.1 Refactor `mod.rs::render()` to use dynamic conditional chunks — build
  `Vec<Constraint>` at runtime: title (optional), messages (Min), notification row
  (Length(1), only when `!state.notifications.is_empty()`), input (Length(3)),
  status (Length(1)). Compute indices programmatically from constraint list.
- [x] 5.2 Wire autocomplete overlay in restructured layout — call
  `layout::render_autocomplete()` using absolute-positioned `Rect` above input chunk
- [x] 5.3 Wire notification row in restructured layout — call
  `layout::render_toasts()` into the conditional notification chunk (when present)
- [x] 5.4 Wire hints into status bar — split status bar area in `render_status()`:
  render existing status content on left, call `render_hints()` on right

## 6. Verification

- [x] 6.1 Run `cargo fmt --all && cargo clippy -- -W warnings` — confirm zero
  `dead_code` warnings for all 6 previously-dead functions
- [x] 6.2 Run `cargo test` — confirm no regressions
- [ ] 6.3 Manual smoke test — run `cargo run`, verify each wired widget appears:
  tool spinners during tool use, hints in status bar, toasts on notification,
  diff rendering on tool diff output, autocomplete on `/`, permission inline prompt
