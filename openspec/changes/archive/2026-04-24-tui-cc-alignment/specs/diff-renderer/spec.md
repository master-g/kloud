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
