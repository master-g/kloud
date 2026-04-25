## ADDED Requirements

### Requirement: Compact tool spinner during tool execution
The status bar SHALL show a compact spinner with active tool names and elapsed time when `active_tools` is non-empty. Each active tool SHALL render as `● tool-name X.Xs`.

#### Scenario: Single tool running
- **WHEN** `active_tools` contains `["read"]` and the tool has been running for 1.5 seconds
- **THEN** status bar shows `● read 1.5s` with a spinning animation on the dot

#### Scenario: Multiple tools running
- **WHEN** `active_tools` contains `["echo", "read"]`
- **THEN** status bar shows `● echo 0.3s ● read 1.5s` (or truncated to fit)

### Requirement: Tool spinner replaces activity line during tool execution
When `active_tools` is non-empty, the activity line above the assistant message SHALL show the compact tool spinner instead of the full shimmer verb animation.

#### Scenario: Transition from thinking to tool
- **WHEN** activity line shows "Thinking…" shimmer and then a tool starts executing
- **THEN** the activity line transitions to show `● tool-name 0.0s` compact spinner
