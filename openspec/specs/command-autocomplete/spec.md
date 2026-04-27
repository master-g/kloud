## ADDED Requirements

### Requirement: Autocomplete popup rendering and trigger
The system SHALL trigger autocomplete when the user types `/` as the first character in insert mode. The `/` character SHALL be inserted into the text area (NOT consumed by search activation). The popup SHALL render at full height using available space above the input area.

#### Scenario: User types / at input start in insert mode
- **WHEN** user types `/` as the first character in insert mode
- **THEN** `/` SHALL be inserted into the text area
- **AND** `AutocompleteState` SHALL activate and `render_autocomplete()` SHALL render the popup above the input area at full height (not clipped to input chunk height)

#### Scenario: User selects a command
- **WHEN** user presses Tab or Enter on a highlighted autocomplete item
- **THEN** the command SHALL be inserted into the input field
- **AND** the autocomplete popup SHALL close

#### Scenario: User dismisses autocomplete
- **WHEN** user presses Escape while autocomplete is visible
- **THEN** the autocomplete popup SHALL close without inserting text

#### Scenario: Popup renders at full height
- **WHEN** autocomplete is active with 4 items (requiring 6 rows: 4 items + 2 border)
- **THEN** the popup SHALL render with height = min(popup_height, available_space_above_input)
- **AND** popup height SHALL NOT be clipped to the input chunk height (3)

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

### Requirement: Search activation key
The system SHALL activate search mode when the user presses `Ctrl+S`, independent of the current input mode. The `/` key SHALL NOT activate search mode.

#### Scenario: User presses Ctrl+S in insert mode
- **WHEN** user presses `Ctrl+S`
- **THEN** the screen SHALL switch to `Screen::Search`
- **AND** the search bar SHALL appear at the top of the messages area

#### Scenario: User presses Ctrl+S in normal mode
- **WHEN** user presses `Ctrl+S` in normal mode
- **THEN** the screen SHALL switch to `Screen::Search`
