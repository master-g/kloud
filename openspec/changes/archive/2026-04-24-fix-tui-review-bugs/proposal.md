## Why

Code review of the tui-cc-replica change revealed 3 critical dead-code bugs (virtual scroll never activates, collapse targets wrong message, batch header duplicates), 4 important issues (Tab toggle dead loop, search screen killed by view sync, PageDown premature auto-scroll, daltonized color confusion), and 3 nits. These must be fixed before the TUI features are usable.

## What Changes

- Fix virtual scroll: call `ensure_cache` before `visible_message_range`, correct `msg_idx` offset for filtered slice
- Fix batch tool header: print once per message, not once per ToolUse block
- Fix Tab collapse: populate collapsed state on first Tab press by finding nearest collapsible tool result
- Fix search screen: preserve `Screen::Search` across `sync_from_active_view` calls
- Fix PageDown: only re-enable auto-scroll when actually at bottom
- Fix daltonized theme: separate warning/success by luminance, not just hue
- Add `Default` derive on `SearchState`, remove unnecessary `clone()` in search hot path, fix `scroll_down` `visible=0` footgun

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

_None_ — all changes are bug fixes to existing implementations, no spec-level behavior changes.

## Impact

- `src/ui/tui/widgets/messages.rs` — virtual scroll cache init, msg_idx correction, batch header dedup
- `src/ui/tui/input.rs` — Tab collapse auto-populate, PageDown auto-scroll guard
- `src/ui/tui/state.rs` — SearchState Default derive, screen preservation in sync_from_active_view
- `src/ui/tui/virtual_scroll.rs` — scroll_down visible parameter fix
- `src/ui/tui/theme.rs` — daltonized warning/success color separation
