## ADDED Requirements

### Requirement: Tool use renders as single-line header
Tool use blocks SHALL render as `⏺ ToolName` on a single line with bold tool name and status dot. The system SHALL NOT render bordered containers (`╭╮╰╯`) around tool use blocks.

#### Scenario: Basic tool use rendering
- **WHEN** a tool use block with name "Read" and status Done is rendered
- **THEN** output is `⏺ Read` where `⏺` is colored with success color and `Read` is bold

#### Scenario: Tool use with content
- **WHEN** a tool use block has pre-rendered content lines
- **THEN** each content line is indented with 2 spaces below the header line
- **AND** no `│` border characters wrap the content

#### Scenario: Empty tool use
- **WHEN** a tool use block has no content
- **THEN** only the `⏺ name` header line is rendered with no bottom border

### Requirement: Tool result renders with left indent indicator
Tool result blocks SHALL render with `⎿` (U+23BF) as left indent indicator in dim style. The system SHALL NOT render full bordered containers.

#### Scenario: Basic tool result rendering
- **WHEN** a tool result block with 3 output lines is rendered
- **THEN** each line is prefixed with `⎿ ` (U+23BF + 2 spaces) in `theme.subtle` dim style
- **AND** no `╭───╮` / `╰───╯` border lines are present

#### Scenario: Collapsed tool result
- **WHEN** a tool result block has more than 5 lines and is collapsed
- **THEN** the first 5 lines are shown with `⎿` prefix
- **AND** a `⎿ ... N more lines (Tab to expand)` line follows in `theme.suggestion` style

#### Scenario: Expanded tool result with collapse hint
- **WHEN** a tool result block has more than 5 lines and is expanded
- **THEN** all lines are shown with `⎿` prefix
- **AND** a `⎿ (Tab to collapse)` line follows in `theme.suggestion` style

### Requirement: Canceled and rejected results use left indent
Canceled and rejected tool results SHALL render with `⎿` left indent instead of bordered containers.

#### Scenario: Canceled tool result
- **WHEN** a tool result has kind Canceled
- **THEN** output is `⎿ Canceled` in `theme.inactive` dim style

#### Scenario: Rejected tool result
- **WHEN** a tool result has kind Rejected
- **THEN** output is `⎿ Rejected` in `theme.inactive` dim style

### Requirement: Bash tools use identical rendering to other tools
Bash tool results SHALL use the same rendering style as all other tools. The system SHALL NOT apply special border colors or visual distinction for bash tools.

#### Scenario: Bash tool result rendering
- **WHEN** a tool result for bash command is rendered
- **THEN** it uses the same `⎿` left indent style as other tools
- **AND** no pink border color is applied

### Requirement: Tool result diff highlighting preserved
Tool results that look like unified diffs SHALL render with diff highlighting using the `⎿` left indent wrapper instead of border wrapping.

#### Scenario: Diff output in tool result
- **WHEN** raw output matches diff pattern
- **THEN** diff lines are highlighted (green for additions, red for removals)
- **AND** each line is prefixed with `⎿` instead of wrapped in borders

### Requirement: Tool status dot preserves animation
Status dot SHALL preserve current color coding and blink animation behavior.

#### Scenario: Running tool status
- **WHEN** a tool use block has status Running
- **THEN** `⏺` dot blinks on/off (alternating every 6 ticks) in `theme.subtle` color

#### Scenario: Done tool status
- **WHEN** a tool use block has status Done
- **THEN** `⏺` dot is solid in `theme.success` color

#### Scenario: Errored tool status
- **WHEN** a tool use block has status Errored
- **THEN** `⏺` dot is solid in `theme.error` color

### Requirement: Theme bash_border field removed
The `Theme` struct SHALL NOT have a `bash_border` field. All tool rendering uses the same border/indent style.

#### Scenario: Theme struct after change
- **WHEN** Theme struct is defined
- **THEN** no `bash_border` field exists
