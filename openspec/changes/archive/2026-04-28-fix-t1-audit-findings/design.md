## Context

T1 module refactor introduced `ScrollState` to consolidate scroll offset tracking and eliminate the redundant `scroll: u16` field. The code review found two scroll API issues:

1. `input.rs:371` directly sets `state.scroll.offset = usize::MAX`, bypassing encapsulation
2. Four call sites pass `usize::MAX` for `content_height` and `visible_height` to `scroll_down()`, causing `max_scroll = 0` and degrading PageDown into jump-to-bottom

## Goals / Non-Goals

**Goals:**
- Fix Encapsulation: route all scroll mutations through `ScrollState` methods
- Preserve incremental scroll-down behavior for PageDown in transcript/idle views
- Clean up unused variable and stale task docs

**Non-Goals:**
- No new capabilities or spec changes
- No VirtualScroll behavior changes
- No input_height formula changes (current formula is functionally correct)

## Decisions

**Decision 1: Add `jump_to_bottom()` to ScrollState**

Replace `state.scroll.offset = usize::MAX` with dedicated method. Sets offset to `usize::MAX` and leaves `auto_scroll_paused` unchanged (the render loop handles clamping to `max_scroll`).

Alternative: `scroll_down(usize::MAX, ...)` — same dummy-param problem. `jump_to_bottom()` is explicit.

**Decision 2: Add `scroll_down_by(n)` for incremental scroll**

New method increments offset without content-height clamping. Only used in transcript/input event handlers where content_height isn't known at call time. The render loop in `messages.rs` already clamps offset to `max_scroll`. Keeps existing `scroll_down(n, content, visible)` for the render path.

**Decision 3: Mark theme_state task as skipped**

Section 5 in the T1 tasks.md is checked complete but `theme_state.rs` doesn't exist. Theme switching lives in `RatatuiBackend.theme`, not in `TuiState`. No fields to extract. Add `[SKIPPED — no theme fields in TuiState]` annotation.

## Risks / Trade-offs

- **`jump_to_bottom()` sets offset to `usize::MAX`** → render loop clamps to `max_scroll` at render time. If scrolling happens without render (can't happen — all events trigger re-render), offset could be stale. Mitigation: ratatui `Paragraph::scroll()` clamps internally anyway.
- **`scroll_down_by` doesn't unpause auto-scroll** → user must scroll to actual bottom to resume. Matches old behavior before T1 ("Don't blindly re-enable auto-scroll" comment). Acceptable — transcript mode has explicit End key for jump-to-bottom.
