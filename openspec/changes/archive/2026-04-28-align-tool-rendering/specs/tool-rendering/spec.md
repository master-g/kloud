## ADDED Requirements

### Requirement: Per-tool user-facing name
The Tool trait SHALL provide a `fn user_facing_name(&self) -> String` method that returns a human-readable tool display name. Default implementation SHALL return `tool.name()`.

#### Scenario: EchoTool
- **WHEN** `user_facing_name()` is called on EchoTool
- **THEN** it SHALL return `"Echo"`

#### Scenario: ReadTool
- **WHEN** `user_facing_name()` is called on ReadTool
- **THEN** it SHALL return `"Read"`

#### Scenario: WriteTool
- **WHEN** `user_facing_name()` is called on WriteTool
- **THEN** it SHALL return `"Write"`

#### Scenario: Tool with no override
- **WHEN** `user_facing_name()` is called on a tool that does not override the method
- **THEN** it SHALL return `tool.name()` as fallback

### Requirement: Per-tool use message rendering
The Tool trait SHALL provide a `fn render_tool_use_message(&self, input: &serde_json::Value, theme: &Theme) -> Vec<Line<'static>>` method returning styled lines for the tool use header. Rendering SHALL happen at store layer, not TUI layer.

#### Scenario: EchoTool renders message
- **WHEN** `render_tool_use_message()` is called on EchoTool with input `{"message": "hello"}`
- **THEN** it SHALL return lines containing the message text

#### Scenario: ReadTool renders file path
- **WHEN** `render_tool_use_message()` is called on ReadTool with input `{"path": "src/main.rs"}`
- **THEN** it SHALL return lines containing the file path in file style

#### Scenario: WriteTool renders file path
- **WHEN** `render_tool_use_message()` is called on WriteTool with input `{"path": "foo.rs"}`
- **THEN** it SHALL return lines containing the file path in file style

#### Scenario: Tool with no override uses default
- **WHEN** `render_tool_use_message()` is called on a tool that does not override
- **THEN** it SHALL return a default rendering showing tool name + JSON input truncated to 80 chars

### Requirement: Per-tool result message rendering
The Tool trait SHALL provide a `fn render_tool_result_message(&self, output: &str, kind: ToolResultKind, theme: &Theme) -> Vec<Line<'static>>` method returning styled lines for the tool result. Rendering SHALL happen at store layer, not TUI layer.

#### Scenario: ReadTool result
- **WHEN** `render_tool_result_message()` is called on ReadTool with file content output
- **THEN** it SHALL return lines with the file content (read tool already formats with line numbers in output)

#### Scenario: Error result
- **WHEN** `render_tool_result_message()` is called with `ToolResultKind::Error`
- **THEN** it SHALL return lines using error style (red text)

#### Scenario: Tool with no override uses default
- **WHEN** `render_tool_result_message()` is called on a tool that does not override
- **THEN** it SHALL return default rendering with the raw output text

### Requirement: Pre-rendered content in DisplayBlock
`DisplayBlock::ToolUse` SHALL carry a `rendered_use: Vec<Line<'static>>` field. `DisplayBlock::ToolResult` SHALL carry a `rendered_result: Vec<Line<'static>>` field. These fields SHALL be populated at store layer by calling Tool trait rendering methods.

#### Scenario: ToolUse block carries rendered content
- **WHEN** a ToolUse block is created via `SessionEvent::ToolExecutionStarted`
- **THEN** the event SHALL include pre-rendered lines from `tool.render_tool_use_message()`
- **AND** `DisplayBlock::ToolUse.rendered_use` SHALL contain those lines

#### Scenario: ToolResult block carries rendered content
- **WHEN** a ToolResult block is created via `SessionEvent::ToolExecutionFinished`
- **THEN** the event SHALL include pre-rendered lines from `tool.render_tool_result_message()`
- **AND** `DisplayBlock::ToolResult.rendered_result` SHALL contain those lines

### Requirement: Tool progress callback
The Tool trait `execute()` method SHALL accept an optional progress callback that tools invoke during long-running operations.

#### Scenario: Progress emitted
- **WHEN** a tool invokes the progress callback with a text string
- **THEN** a `SessionEvent::ToolProgress` SHALL be emitted and stored in the matching `DisplayBlock::ToolUse.progress_text`

#### Scenario: No callback provided
- **WHEN** `execute()` is called with `None` for the progress callback
- **THEN** the tool SHALL execute normally without emitting progress
