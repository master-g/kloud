## ADDED Requirements

### Requirement: Tool block bordered container
Tool use and tool result blocks SHALL be wrapped in a bordered container with rounded corners.

#### Scenario: Tool use block with content
- **WHEN** a `DisplayBlock::ToolUse` is rendered
- **THEN** the system SHALL render a `╭─╮` / `│` / `╰─╯` bordered container
- **AND** the tool name SHALL appear in the top border line in purple bold
- **AND** the status dot SHALL appear in the top border line

#### Scenario: Tool result block
- **WHEN** a `DisplayBlock::ToolResult` is rendered
- **THEN** the result content SHALL render inside a bordered container below the tool use block
- **AND** the `⎿` prefix SHALL be replaced by the border structure

#### Scenario: Bash tool output
- **WHEN** the tool name is `bash`
- **THEN** the border color SHALL be pink (bright magenta) instead of default subtle color

#### Scenario: Collapsed tool result
- **WHEN** tool result exceeds collapse threshold and is collapsed
- **THEN** the border SHALL contain only visible lines plus collapse hint
