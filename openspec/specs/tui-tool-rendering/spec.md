## MODIFIED Requirements

### Requirement: Tool spinner display in activity line
The system SHALL render active tool spinners using `render_tool_spinners()` during
tool execution. Spinners SHALL replace or supplement the shimmer animation in the
activity line.

#### Scenario: Tools are executing
- **WHEN** `active_tools` in `TuiState` contains one or more tool names
- **THEN** `render_tool_spinners()` SHALL render compact spinners with elapsed time
- **AND** `#[allow(dead_code)]` SHALL be removed from `render_tool_spinners()`

#### Scenario: No active tools
- **WHEN** `active_tools` is empty
- **THEN** the activity line SHALL show the standard status (existing behavior unchanged)
