## ADDED Requirements

### Requirement: Tool spinner showcase demo
The showcase example SHALL include a demo that activates multiple tools simultaneously
to exercise the `render_tool_spinners()` function wired in the message flow.

#### Scenario: Showcase runs tool spinners demo
- **WHEN** showcase sends prompt "show tool spinners demo"
- **THEN** the LLM response SHALL include multiple ToolUse blocks
- **AND** the activity line SHALL display compact spinners (`● tool-name X.Xs`) for each active tool
