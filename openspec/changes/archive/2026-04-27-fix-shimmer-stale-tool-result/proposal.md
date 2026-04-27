## Why

Two bugs prevent the TUI showcase from rendering correctly after the first demo:

1. **Shimmer animation only works on the first demo** — `StalledState.last_response_length` accumulates across demos and is never reset. When a new demo starts with `response_char_count = 0`, the stall detector sees no token growth and triggers after 3 seconds, turning the activity line red and masking the shimmer effect. This diverges from Claude Code's `useStalledAnimation` hook, which resets via `mountTime` on every component mount (equivalent to a new query).

2. **All demos after "read demo" show "Read completed..."** — `latest_tool_result()` in the showcase scans ALL messages for tool results. After the read demo completes, the stale `toolu_read_demo` result persists in message history and matches every subsequent request, producing the wrong response. This violates the Anthropic Messages API convention where tool results are always the last user message.

## What Changes

- Reset `StalledState` when status transitions to Streaming (matching CC's component mount reset via `mountTime` ref)
- Fix `latest_tool_result()` to only inspect the last user message (matching Anthropic API convention)
- Both fixes are localized to single functions with no API or protocol changes

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `tui-streaming-animations`: `StalledState` must reset on Idle→Streaming transition to prevent false stall detection across query boundaries (matching CC `useStalledAnimation` mount-time reset)
- `tui-showcase-extended`: `latest_tool_result()` must only check the last user message to avoid stale tool result matches (matching Anthropic API tool result convention)

## Impact

- **Files modified**: `src/ui/tui/state.rs` (StalledState reset), `examples/tui_showcase.rs` (tool result lookup scope)
- **No API changes** — internal state management fix
- **No new dependencies**
- **Risk**: very low — both fixes are isolated, single-function changes, both validated against CC source code
