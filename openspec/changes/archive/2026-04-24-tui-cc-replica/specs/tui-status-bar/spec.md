## ADDED Requirements

### Requirement: Bottom status bar with session metadata
The TUI SHALL display a status bar at the bottom of the terminal containing: model name, permission mode, token usage (input/output), cost estimate, and elapsed time.

#### Scenario: Status bar during idle
- **WHEN** the session is idle (not streaming)
- **THEN** the status bar SHALL show: `model_name | mode | tokens_in/tokens_out | $cost | branch:branch_name`

#### Scenario: Status bar during streaming
- **WHEN** the session is streaming
- **THEN** the status bar SHALL show live-updating token counts and elapsed time
- **THEN** the cost SHALL increment in real-time as tokens arrive

### Requirement: Context meter visualization
The status bar SHALL include a visual context meter showing used vs total context window as a progress bar with percentage.

#### Scenario: Context meter at 50%
- **WHEN** 100k of 200k context tokens are used
- **THEN** a progress bar SHALL be 50% filled
- **THEN** the percentage `50%` SHALL be displayed next to the bar

#### Scenario: Context meter color thresholds
- **WHEN** context usage is < 70%
- **THEN** the meter SHALL be green
- **WHEN** context usage is 70-90%
- **THEN** the meter SHALL be yellow/warning
- **WHEN** context usage is > 90%
- **THEN** the meter SHALL be red/error

### Requirement: Worktree indicator
When the session is running inside a git worktree, the status bar SHALL show the worktree name.

#### Scenario: Running in worktree
- **WHEN** the current directory is a git worktree named `feature-auth`
- **THEN** the status bar SHALL include `worktree:feature-auth`

#### Scenario: Not in worktree
- **WHEN** the current directory is the main git repo
- **THEN** no worktree indicator SHALL be shown (only branch)
