## Context

Two audits identified seven issues in the bootstrap branch:

1. **`ToolResultKind::Canceled` / `Rejected` are dead code** — defined in `tools/call.rs`, rendered in `tool_block.rs`, but never produced by session execution.
2. **`user_facing_name()` has no input param** — CC's `userFacingName(input)` receives parsed tool input for context-aware display. kloud's is always static.
3. **`looks_like_diff()` false-positives** on prose containing `@@` patterns.
4. **Syntect LazyLock blocks first render** — `SYNTAX_SET` / `SYNTECT_THEME` lazily load on first code block, causing a visible frame drop.
5. **`_server_name` param dead** in `render_tool_use_line()` after rendering refactor.
6. **`DisplayBlock` lost `PartialEq+Eq`** due to `ratatui::text::Line<'static>` not implementing them.
7. **Test cleanup** — `// width=1` debugging comments in `read.rs` tests.

## Goals / Non-Goals

**Goals:**
- Remove or wire `Canceled`/`Rejected` variants so rendering branches are reachable
- Add `input` parameter to `user_facing_name()` matching CC's protocol
- Tighten `looks_like_diff()` to require contiguous diff lines after hunk header
- Pre-warm syntect at app startup to avoid first-render jank
- Restore or remove `server_name` display in tool use rendering
- Clean test artifacts

**Non-Goals:**
- Full CC tool rendering protocol (renderToolUseTag, renderToolUseRejectedMessage, etc.) — those are feature additions for a later change
- PartialEq restore for DisplayBlock — ratatui::Line doesn't support it, and no callers currently rely on it
- Multi-server tool support beyond wiring the existing `server_name` field

## Decisions

### 1. Remove `Canceled` and `Rejected` variants (not wire them)

**Chosen**: Remove `Canceled` and `Rejected` from `ToolResultKind`, keeping `Success` and `Error`.

**Rationale**: No execution path produces these outcomes. kloud's session loop doesn't have tool-cancellation (no Ctrl+C mid-tool) or permission-rejection flows yet. Adding dead production code just to make rendering branches live is wrong order. When cancellation or rejection flows are added, those variants can be reintroduced with proper wiring.

**Alternative considered**: Wire `Canceled` via `interruptBehavior()` and `Rejected` via permission denial. Rejected because: no interrupt infrastructure exists, no permission system exists, adding stubs that call into nothing is worse than clean removal.

### 2. `user_facing_name(&self, input: &serde_json::Value)` — add input param

**Chosen**: Change signature to `fn user_facing_name(&self, input: &serde_json::Value) -> String`.

**Rationale**: CC's `userFacingName(input)` receives parsed args. This lets tools like Bash show "Bash (git status)". Default impl ignores the param (`self.name().to_string()`). Current tools (Read, Write, Echo) pass input but ignore it — correct for now.

**Call-site impact**: `tool_registry.get(&name).map(|t| t.user_facing_name(&input))` in `session/tools.rs` and `session/stream.rs`.

### 3. Diff detection: require contiguous diff context lines

**Chosen**: After finding a hunk header, scan forward for at least one line matching `^[ +-]` within the next 3 lines.

**Rationale**: A standalone `@@` in prose (e.g., email addresses like `user@@host`) won't have diff-context lines. The hunk-header regex check already filters most noise; adding the context-line requirement eliminates the remaining false-positives.

### 4. Pre-warm syntect in `main.rs`, not in `LazyLock`

**Chosen**: Call `syntect::parsing::SyntaxSet::load_defaults_newlines()` and `syntect::highlighting::ThemeSet::load_defaults()` during app init, before the TUI starts. Store in `TuiState` or pass via a `once_cell` that gets eagerly initialized.

**Rationale**: Syntect loads ~50ms of syntax definitions. Doing this lazily on first code-block render causes a visible frame drop. Pre-warming at startup hides the cost during the terminal handshake.

### 5. Restore `server_name` display in tool use header

**Chosen**: Use `server_name` when present: `⏺ server - ToolName(input)`. When `None`, just `⏺ ToolName(input)`.

**Rationale**: CC renders `{serverName} - {name}` for MCP tools. kloud already stores `server_name` on `DisplayBlock::ToolUse`. The param was suppressed during the rendering refactor. Restoring it is trivial and correct.

## Risks / Trade-offs

- **Cancelled/Rejected removal**: If a cancellation flow is added next sprint, these variants need to be re-added. Mitigation: removal is a 3-line delete; re-adding is equally cheap. The spec in `cc-tool-rendering` keeps the Canceled/Rejected rendering scenarios as aspirations, not active requirements.
- **user_facing_name signature change**: Breaking for all `Tool` impls. Mitigation: only 3 built-in tools exist, all compile-time checked.
- **Syntect pre-warm cost**: ~50ms at startup. Acceptable — it's currently paid on first code-block render anyway, which is more noticeable.

## Open Questions

- None. All issues have clear resolution paths from audit findings.
