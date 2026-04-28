## Context

The t1-module-refactor split `state.rs` into `state/{mod,app_state,input_state,scroll_state,anim_state}.rs`. Code review found a risky `usize::MAX` sentinel in `jump_to_bottom()`, dead parameters in `scroll_messages_down()`, and minor style issues.

## Goals / Non-Goals

**Goals:**
- Eliminate `usize::MAX` sentinel in `ScrollState` — replace with `jump_to_bottom: bool` flag
- Remove unused `_content_height`/`_visible_height` params from `scroll_messages_down`
- Fix `AppState` derives and import style to match sibling structs
- Clean up unused `_idx` binding in showcase example

**Non-Goals:**
- No new features
- No API surface changes beyond the param removal
- No changes to animation, input handling logic, or rendering behavior

## Decisions

### 1. Boolean flag instead of usize::MAX sentinel

**Decision**: Add `jump_to_bottom: bool` field to `ScrollState`. `jump_to_bottom()` sets it; render loop checks it before clamping, then clears it.

**Rationale**: `usize::MAX` creates an unbounded intermediate state. If `offset_u16()` is called between `jump_to_bottom()` and the render clamp, the result is `u16::MAX = 65535`, causing a visual jump in any content shorter than 65535 lines. A flag is unambiguous.

**Alternative considered**: Keep sentinel but add a `is_at_bottom()` method. Rejected — still requires callers to know about the sentinel contract.

### 2. Remove dead params from scroll_messages_down

**Decision**: Drop `_content_height`/`_visible_height` from `scroll_messages_down()`. The method body ignores them; render loop handles clamping.

**Rationale**: Dead parameters mislead future callers into thinking the method does clamping.

### 3. AppState derives + import cleanup

**Decision**: Add `#[derive(Debug, Clone)]` to `AppState`. Move `std::collections::HashMap` to a `use` import.

**Rationale**: Consistency with `InputState`, `ScrollState`, etc.

## Risks / Trade-offs

- **Risk**: `jump_to_bottom` flag must be cleared in the render loop. Missing the clear causes infinite-bottom behavior. → Mitigation: single clear point in `messages.rs` render, right after reading the flag.
- **Risk**: Changing `scroll_messages_down` signature is a breaking change for any external caller. → Mitigation: method is `pub(crate)`, no external consumers.
