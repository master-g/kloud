## 1. Fix ScrollState API

- [x] 1.1 Add `jump_to_bottom()` method to `ScrollState` — sets `offset = usize::MAX`, leaves `auto_scroll_paused` unchanged
- [x] 1.2 Add `scroll_down_by(n: usize)` method to `ScrollState` — increments offset by `n` without content-height clamping, leaves `auto_scroll_paused` unchanged
- [x] 1.3 `cargo check` passes with new methods

## 2. Fix input.rs scroll call sites

- [x] 2.1 Replace `state.scroll.offset = usize::MAX` (L371) with `state.scroll.jump_to_bottom()`
- [x] 2.2 Replace `state.scroll.scroll_down(n, usize::MAX, usize::MAX)` in transcript event handlers (L355, L363) with `state.scroll.scroll_down_by(n)`
- [x] 2.3 Update `scroll_messages_down()` wrapper to delegate to `scroll_down_by` internally (all callers pass dummy content/visible params)
- [x] 2.4 `cargo check` passes with input fixes

## 3. Fix unused variable in tui_showcase.rs

- [x] 3.1 Fix `let idx = i as usize;` → `let _idx = i as usize;` at `examples/tui_showcase.rs:912`
- [x] 3.2 `cargo check --example tui_showcase` clean (no warnings)

## 4. Fix T1 tasks.md

- [x] 4.1 Update `openspec/changes/t1-module-refactor/tasks.md` section 5 — change all `[x]` to `[ ]` and add `[SKIPPED — no theme fields in TuiState; theme lives in RatatuiBackend.theme]` annotation
- [x] 4.2 Verify no `theme_state.rs` references remain in T1 artifacts (found historical refs in spec/proposal — planning docs, not executable)

## 5. Final verification

- [x] 5.1 `cargo fmt --all && cargo clippy -- -W warnings` clean
- [x] 5.2 `cargo test` all passes (50 passed)
- [x] 5.3 `cargo check --example tui_showcase` clean
