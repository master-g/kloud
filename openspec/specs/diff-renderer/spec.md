## ADDED Requirements

### Requirement: Unified diff rendering with line numbers
The system SHALL render unified diff content as colored lines with a line-number gutter. Added lines SHALL be green, removed lines SHALL be red, and context/header lines SHALL be default color.

#### Scenario: Multi-hunk diff rendering
- **WHEN** a diff string contains `@@ -1,3 +1,4 @@` hunks with added and removed lines
- **THEN** each hunk renders with a dim `@@ ... @@` header, green `+` lines, red `-` lines, and dim line numbers in the gutter

### Requirement: Diff content truncation for large outputs
The diff renderer SHALL truncate output exceeding a configurable line limit, showing a "N more lines" indicator.

#### Scenario: Large diff truncation
- **WHEN** diff content has 200 lines and the limit is 50
- **THEN** the first 50 lines are rendered followed by "... 150 more lines" in dim text

### Requirement: Diff tool result showcase demo
The showcase example SHALL include a demo that produces a tool result containing unified diff content, exercising the `looks_like_diff()` detection and `render_diff()` rendering path (not the markdown text path).

#### Scenario: Showcase runs diff tool result demo
- **WHEN** showcase sends prompt "show diff tool demo"
- **THEN** the LLM response SHALL include a ToolUse block followed by a tool result containing unified diff output with `@@ -\d+ \+\d+ @@` hunk headers
- **AND** the tool result SHALL be rendered with `render_diff()` (colored add/remove/context lines)
- **AND** the result SHALL NOT be rendered as plain text
