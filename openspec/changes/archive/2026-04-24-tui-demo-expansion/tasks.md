## 1. Key Injection Infrastructure

- [x] 1.1 Add `key_inject_rx: Option<mpsc::Receiver<crossterm::Event>>` field to `UiChannels` in `src/ui/backend.rs`, with `None` default in `create_ui_channels()`
- [x] 1.2 Add fourth `select!` branch in `src/ui/tui/mod.rs` that polls `key_inject_rx` (using `stream::pending()` when `None`) and passes events through `input::handle_event()`
- [x] 1.3 Verify `cargo check` passes with no warnings

## 2. New DemoClient Stream Scenarios

- [x] 2.1 Add `batch_tools_demo` method: 3 consecutive ToolUse blocks targeting "echo" with `StopReason::ToolUse`
- [x] 2.2 Add `mcp_tool_demo` method: single ToolUse block with server_name populated via stream handler's `server_names_by_id`
- [x] 2.3 Add `max_tokens_demo` method: text stream ending with `StopReason::MaxTokens`
- [x] 2.4 Add `refusal_demo` method: text stream ending with `StopReason::Refusal`
- [x] 2.5 Add `long_text_demo` method: large repeated output (>100 paragraphs) to stress virtual scroll
- [x] 2.6 Add `context_full_demo` method: stream with `input_tokens: 190000` to trigger red context meter
- [x] 2.7 Add keyword dispatch entries in `stream_plan()` for all new demos

## 3. Extended Showcase Script (UiAction Phases)

- [x] 3.1 Add phase: send `SlashCommand { command: "help", args: "" }` → system info message
- [x] 3.2 Add phase: send `SlashCommand { command: "unknown_cmd", args: "" }` → system error message
- [x] 3.3 Add phase: send `SlashCommand { command: "theme", args: "light" }` → theme change, pause, then restore dark
- [x] 3.4 Add phase: send "cancel demo" message, wait 1.5s, send `CancelTurn` → cancel flow
- [x] 3.5 Add phase: send `SetScreen(Screen::Transcript)`, pause, send `SetScreen(Screen::Prompt)` → transcript mode

## 4. Key Injection Demo Phases

- [x] 4.1 Create `key_inject_tx` channel in `main()`, pass to `run_showcase` coroutine and `UiChannels`
- [x] 4.2 Add search demo phase: inject `'/'`, type "think", inject `Enter`, `'n'`, wait, inject `Esc`
- [x] 4.3 Add multi-line input phase: inject text chars, `Shift+Enter`, more text, `Enter`
- [x] 4.4 Add Tab collapse phase: ensure tool result exists, inject `Tab` key

## 5. Configuration and Polish

- [x] 5.1 Set `show_title_bar: true` on `RatatuiBackend` in demo
- [x] 5.2 Tune pacing delays for new phases (maintain natural rhythm)
- [x] 5.3 Run full demo end-to-end and verify all scenarios render correctly
- [x] 5.4 Run `cargo fmt --all && cargo clippy -- -W warnings`
