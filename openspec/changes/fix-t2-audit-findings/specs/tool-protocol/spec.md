## MODIFIED Requirements

### Requirement: Tool trait rendering methods
The `Tool` trait SHALL provide rendering methods that match Claude Code's `Tool` type protocol:

- `user_facing_name(&self, input: &serde_json::Value) -> String` — SHALL return a human-readable display name. Default implementation SHALL return `self.name().to_string()`. The `input` parameter SHALL contain the tool's parsed arguments for context-aware naming.
- `render_tool_use_message(&self, input: &serde_json::Value, theme: &Theme) -> Vec<Line<'static>>` — SHALL render the tool-use input summary. Default SHALL serialize input to JSON truncated to 80 chars.
- `render_tool_result_message(&self, output: &str, kind: ToolResultKind, theme: &Theme) -> Vec<Line<'static>>` — SHALL render the tool-result output. Default SHALL apply error/inactive styling based on `kind`.

#### Scenario: user_facing_name with input
- **WHEN** `user_facing_name()` is called on a tool with `input = {"path": "README.md"}`
- **THEN** the default implementation SHALL return `self.name()` (ignoring input)
- **AND** tool-specific overrides MAY use input to produce dynamic names

#### Scenario: render_tool_use_message default
- **WHEN** `render_tool_use_message()` is called with JSON input `{"path": "src/main.rs"}`
- **THEN** the default SHALL return a single line with truncated JSON text

#### Scenario: render_tool_result_message for error
- **WHEN** `render_tool_result_message()` is called with `kind = Error`
- **THEN** output lines SHALL be styled with `theme.error`

#### Scenario: render_tool_result_message for success
- **WHEN** `render_tool_result_message()` is called with `kind = Success`
- **THEN** output lines SHALL be styled with `theme.inactive`

### Requirement: ToolResult uses ToolResultKind enum
`ToolResult` SHALL use a `kind: ToolResultKind` field with `Success` and `Error` variants instead of `output: Result<String, String>`. The `output` field SHALL be a plain `String` containing either the success payload or error message text.

#### Scenario: Successful tool result
- **WHEN** a tool execution succeeds
- **THEN** `ToolResult { kind: Success, output: "file contents..." }` SHALL be produced

#### Scenario: Failed tool result
- **WHEN** a tool execution fails
- **THEN** `ToolResult { kind: Error, output: "error message" }` SHALL be produced
