## REMOVED Requirements

### Requirement: Canceled and rejected results use left indent
**Reason**: `ToolResultKind::Canceled` and `ToolResultKind::Rejected` variants are removed — no execution path produces them. Rendering branches for these variants were dead code. Variants will be re-added when cancellation/rejection execution flows exist.
**Migration**: `ToolResultKind` enum now has only `Success` and `Error`. Spec scenarios preserved in comment for re-addition.

## MODIFIED Requirements

### Requirement: Tool use renders as single-line header
Tool use blocks SHALL render as `⏺ ToolName` on a single line with bold tool name and status dot. When `server_name` is present and non-empty, the display SHALL prefix with `server - ` per CC convention (e.g., `⏺ mcp-server - ToolName`). The system SHALL NOT render bordered containers (`╭╮╰╯`) around tool use blocks.

#### Scenario: Basic tool use rendering
- **WHEN** a tool use block with name "Read" and status Done is rendered
- **THEN** output is `⏺ Read` where `⏺` is colored with success color and `Read` is bold

#### Scenario: Tool use with server name
- **WHEN** a tool use block has `server_name = Some("mcp-github")` and `display_name = "Read"`
- **THEN** output is `⏺ mcp-github - Read` where `Read` is bold

#### Scenario: Tool use with content
- **WHEN** a tool use block has pre-rendered content lines
- **THEN** each content line is indented with 2 spaces below the header line
- **AND** no `│` border characters wrap the content

#### Scenario: Empty tool use
- **WHEN** a tool use block has no content
- **THEN** only the `⏺ name` header line is rendered with no bottom border

### Requirement: Tool status dot preserves animation
Status dot SHALL preserve current color coding and blink animation behavior. The tool circle SHALL use `●` (U+25CF, BLACK_CIRCLE) on all platforms per CC convention. Platform-specific override (`⏺` on macOS) is documented in constants.

#### Scenario: Running tool status
- **WHEN** a tool use block has status Running
- **THEN** `●` dot blinks on/off (alternating every 6 ticks) in `theme.subtle` color

#### Scenario: Done tool status
- **WHEN** a tool use block has status Done
- **THEN** `●` dot is solid in `theme.success` color

#### Scenario: Errored tool status
- **WHEN** a tool use block has status Errored
- **THEN** `●` dot is solid in `theme.error` color

## ADDED Requirements

### Requirement: Tool circle character matches CC
The `TOOL_CIRCLE` constant SHALL be `●` (U+25CF, BLACK_CIRCLE) matching Claude Code's `BLACK_CIRCLE` constant. The previous macOS/other platform split SHALL be removed.

#### Scenario: Tool circle on any platform
- **WHEN** a tool use block is rendered
- **THEN** the status dot is `●` (U+25CF)
