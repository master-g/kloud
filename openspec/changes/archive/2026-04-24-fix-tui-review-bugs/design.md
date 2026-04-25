## Context

The `tui-cc-replica` change added virtual scrolling, tool collapse, search, multi-theme support, and batch tool rendering. A code review found 10 bugs ranging from critical dead-code paths to cosmetic nits. All bugs are in existing code — no new features.

## Goals / Non-Goals

**Goals:**
- Fix all 3 critical bugs that break virtual scroll, tool collapse targeting, and batch header display
- Fix all 4 important issues that cause dead loops, search screen death, premature auto-scroll, and color-blind confusion
- Apply 3 nits (Default derive, remove clone, fix visible=0)

**Non-Goals:**
- No new features or refactors beyond what's needed to fix the bugs
- No changes to the agent loop, LLM client, or tool execution pipeline
- No new tests beyond what's minimally needed to verify fixes

## Decisions

### D1: Virtual scroll cache init

**Decision**: Call `ensure_cache` at the top of `render_messages` when message count > 100, before calling `visible_message_range`.

**Rationale**: The cache is never populated today, so `line_heights` is always empty and `visible_message_range` returns `(0,0,0)`. The fix is a single call site — no architectural change needed.

### D2: msg_idx offset correction

**Decision**: When virtual scroll is active, add the `start_idx` from `visible_message_range` to the `enumerate()` index to get the real message index for collapse state lookups.

**Rationale**: The filtered slice starts at `start_idx`, so enumerate gives 0-based indices within the slice, not the full message list.

### D3: Batch header dedup

**Decision**: Move batch tool counting and header rendering above the block iteration loop. Track a `batch_header_printed` flag per message.

**Rationale**: The current code counts ToolUse blocks inside the block loop and prints the header each time. Moving the count outside and printing once is the minimal fix.

### D4: Tab collapse auto-populate

**Decision**: When `collapsed_tools` is empty and Tab is pressed, scan messages for the nearest tool result block with >5 lines and insert it as collapsed (true). If none found, no-op.

**Rationale**: Without initial population, the empty-map early return means Tab never does anything. Auto-populating on first press gives the user a working toggle.

### D5: Search screen preservation

**Decision**: In `sync_from_active_view`, skip `self.screen = active_view.screen` when `self.screen == Screen::Search`.

**Rationale**: The server doesn't know about Search screen — it only sends Prompt/Transcript. Overwriting kills the search session mid-use.

### D6: PageDown auto-scroll guard

**Decision**: Replace the blind `auto_scroll_paused = false` with a call to `scroll_messages_down` which checks if the scroll position is actually at the bottom before re-enabling auto-scroll.

**Rationale**: PageDown should only re-enable auto-scroll when the user has actually reached the bottom of the content.

### D7: Daltonized color separation

**Decision**: Shift daltonized warning to a brighter blue `rgb(120,160,230)` and success to teal-shifted `rgb(78,155,180)`. Increase luminance delta to >30.

**Rationale**: The current 8-point red-channel delta is invisible to color-blind users. Teal vs blue provides luminance separation.

### D8: scroll_down visible=0 fix

**Decision**: Pass `visible_lines` as a parameter to `scroll_down` instead of hardcoding `visible = 0`.

**Rationale**: The hardcoded 0 makes `max_scroll = total_lines`, so auto-scroll never re-enables. The function already takes `total_lines` — adding `visible_lines` is the obvious fix.

## Risks / Trade-offs

- **D4 (Tab auto-populate)**: Scanning all messages on first Tab press is O(n). Acceptable because n is bounded by message count and this only happens once. → Mitigation: limit scan to last 100 messages.
- **D5 (Search preservation)**: If the server sends a screen change that should override Search (e.g., shutdown), the skip could block it. → Mitigation: only skip when `active_view.screen` is Prompt/Transcript, not for other transitions.
- **D7 (Daltonized colors)**: New colors may look slightly different for non-color-blind users. → Mitigation: minimal shift, only affects daltonized themes.
