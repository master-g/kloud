## MODIFIED Requirements

### Requirement: Tool block rendering uses pre-rendered content
The `tool_block.rs` rendering functions SHALL consume pre-rendered `Vec<Line<'static>>` from DisplayBlock fields instead of performing content rendering. They SHALL only handle layout concerns: status dot, collapse/expand, and `⎿` prefix.

#### Scenario: Tool use line rendering
- **WHEN** `render_tool_use_line()` is called for a tool use block
- **THEN** it SHALL prepend the status dot (blinking/solid) before the tool name
- **AND** it SHALL append the pre-rendered lines from `DisplayBlock::ToolUse.rendered_use`

#### Scenario: Tool result block rendering
- **WHEN** `render_tool_result_block()` is called for a tool result
- **THEN** it SHALL apply `⎿` prefix and collapse logic to the pre-rendered lines from `DisplayBlock::ToolResult.rendered_result`
- **AND** it SHALL select prefix style based on `ToolResultKind`

#### Scenario: Error result routing
- **WHEN** a tool result has `ToolResultKind::Canceled`
- **THEN** the renderer SHALL render a minimal dim "Canceled" indicator
- **WHEN** a tool result has `ToolResultKind::Rejected`
- **THEN** the renderer SHALL render a dim "Rejected" message
- **WHEN** a tool result has `ToolResultKind::Error`
- **THEN** the renderer SHALL apply `theme.error` style to the rendered lines

#### Scenario: Removal of classify_tool
- **WHEN** the codebase no longer uses `classify_tool()` or `ToolKind`
- **THEN** these SHALL be deleted from `tool_block.rs`

### Requirement: Tool progress display
The TUI SHALL display real-time progress messages emitted by tools during execution.

#### Scenario: Progress message during tool execution
- **WHEN** a `DisplayBlock::ToolUse` has `progress_text: Some(text)` and `status: Running`
- **THEN** the TUI SHALL display the progress text below the rendered tool use lines
- **AND** the progress text SHALL update in place when `progress_text` changes

#### Scenario: No progress emitted
- **WHEN** a tool executes with `progress_text: None`
- **THEN** only the tool use line (with status dot) and rendered content SHALL be displayed
