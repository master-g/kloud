## 1. Additive Widgets (low risk)

- [ ] 1.1 Wire `render_tool_spinners()` into `activity_line.rs` — call from activity line render when `active_tools` is non-empty, remove `#[allow(dead_code)]`
- [ ] 1.2 Wire `render_hints()` into status bar area in `layout.rs` — add width check, call when terminal wide enough, remove `#[allow(dead_code)]`
- [ ] 1.3 Wire `render_toasts()` into `layout.rs` — read `pending_toasts` from `TuiState`, render above input area, remove `#[allow(dead_code)]`

## 2. Diff Renderer (conditional activation)

- [ ] 2.1 Add diff detection helper — function that checks if tool result output starts with `@@` hunk markers
- [ ] 2.2 Wire `render_diff()` into tool result rendering in `tool_block.rs` — use detection helper, call `render_diff()` for diff output, plain text fallback otherwise, remove `#[allow(dead_code)]`

## 3. Autocomplete Popup (input wiring)

- [ ] 3.1 Wire `/` key in `input.rs` to activate `AutocompleteState` when input starts with `/`
- [ ] 3.2 Wire `render_autocomplete()` into `layout.rs` below input area when `AutocompleteState` is active, remove `#[allow(dead_code)]`
- [ ] 3.3 Verify Tab/Arrow/Escape navigation works through existing `AutocompleteState` logic

## 4. Permission Dialog (replacement)

- [ ] 4.1 Wire `render_permission_prompt()` into message rendering path in `widgets/mod.rs` or `layout.rs`, remove `#[allow(dead_code)]`
- [ ] 4.2 Delete inline permission rendering in `messages.rs` (lines ~87-104)
- [ ] 4.3 Verify y/n key handling still sends `UiAction::PermissionResponse`

## 5. Verification

- [ ] 5.1 Run `cargo clippy -- -W warnings` — confirm zero `dead_code` warnings for wired functions
- [ ] 5.2 Run `cargo run --example tui_showcase` — verify all 6 widgets render correctly
- [ ] 5.3 Run `cargo test` — confirm no regressions
