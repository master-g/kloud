## MODIFIED Requirements

### Requirement: ToolResult status enumeration
The system SHALL represent tool execution results using a `ToolResultKind` enum with four variants: `Success`, `Error`, `Canceled`, `Rejected`. The `ToolResult` struct SHALL flatten to `kind: ToolResultKind` + `output: String` (replacing `output: Result<String, String>`).

#### Scenario: Successful tool execution
- **WHEN** a tool executes successfully
- **THEN** `ToolResult` SHALL contain `ToolResultKind::Success` and the output string

#### Scenario: Tool execution error
- **WHEN** a tool fails during execution
- **THEN** `ToolResult` SHALL contain `ToolResultKind::Error` and the error message string

#### Scenario: Tool execution canceled
- **WHEN** a tool is canceled by the user
- **THEN** `ToolResult` SHALL contain `ToolResultKind::Canceled` and a cancellation message

#### Scenario: Tool execution rejected
- **WHEN** a tool call is rejected by permission policy
- **THEN** `ToolResult` SHALL contain `ToolResultKind::Rejected` and a rejection message

### Requirement: Tool trait rendering methods
The Tool trait SHALL include rendering methods with default implementations: `user_facing_name()`, `render_tool_use_message()`, `render_tool_result_message()`. Default implementations SHALL provide generic fallback rendering. These methods SHALL be called at the store layer, not the TUI layer.

#### Scenario: Builtin tool overrides rendering
- **WHEN** a builtin tool implements `render_tool_use_message()`
- **THEN** the store SHALL call that tool's custom rendering and store the result in the DisplayBlock

#### Scenario: Tool uses default rendering
- **WHEN** a tool does not override rendering methods
- **THEN** the store SHALL use default implementations showing tool name and raw input/output
