## Context

`tui_showcase.rs` is a 1400-line deterministic demo that uses `DemoClient` (implements `LlmClient`) to script LLM responses. Each demo keyword (e.g. "thinking demo") triggers a specific `stream_plan` branch returning `Vec<(u64, StreamEvent)>`. The `run_showcase` function drives the autoplay via `UiAction` and `CrosstermEvent` injection.

Recently-added components not yet covered:
- Permission prompt (`PendingPermissionView`)
- Diff renderer (`render_diff`)
- Notification toasts (`Notification`, `render_toasts`)
- Search mode (`SearchState`, `Ctrl+F`)
- Stop reasons: `PauseTurn`, `StopSequence`

## Goals / Non-Goals

**Goals:**
- Exercise every TUI widget and state path that has visual rendering
- Cover all `StopReason` variants in the showcase
- Trigger permission prompt flow with y/n response
- Show diff-colored tool output
- Demonstrate search mode lifecycle
- Show notification toasts with auto-dismiss

**Non-Goals:**
- Adding new production features
- Performance testing or benchmarking
- Refactoring existing demo code

## Decisions

1. **Permission demo**: Inject `AppEvent::View` with `pending_permission` set via `UiAction::SlashCommand("permission")` — simpler than adding a new stream branch since permissions are server-side state. Actually, since the showcase uses `SessionStore::apply(SessionEvent::PendingPermissionChanged)`, we can trigger it through the store by having the DemoClient return a `PauseTurn` stop reason, then the session driver sets pending_permission. But the simplest approach: add a new showcase-only `UiAction` or use existing `SlashCommand("permission")` to toggle it. We'll use key injection to simulate the permission prompt appearing and y/n response.

2. **Diff demo**: Return tool output containing unified diff text. The existing `render_diff` function colors +/-/@ lines, so we just need a tool result with diff content.

3. **Notification toasts**: The `Notification` struct is created in `TuiState` but there's no server-side event to push them. We'll add a small helper in the showcase that posts notifications via a direct state mutation path — or use a `SlashCommand("notify")` action that the session interprets as a system message.

4. **Search mode**: Use key injection: Ctrl+F → type query → Enter → n/N → Esc. Already partially covered; extend with full cycle.

5. **Stop reasons**: Add `pause_turn_demo` and `stop_sequence_demo` branches to `stream_plan`.

## Risks / Trade-offs

- Showcase runtime will increase ~15-20s with new demos — acceptable for a demo binary
- Permission prompt demo requires careful timing to show the prompt then auto-respond
- Notification toasts need a trigger mechanism; using SlashCommand keeps it self-contained
