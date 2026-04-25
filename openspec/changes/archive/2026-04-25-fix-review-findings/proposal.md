## Why

Code review identified 3 real issues in the `extend-tui-showcase` change:
1. Permission prompt blocks Ctrl+C — user can't quit while permission is shown
2. `[toast]` prefix leaks into transcript system messages as visible text
3. `last_seen_message_count` breaks silently when message list shrinks (view normalization)

## What Changes

- Add Ctrl+C escape hatch to permission prompt key handler in `input.rs`
- Move toast detection to a separate mechanism: either strip `[toast]` from system message content or use a dedicated `SessionEvent` variant
- Guard `last_seen_message_count` against shrinkage in `state.rs`

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `tui-input`: Ctrl+C allowed during permission prompt
- `notification-toast`: toast trigger no longer leaks prefix into transcript

## Impact

- `src/ui/tui/input.rs` — permission prompt key handler
- `src/ui/tui/state.rs` — `sync_from_active_view` toast detection + message count guard
- `src/app/session/run.rs` — `/notify` slash command content
