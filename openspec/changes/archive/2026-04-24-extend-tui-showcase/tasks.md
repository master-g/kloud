## 1. Permission Prompt Demo

- [x] 1.1 Add permission key handling in input.rs (y/n when pending_permission set) + "permission" slash command handler in session/run.rs
- [x] 1.2 Handle UiAction::PermissionResponse in session idle state (clear permission, emit system message)
- [x] 1.3 In `run_showcase`, add permission demo phase: SlashCommand("permission"), inject 'y' key to accept
- [x] 1.4 In `run_showcase`, add permission deny phase: SlashCommand("permission"), inject 'n' key to deny

## 2. Diff Output Demo

- [x] 2.1 Add `diff_demo` branch to `DemoClient::stream_plan` — returns text stream with fenced diff code block (additions, deletions, headers)
- [x] 2.2 In `run_showcase`, add diff demo phase: send "show diff demo" after existing demos

## 3. Notification Toast Demo

- [x] 3.1 Add "notify" slash command handler in session/run.rs — emits system message with [toast] prefix
- [x] 3.2 Add notification creation in TuiState::sync_from_active_view — detects new system messages with [toast] prefix, creates Notification structs
- [x] 3.3 In `run_showcase`, add toast demo phase: SlashCommand("notify", "Hello from kloud!")

## 4. Search Mode Full Cycle Demo

- [x] 4.1 In `run_showcase`, add full search lifecycle: '/' (activate) → type "thinking" → Enter (submit) → n (next) → N (prev) → Esc (exit)
- [x] 4.2 SearchState match count and wrapped indicator displayed during demo

## 5. Additional Stop Reason Demos

- [x] 5.1 Add `pause_turn_demo` branch to `DemoClient::stream_plan` — returns text ending with `StopReason::PauseTurn`
- [x] 5.2 Add `stop_sequence_demo` branch to `DemoClient::stream_plan` — returns text ending with `StopReason::StopSequence`
- [x] 5.3 In `run_showcase`, add phases for both demos after existing stop reason demos

## 6. Update Help Text and Ordering

- [x] 6.1 Update fallback help text in `stream_plan` to include new demo keywords (diff, pause turn, stop sequence)
- [x] 6.2 Reorder `run_showcase` phases to group new demos logically after existing ones

## 7. Build Verification

- [x] 7.1 Run `cargo check` to verify compilation
- [x] 7.2 Run `cargo clippy -- -W warnings` to verify no new warnings
- [x] 7.3 Run `cargo test` to verify all tests pass (50 passed)
