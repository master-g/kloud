## ADDED Requirements

### Requirement: Slash command autocomplete dropdown
The TUI SHALL display a filtered dropdown of available slash commands below the input area when the current input starts with `/`. The dropdown SHALL show command names and summary descriptions.

#### Scenario: Typing slash shows commands
- **WHEN** user types `/` in an empty input
- **THEN** a dropdown appears below the input showing all available commands with their summaries

#### Scenario: Filtered by prefix
- **WHEN** user types `/th`
- **THEN** the dropdown shows only commands starting with "th" (e.g., "theme")

### Requirement: Tab completes autocomplete selection
Pressing `Tab` when the autocomplete dropdown is visible SHALL complete the input with the selected command name and a trailing space.

#### Scenario: Tab completion
- **WHEN** autocomplete dropdown shows "/theme" as the selected item
- **AND** user presses `Tab`
- **THEN** input is replaced with "/theme " and the dropdown closes

### Requirement: Arrow keys navigate autocomplete
`Up`/`Down` arrow keys SHALL navigate the autocomplete selection when the dropdown is visible.

#### Scenario: Navigate and select
- **WHEN** dropdown shows 3 commands and user presses `Down` twice then `Tab`
- **THEN** the third command is completed into the input
