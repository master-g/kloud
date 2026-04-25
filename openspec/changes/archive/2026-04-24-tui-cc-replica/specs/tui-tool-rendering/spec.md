## ADDED Requirements

### Requirement: Tool call visual display with status indicators
Each tool call SHALL be displayed with: tool name (colored), input preview (truncated), and status indicator. Status SHALL be: running (blinking `●`), success (green `●`), error (red `●`), cancelled (gray `○`).

#### Scenario: Tool execution lifecycle
- **WHEN** a tool call starts
- **THEN** display `[tool_name] input_preview...` with blinking indicator
- **WHEN** the tool completes successfully
- **THEN** the indicator SHALL change to green `●` and show execution time
- **WHEN** the tool fails
- **THEN** the indicator SHALL change to red `●` and show error message

### Requirement: Collapsible tool output
Tool results SHALL be collapsible. By default, results exceeding 5 lines SHALL be collapsed to show only the first 2 and last 1 lines with a `[N more lines]` indicator. User can press `Tab` on a tool block to toggle collapse.

#### Scenario: Long tool output
- **WHEN** a tool result contains 20 lines
- **THEN** only first 2 lines + `[17 more lines, press Tab to expand]` + last 1 line SHALL be displayed
- **WHEN** user navigates to the tool block and presses `Tab`
- **THEN** all 20 lines SHALL be displayed
- **WHEN** user presses `Tab` again
- **THEN** the output SHALL collapse back

### Requirement: Batch tool grouping
Concurrent tool calls SHALL be visually grouped under a single header showing count and progress (e.g., `3 tools running... (1/3 done)`).

#### Scenario: Three concurrent tools
- **WHEN** three tool calls are issued simultaneously
- **THEN** they SHALL be grouped under a `● Running 3 tools...` header
- **WHEN** one tool completes
- **THEN** the header SHALL update to `● Running 3 tools... (1 done)`
- **WHEN** all complete
- **THEN** the header SHALL show `● 3 tools completed in X.Xs`

### Requirement: Tool type visual differentiation
Different tool types SHALL use distinct visual formatting: `Bash` commands in monospace with `$` prefix, `Read`/`Edit` with file path highlight, `WebSearch` with URL links.

#### Scenario: Bash tool display
- **WHEN** a Bash tool is executed
- **THEN** the command SHALL be displayed with `$ ` prefix in monospace style
- **THEN** stdout and stderr SHALL be visually separated

#### Scenario: File tool display
- **WHEN** a Read or Edit tool is executed
- **THEN** the file path SHALL be highlighted with the `suggestion` color
