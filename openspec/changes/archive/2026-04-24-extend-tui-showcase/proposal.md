## Why

The `tui_showcase` example covers most core TUI features (thinking, tools, markdown, animations) but several recently-added components lack demo coverage: permission prompts, diff rendering, notification toasts, search mode, and several stop reasons (pause_turn, stop_sequence). Without demos these code paths are only exercised in production, making visual regression testing impossible.

## What Changes

- Add new demo streams: permission prompt flow, diff output via tool result, notification toast triggers, pause_turn stop reason, stop_sequence stop reason
- Add new key-injection phases: full search mode cycle (Ctrl+F → type → Enter → n/N navigation → Esc), notification dismiss timeout verification, permission prompt y/n response
- Extend the `run_showcase` autoplay sequence to include all new demos
- Update the fallback help text to list all available demo keywords

## Capabilities

### New Capabilities

- `tui-showcase-extended`: Covers new demo streams and key-injection phases for permission, diff, toast, search, and additional stop reasons

### Modified Capabilities

- `tui-input`: Search mode key-injection phases added to showcase

## Impact

- `examples/tui_showcase.rs`: New stream methods on DemoClient, new phases in run_showcase, updated fallback help text
- No production code changes — only the example binary
