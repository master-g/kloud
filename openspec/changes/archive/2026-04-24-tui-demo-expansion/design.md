## Context

The `tui_showcase` example is an auto-running demo that exercises the TUI without a real LLM. It uses a `DemoClient` (implements `LlmClient`) that returns scripted `StreamEvent` sequences, and a `run_showcase` coroutine that sends `UiAction::SendMessage` on a timer.

Currently the demo covers: thinking stream, single tool use, tool error, redacted thinking, markdown (basic + enhanced), shimmer animation, stall detection, token counting, and thinking duration.

Missing coverage includes: batch tools, MCP tools, system messages, cancel flow, search mode, vim mode, multi-line input, transcript mode, various stop reasons, context meter at high fill, title bar, and tool collapse toggle.

## Goals / Non-Goals

**Goals:**

- Cover all TUI rendering paths that don't require new infrastructure (permission prompt, progress message are deferred)
- Add a key injection mechanism so the demo can drive interactive UI features (search, vim, multi-line)
- Keep the demo fully self-running — no user interaction required
- Natural pacing: total runtime grows as scenarios are added, no need to gate phases

**Non-Goals:**

- Permission prompt demo (blocked on M2/M3: `Tool::check_permissions()`, `PendingPermissionView` flow)
- Progress message demo (blocked on M2/M3: `onProgress` callback in Tool trait)
- Interactive REPL mode (demo stays scripted)
- Changes to production TUI event handling beyond adding the optional injection channel

## Decisions

### D1: Key injection via mpsc channel in UiChannels

**Decision**: Add `key_inject_rx: Option<mpsc::Receiver<crossterm::Event>>` to `UiChannels`. The TUI event loop adds a fourth `select!` branch that polls this channel. When `None`, the branch is a no-op (`stream::pending()`).

**Rationale**: Minimal invasive change. Production code creates `UiChannels` with `key_inject_rx: None` — zero overhead. Demo creates with `Some(rx)` and the `run_showcase` coroutine holds `key_inject_tx`.

**Alternative considered**: Directly pushing crossterm events via a file descriptor or pipe. Rejected — too platform-specific, fragile.

### D2: DemoClient stream_plan extensions are self-contained

**Decision**: Each new demo scenario is a new method on `DemoClient` returning `Vec<(u64, StreamEvent)>`. The `stream_plan` dispatcher matches on keywords in the user message text.

**Rationale**: Consistent with existing pattern. No architectural change needed. Each scenario is independently testable.

### D3: UiAction-driven phases in run_showcase for non-stream features

**Decision**: The showcase script sends `UiAction` variants directly (not just `SendMessage`) for:
- `SlashCommand { command: "help", args: "" }` → system info message
- `SlashCommand { command: "unknown", args: "" }` → system error message
- `SlashCommand { command: "theme", args: "light" }` → theme change
- `CancelTurn` → cancel during streaming
- `SetScreen(Screen::Transcript)` + `SetScreen(Screen::Prompt)` → transcript mode

**Rationale**: `run_showcase` already holds `action_tx`. These are zero-cost additions.

### D4: Batch tools via multi-ToolUse response

**Decision**: `batch_tools_demo` returns an assistant message with 3 consecutive `ToolUse` content blocks. Each uses a registered tool name (`echo`). The Session will dispatch all 3, producing the batch header "3 tools (3/3)" in the TUI.

**Rationale**: Tests the real `continue_after_assistant_turn` → `extract_tool_calls` → loop dispatch path. No mocking needed since `EchoTool` is registered.

### D5: MCP tool via ContentBlock::ToolUse with server_name

**Decision**: The `DisplayBlock::ToolUse` already has a `server_name: Option<String>` field. The `mcp_tool_demo` sets it to `Some("github-mcp")` via the store's `display_blocks_from_content_blocks` which reads from `server_names_by_id`. We need the `DemoClient` to produce a `ContentBlock::ToolUse` and the stream handler to populate `server_names_by_id`.

**Rationale**: The field exists but is always `None` today. The demo exercises the rendering path for MCP tool names.

## Risks / Trade-offs

- **[Key injection race]** — Injected keys arrive asynchronously; if the demo sends keys while Session is streaming, the TUI might be in a state that doesn't accept the key (e.g., search in non-idle state). → Mitigation: script uses generous delays between phases; key injection phases only during Idle state.
- **[Demo runtime grows to 2-3 minutes]** — Adding ~15 new phases on top of existing 10. → Acceptable: the demo is meant to showcase, not run in CI. Could add `SHOWCASE_PACE_MULTIPLIER=0` for instant mode later.
- **[Batch tools produce real tool results]** — 3 echo calls produce 3 result blocks. This is real execution, not mocked. → Acceptable: echo is fast and deterministic.
- **[Cancel might not produce visible "Cancelling" state]** — The cancel path is fast; the UI might not render the cancelling state visibly. → Mitigation: add a brief delay in the DemoClient's thinking stream so cancel arrives mid-think.
