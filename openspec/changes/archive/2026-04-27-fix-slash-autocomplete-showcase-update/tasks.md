## 1. Fix `/` Key Behavior

- [x] 1.1 Remove `SearchActivate` handler for `/` key — delete `input.rs:61-63`
  (`(KeyCode::Char('/'), KeyModifiers::NONE) if state.text_area.is_empty() => Some(UiAction::SearchActivate)`)
- [x] 1.2 Add `Ctrl+S` search activation — add `(KeyCode::Char('s'), KeyModifiers::CONTROL) => Some(UiAction::SearchActivate)`
  at top level of the main match in `input.rs`

## 2. Fix Autocomplete Popup Height

- [x] 2.1 Fix `render_autocomplete()` height calculation — change
  `height: popup_height.min(area.height)` to `height: popup_height.min(area.y)`
  in `layout.rs`

## 3. Update Showcase

- [x] 3.1 Add `tool_spinners_demo()` to `DemoClient` — stream 3 ToolUse blocks
  (bash, read, edit) to exercise `render_tool_spinners()` in the message flow.
  The tool results follow separately.
- [x] 3.2 Add `diff_tool_result_demo()` to `DemoClient` — stream a ToolUse block
  targeting a tool (e.g. "edit_file"), then in `stream_plan()` match the tool result
  ID and return a tool result containing unified diff output with `@@ ... @@` hunk
  headers so `looks_like_diff()` triggers `render_diff()`.
- [x] 3.3 Add new entries to `stream_demos` array — insert "show tool spinners demo"
  and "show diff tool demo" entries after existing demos
- [x] 3.4 Fix search demo key injection — change `/` key injection to `Ctrl+S`
  in the search demo phase (line ~1431 and line ~1563): replace
  `KeyCode::Char('/')` with `KeyCode::Char('s')` + `KeyModifiers::CONTROL`
- [x] 3.5 Fix autocomplete demo — verify `/` key injection now triggers autocomplete
  (no code change needed if 1.1 is fixed; verify manually)

## 4. Verification

- [x] 4.1 Run `cargo fmt --all && cargo clippy -- -W warnings` — confirm zero warnings
- [x] 4.2 Run `cargo test` — confirm no regressions
- [ ] 4.3 Run `cargo run --example tui_showcase` — verify:
  - `/` in idle state shows autocomplete popup (not search)
  - `Ctrl+S` activates search mode
  - Autocomplete popup renders at full height
  - Tool spinners demo shows `● tool-name X.Xs` during tool execution
  - Diff tool demo shows colored diff output in tool result
  - Later interactive phases (autocomplete, search, permission) work correctly
