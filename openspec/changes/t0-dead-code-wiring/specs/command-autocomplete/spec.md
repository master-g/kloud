## MODIFIED Requirements

### Requirement: Autocomplete popup rendering and trigger
The system SHALL wire `render_autocomplete()` to the render path and trigger it when
the user types `/` at the beginning of the input field. Navigation and selection use
existing `AutocompleteState` logic.

#### Scenario: User types / at input start
- **WHEN** user types `/` as the first character in the input field
- **THEN** `AutocompleteState` SHALL activate and `render_autocomplete()` SHALL render
  the dropdown below the input area
- **AND** `#[allow(dead_code)]` SHALL be removed from `render_autocomplete()`

#### Scenario: User selects a command
- **WHEN** user presses Tab or Enter on a highlighted autocomplete item
- **THEN** the command SHALL be inserted into the input field
- **AND** the autocomplete popup SHALL close

#### Scenario: User dismisses autocomplete
- **WHEN** user presses Escape while autocomplete is visible
- **THEN** the autocomplete popup SHALL close without inserting text
