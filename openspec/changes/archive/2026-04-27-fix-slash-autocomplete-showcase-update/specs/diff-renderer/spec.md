## ADDED Requirements

### Requirement: Diff tool result showcase demo
The showcase example SHALL include a demo that produces a tool result containing unified
diff content, exercising the `looks_like_diff()` detection and `render_diff()` rendering
path (not the markdown text path).

#### Scenario: Showcase runs diff tool result demo
- **WHEN** showcase sends prompt "show diff tool demo"
- **THEN** the LLM response SHALL include a ToolUse block followed by a tool result
    containing unified diff output with `@@ -\d+ \+\d+ @@` hunk headers
- **AND** the tool result SHALL be rendered with `render_diff()` (colored add/remove/context lines)
- **AND** the result SHALL NOT be rendered as plain text
