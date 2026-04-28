## MODIFIED Requirements

### Requirement: Diff output detection and rendering
The system SHALL detect unified diff content in tool result blocks and render it using
`render_diff()` instead of plain text. Detection SHALL first find a line matching the hunk
header pattern `@@ -\d+(,\d+)? \+\d+(,\d+)? @@`, then SHALL verify at least one of the
next 3 lines starts with `+`, `-`, or ` ` (space) to confirm it is genuine diff output.

#### Scenario: Tool result contains unified diff
- **WHEN** a tool result block output contains lines starting with `@@`, `+`, `-`, and ` ` in contiguous hunk blocks
- **THEN** the system SHALL pass the output to `render_diff()` for colored rendering

#### Scenario: Tool result contains @@ in prose
- **WHEN** a tool result block output contains a line starting with `@@` (e.g., `@@ -0,0 +1 @@` inside a non-diff message) but the following 3 lines are not diff-context lines (no `+`/`-`/` ` prefix)
- **THEN** the system SHALL render it as plain text

#### Scenario: Tool result is plain text
- **WHEN** a tool result block output does not contain diff hunk headers at all
- **THEN** the system SHALL render it as plain text (existing behavior unchanged)
