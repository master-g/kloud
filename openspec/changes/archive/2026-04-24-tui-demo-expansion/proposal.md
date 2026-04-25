## Why

The `tui_showcase` demo only covers ~10 streaming scenarios (thinking, tool use, markdown, shimmer, stall, token counting). Many TUI rendering paths lack visual test coverage: system/progress messages, batch tools, MCP tools, cancel flow, search mode, vim mode, multi-line input, transcript mode, permission prompt, and various stop reasons. A comprehensive auto-running demo would serve as both a visual regression test and a feature showcase.

## What Changes

- Add 6 new `DemoClient` stream scenarios: batch tools, MCP tool (with server_name), MaxTokens stop, Refusal stop, long text (virtual scroll stress), context meter full
- Extend `run_showcase` script with UiAction-driven phases: `/help` (system info), `/unknown` (system error), `/theme light` (theme change), `CancelTurn` (cancel mid-stream), `SetScreen(Transcript)` (transcript mode)
- Add `key_inject_rx` channel to `UiChannels` so the demo can synthesize crossterm keyboard events for search mode, vim mode, multi-line input, Tab collapse, and PgUp scroll pause
- Enable `show_title_bar = true` in the demo to exercise the title bar widget
- Future: permission prompt and progress message demos blocked on M2/M3 infrastructure

## Capabilities

### New Capabilities

- `key-injection`: Channel-based synthetic keyboard event injection into the TUI event loop, enabling scripted demos of interactive UI features (search, vim, multi-line editing)

### Modified Capabilities

- `tui-bugfixes`: The showcase demo script and DemoClient gain new streaming scenarios and scripted phases; no spec-level behavior changes to existing TUI components

## Impact

- `examples/tui_showcase.rs` — major expansion (new stream methods, extended script, key injection)
- `src/ui/backend.rs` — `UiChannels` gains `key_inject_rx` field
- `src/ui/tui/mod.rs` — event loop gains fourth `select!` branch for injected keys
- `src/ui/tui/input.rs` — no changes (reuse existing `handle_event`)
- No breaking API changes; key injection channel is optional (None by default for production)
