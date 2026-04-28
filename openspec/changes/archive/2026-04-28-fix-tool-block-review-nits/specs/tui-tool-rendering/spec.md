## MODIFIED Requirements

### Requirement: Tool use header rendering code quality
The `render_tool_use_line()` function SHALL have clear control flow where the subsequent-lines loop is contained within the first-line conditional block. The fallback path for empty rendered content SHALL include a comment explaining it uses the raw tool name intentionally.

#### Scenario: Control flow clarity
- **WHEN** reading `render_tool_use_line()` source code
- **THEN** the `for line in iter` loop SHALL be inside the `if let Some(first) = iter.next()` block
- **AND** no statement after the `if let` block SHALL depend on `iter` state

#### Scenario: Fallback path documentation
- **WHEN** `rendered.is_empty()` is true
- **THEN** the code SHALL include a comment explaining this fallback uses raw `name` (not user-facing name) because no tool rendering method produced content
