## Why

Code review of the t1-module-refactor diff found several issues: a risky `usize::MAX` scroll sentinel that can cause a visual jump, unused `_content_height`/`_visible_height` parameters creating dead-parameter confusion, missing derives on `AppState`, and minor nits (inline `std::collections::HashMap`, unused `_idx` in showcase). Fixing now prevents subtle scroll glitches and keeps the codebase clean before the refactor lands on main.

## What Changes

- Replace `jump_to_bottom()` sentinel (`usize::MAX`) with a boolean flag that the render loop checks, eliminating the unbounded intermediate offset.
- Remove dead parameters `_content_height` and `_visible_height` from `scroll_messages_down()` (and its caller in `input.rs`).
- Add `#[derive(Debug, Clone)]` to `AppState` for consistency with sibling structs.
- Extract `std::collections::HashMap` import to a `use` statement in `app_state.rs`.
- Remove unused `let _idx = i as usize;` line from `examples/tui_showcase.rs`.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `sticky-scroll`: scroll sentinel strategy changes from `usize::MAX` offset to boolean flag
- `tui-input`: `scroll_messages_down` signature simplified (dead params removed)

## Impact

- `src/ui/tui/state/scroll_state.rs` — `jump_to_bottom`, new field
- `src/ui/tui/state/app_state.rs` — derives, import
- `src/ui/tui/state/mod.rs` — `scroll_messages_down` signature
- `src/ui/tui/input.rs` — caller of `scroll_messages_down`
- `src/ui/tui/widgets/messages.rs` — render-loop scroll handling
- `examples/tui_showcase.rs` — remove unused binding
