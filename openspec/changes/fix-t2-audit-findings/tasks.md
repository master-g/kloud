## 1. ToolResultKind cleanup

- [ ] 1.1 Remove `Canceled` and `Rejected` variants from `ToolResultKind` enum in `src/tools/call.rs`
- [ ] 1.2 Remove corresponding match arms from `render_tool_result_block` in `src/ui/tui/widgets/tool_block.rs`
- [ ] 1.3 Update `cc-tool-rendering` spec scenarios for Canceled/Rejected removal
- [ ] 1.4 Run `cargo check && cargo test` to verify no regressions

## 2. user_facing_name(input) signature

- [ ] 2.1 Change `Tool::user_facing_name()` signature to `fn user_facing_name(&self, input: &serde_json::Value) -> String` in `src/tools/traits.rs`
- [ ] 2.2 Update `EchoTool`, `ReadTool`, `WriteTool` impls to accept `input` param (ignore it, return title-case name as before)
- [ ] 2.3 Update call sites in `src/app/session/tools.rs` and `src/app/session/stream.rs` to pass `&input` to `user_facing_name()`
- [ ] 2.4 Run `cargo check && cargo test` to verify

## 3. Server name display in tool use

- [ ] 3.1 Un-suppress `server_name` in `render_tool_use_line()`: when `Some(name)` and non-empty, render `⏺ name - display_name`; when `None` or empty, render `⏺ display_name`
- [ ] 3.2 Verify rendering in `tui_showcase` example matches expected output
- [ ] 3.3 Run `cargo check`

## 4. Diff detection heuristic fix

- [ ] 4.1 Tighten `looks_like_diff()` in `src/ui/tui/widgets/diff.rs`: after finding hunk header `@@ -\d+ \+\d+ @@`, scan next 3 lines for at least one starting with `+`, `-`, or ` `
- [ ] 4.2 Add a test for `looks_like_diff()` false-positive: prose containing `@@` without diff context
- [ ] 4.3 Add a test for `looks_like_diff()` true-positive: real unified diff output
- [ ] 4.4 Run `cargo test diff`

## 5. Syntect pre-warming

- [ ] 5.1 Replace `LazyLock` statics in `src/ui/tui/widgets/highlight.rs` with `once_cell::sync::OnceCell` or a free function that gets called eagerly
- [ ] 5.2 Call the pre-warm function from `TuiState::new()` or `main.rs` before the TUI event loop starts
- [ ] 5.3 Verify highlight functions still work with `cargo test`

## 6. Test cleanup and final verification

- [ ] 6.1 Remove `// width=1` comments from tests in `src/tools/builtin/read.rs`
- [ ] 6.2 Run full verification: `cargo fmt --all && cargo clippy -- -W warnings && cargo test`
- [ ] 6.3 Manual visual check: run `cargo run --example tui_showcase` and verify tool blocks render correctly with server name
