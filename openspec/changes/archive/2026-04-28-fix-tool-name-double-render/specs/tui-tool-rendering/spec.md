## MODIFIED Requirements

### Requirement: Tool use header layout
The system SHALL render tool use blocks as a single header line where the status dot prefix is merged into the first rendered line. The system SHALL NOT create a separate header line with the raw tool name.

#### Scenario: Tool with rendered content (normal path)
- **WHEN** `render_tool_use_line()` receives non-empty `rendered` lines
- **THEN** the status dot SHALL be prepended to the first rendered line's spans
- **AND** no separate `dot + name` header line SHALL be created
- **AND** subsequent rendered lines SHALL be indented with `"  "` prefix

#### Scenario: Tool without rendered content (fallback)
- **WHEN** `rendered` is empty
- **THEN** the system SHALL fall back to rendering `dot + display_name` as a single line
- **AND** `display_name` SHALL use `server_name - name` format if server_name is present

#### Scenario: MCP tool with server_name
- **WHEN** `server_name` is `Some(srv)` and `rendered` is non-empty
- **THEN** the first line SHALL be `dot + srv + " - " + rendered_first_line_content`

#### Scenario: Tool name appears exactly once
- **WHEN** a registered tool's `render_tool_use_message()` returns content containing the user-facing name
- **THEN** the rendered output SHALL show the tool name exactly once (no duplication)
