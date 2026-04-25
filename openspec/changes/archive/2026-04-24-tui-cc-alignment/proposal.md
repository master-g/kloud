## Why

Kloud's TUI has solid infrastructure (virtual scroll, text area, theme, animations) but lacks several visual and interaction features that make Claude Code's terminal UI feel polished and informative. The demo showcase covers streaming and tool use, but the real REPL experience is missing: cost tracking in the status bar, diff-style file edit previews, permission prompts with explanations, slash command autocomplete, and the compact "spinner with tool name" activity pattern. Closing these gaps makes kloud a more faithful reference implementation and exercises more rendering paths.

## What Changes

- **Cost tracking display**: Show session cost (USD), duration, and rate-limit status in the status bar, computed from `Usage` data with per-model pricing tables
- **Permission prompt widget**: Render the existing `PendingPermissionView` as an inline prompt with accept/deny/cancel actions, file diff preview for write operations, and explanation text
- **File diff preview**: Add a unified-diff renderer that shows proposed file changes inside tool use blocks (for `write`/`edit` tools), with line-number gutter and +/- coloring
- **Slash command autocomplete**: Show a filtered dropdown of available commands below the input area when the user types `/`, with Tab completion and description text
- **Compact tool spinner**: Replace the full-width activity line with a compact spinner pattern (● + tool name + elapsed time) matching CC's "running tool" indicator
- **Notification toasts**: Render transient notification messages (e.g., "Theme changed", "Permission denied") as styled inline toasts that auto-dismiss after N seconds
- **Command hints bar**: Show contextual keybinding hints below the input area (e.g., "Shift+Enter: New line | Esc: Vim mode | Tab: Collapse")

## Capabilities

### New Capabilities
- `cost-tracker`: Session cost computation from token usage with model pricing tables, duration tracking, and status bar rendering
- `permission-prompt`: Inline permission prompt widget with accept/deny actions, file diff preview, and explanation display
- `diff-renderer`: Unified diff rendering with line-number gutter, +/- syntax coloring, and context lines
- `command-autocomplete`: Slash command autocomplete dropdown with filtering, Tab completion, and description display
- `notification-toast`: Transient notification toast system with auto-dismiss and styled rendering
- `command-hints`: Contextual keybinding hints bar rendered below the input area

### Modified Capabilities
- `tui-tool-rendering`: Add compact tool spinner pattern alongside existing full activity line
- `tui-status-bar`: Add cost, duration, and rate-limit sections to the status bar

## Impact

- `src/ui/tui/widgets/` — new widgets for diff, permission prompt, autocomplete, toasts, hints
- `src/ui/tui/state.rs` — new state fields for cost, notifications, autocomplete selection
- `src/ui/tui/widgets/layout.rs` — status bar expansion, hints bar below input
- `src/ui/tui/widgets/tool_block.rs` — compact spinner mode
- `src/ui/events.rs` — new `UiAction` variants for permission responses
- `src/agent/view.rs` — expanded `SessionView` with cost and notification fields
- `src/agent/store.rs` — cost computation from usage events
- No breaking API changes; all additions are additive
