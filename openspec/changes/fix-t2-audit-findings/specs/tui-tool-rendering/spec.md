## MODIFIED Requirements

### Requirement: Tool spinner display in activity line
The system SHALL render active tool spinners using `render_tool_spinners()` during
tool execution. Spinners SHALL replace or supplement the shimmer animation in the
activity line. Tool names SHALL be retrieved from `state.app.active_tools` and
`state.app.tool_start_times`.

#### Scenario: Tools are executing
- **WHEN** `state.app.active_tools` in `TuiState` contains one or more tool names
- **THEN** `render_tool_spinners()` SHALL render compact spinners with elapsed time from `state.app.tool_start_times`

#### Scenario: No active tools
- **WHEN** `state.app.active_tools` is empty
- **THEN** the activity line SHALL show the standard status (existing behavior unchanged)
