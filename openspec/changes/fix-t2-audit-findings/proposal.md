## Why

Two audits (code review of bootstrap branch + CC fidelity comparison against `docs/cc/` source) found seven issues ranging from dead code to protocol mismatches. Fixing these before T2 (message rendering) continues prevents tech-debt accumulation and ensures the tool rendering protocol stays aligned with CC.

## What Changes

- **Wire `ToolResultKind::Canceled` and `ToolResultKind::Rejected`** into execution paths or remove the dead variants. Currently only `Success`/`Error` are produced; rendering branches in `tool_block.rs` for `Canceled`/`Rejected` are unreachable.
- **Add `input` parameter to `Tool::user_facing_name()`** — CC's `userFacingName(input)` receives parsed input, enabling context-aware display names (e.g., Bash shows "Bash (git status)"). kloud's current signature takes no input, locking all tools to static names.
- **Fix `looks_like_diff()` false-positive** — the heuristic matches any line starting with `@@` containing another `@@`, which catches non-hunk-header prose. Tighten to require contiguous `+`/`-`/` ` lines after the hunk header.
- **Pre-warm syntect syntax set at startup** — `SYNTAX_SET` and `SYNTECT_THEME` are `LazyLock` statics that block the first code-block render. Move initialization to app startup.
- **Wire or remove `_server_name` param** in `render_tool_use_line()` — unused after rendering refactor. Either restore CC's `{server} - {name}` display or remove the dead parameter.
- **Clean up test artifacts** — remove `// width=1` debugging comments from `read.rs` tests.

## Capabilities

### Modified Capabilities

- **cc-tool-rendering**: Canceled/Rejected scenarios currently specified but unreachable — requirement stays, implementation must wire them. `TOOL_CIRCLE` platform split documented.
- **diff-renderer**: Tighten `looks_like_diff()` detection heuristic to reduce false positives.
- **tool-protocol**: `user_facing_name()` signature gains `input: &serde_json::Value` parameter. Tool trait rendering methods documented as matching CC's protocol.
- **tui-tool-rendering**: `render_tool_use_line()` drops dead `_server_name` parameter. Use `server_name` for display prefix per CC.

## Impact

- **Code**: `src/tools/traits.rs`, `src/tools/call.rs`, `src/tools/builtin/*.rs`, `src/app/session/tools.rs`, `src/app/session/stream.rs`, `src/agent/store.rs`, `src/ui/tui/widgets/tool_block.rs`, `src/ui/tui/widgets/messages.rs`, `src/ui/tui/widgets/diff.rs`, `src/ui/tui/widgets/highlight.rs`, `src/ui/tui/state/mod.rs`, `src/main.rs`
- **Tests**: `src/tools/builtin/read.rs` (test cleanup), `src/agent/store.rs` (store tests)
- **Breaking**: `Tool::user_facing_name()` signature change — all tool impls must update
- **No API/dependency changes**
