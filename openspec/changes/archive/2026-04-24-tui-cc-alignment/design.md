## Context

Kloud's TUI module (`src/ui/tui/`) uses ratatui with a centralized `TuiState` and widget-based rendering. The session publishes `SessionView` snapshots via `AppEvent::View`, and the TUI re-renders at 20 FPS. Current widgets handle messages, activity lines, input, status bar, and tool blocks.

Claude Code's Ink/React TUI provides rich visual feedback: cost tracking, file diff previews, permission prompts, command autocomplete, and contextual hints. These features are essential for a production-feeling agent CLI.

The codebase already has `PendingPermissionView` in the view model (unused), `SessionEvent::UsageUpdated` with token counts, and slash command handling in the session. The design leverages these existing hooks.

## Goals / Non-Goals

**Goals:**
- Display session cost (USD) and duration in the status bar
- Render inline permission prompts with accept/deny and file diff preview
- Show unified diff output for file write/edit tool results
- Add slash command autocomplete dropdown below input
- Render compact tool spinner (● tool-name 1.2s) during tool execution
- Display transient notification toasts for system events
- Show contextual keybinding hints below the input area

**Non-Goals:**
- Exact pixel-for-pixel CC replication (different rendering backends)
- Mouse support or text selection (deferred to future work)
- Image rendering or clipboard paste support
- Undo/redo in the text area
- Rate limit tracking from API headers (no API access in TUI layer)
- Changes to the LLM client or session runtime beyond data flow additions

## Decisions

### D1: Cost computation in the store from accumulated usage

**Decision**: `SessionStore` accumulates `input_tokens` and `output_tokens` from `UsageUpdated` events. A `CostTracker` struct computes cost using a `HashMap<ModelName, PricingTier>` pricing table. The `SessionView` exposes `session_cost: f64` and `session_duration: Duration`.

**Rationale**: Cost is derived from existing data. No new events needed — just computation on top of `UsageUpdated`. Pricing table is static config loaded once.

**Alternative**: Compute cost in the TUI state. Rejected — cost is session-level data, not presentation state.

### D2: Permission prompt as a modal overlay widget

**Decision**: When `SessionView::pending_permission` is `Some(...)`, the message area renders a permission prompt widget inline (below the last assistant message). It shows the label, description, and two buttons: `[y] Allow / [n] Deny`. For write operations, a diff preview is shown above the buttons.

**Rationale**: CC uses inline prompts, not modal dialogs. Easier to implement in ratatui's layout system. The input area remains focused for key handling.

**Alternative**: Full-screen modal overlay. Rejected — breaks the chat flow and is more complex to implement.

### D3: Diff renderer as a standalone widget

**Decision**: A `DiffRenderer` takes a unified diff string and renders it as colored lines with a line-number gutter. Used by both the permission prompt (preview) and tool result blocks (display).

**Rationale**: Reusable across contexts. Unified diff format is the standard. No need for a full diffing library — the tool results already contain diff text.

### D4: Autocomplete as a filtered list in TextArea state

**Decision**: `TuiState` gains an `autocomplete` field tracking the current filter and selection index. When the input starts with `/`, the autocomplete list is populated from a static command list. Arrow keys navigate, Tab confirms, Esc dismisses.

**Rationale**: Minimal state addition. The static command list already exists in `SLASH_COMMANDS`. No new events needed — purely TUI-local state.

### D5: Compact tool spinner replaces activity line during tool execution

**Decision**: When `active_tools` is non-empty, the status line shows a compact spinner: `● tool-name 1.2s` per active tool. The full activity line (shimmer + verb) continues for thinking/responding states.

**Rationale**: CC uses this compact pattern for tool execution. The existing activity line is better for thinking/responding. Best of both worlds.

### D6: Notifications as a Vec with auto-dismiss

**Decision**: `TuiState` gains a `notifications: Vec<Notification>` field. Each notification has a `created_at: Instant` and a `DismissPolicy::After(Duration)`. The render loop checks for expired notifications and removes them. New notifications are pushed by `AppEvent::View` changes or local state changes.

**Rationale**: Simple timer-based dismiss. No additional timer wheel or event needed — the 20 FPS tick loop handles expiry checks.

### D7: Command hints as a static rendered section below input

**Decision**: A `render_hints` function reads the current input mode, screen, and streaming state to produce a one-line hint string. Rendered below the input area as dimmed text.

**Rationale**: Zero state cost. Purely a rendering function. Hints change based on context (idle vs streaming vs search vs transcript).

## Risks / Trade-offs

- **[Status bar width]** — Adding cost, duration, and tool spinners to the status bar may exceed narrow terminal widths. → Mitigation: truncate or hide sections when width < 80 columns; show compact form (cost only).
- **[Permission prompt key handling]** — Adding `y/n` keys for permission while input is focused may conflict with text editing. → Mitigation: permission prompt captures keys only when `pending_permission` is active; normal input keys are suppressed during this state.
- **[Pricing table accuracy]** — Hardcoded pricing may become stale. → Mitigation: pricing table is a simple `HashMap` that can be loaded from config or API later. Start with current model pricing.
- **[Autocomplete flicker]** — Rendering the dropdown on every keystroke may cause visual noise. → Mitigation: only show dropdown when there are ≥1 matches and input starts with `/`.
- **[Notification overlap]** — Multiple notifications stacking may cover message content. → Mitigation: render at most 2 visible notifications; older ones auto-dismiss faster.
