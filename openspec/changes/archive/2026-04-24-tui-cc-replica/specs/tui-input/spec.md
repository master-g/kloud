## ADDED Requirements

### Requirement: Multi-line text input
The input area SHALL support multi-line text entry. Pressing `Enter` SHALL submit only when the cursor is on the first and only line. `Shift+Enter` or `Alt+Enter` SHALL insert a newline.

#### Scenario: Single line submit
- **WHEN** user types text and presses `Enter` on a single-line input
- **THEN** the message SHALL be submitted

#### Scenario: Multi-line with newline
- **WHEN** user presses `Shift+Enter`
- **THEN** a newline SHALL be inserted without submitting
- **WHEN** user then presses `Enter` on a non-first line
- **THEN** a newline SHALL be inserted (not submit)

#### Scenario: Multi-line submit from first line
- **WHEN** input has multiple lines and cursor is on the first line
- **THEN** pressing `Enter` SHALL insert a newline (not submit)
- **WHEN** cursor is on the first line and input is single-line
- **THEN** pressing `Enter` SHALL submit

### Requirement: Input history navigation
The system SHALL maintain a history of previously submitted inputs. `Up` arrow (when cursor at start) SHALL navigate backward through history. `Down` arrow (when cursor at end) SHALL navigate forward.

#### Scenario: Navigate input history
- **WHEN** user presses `Up` with empty input or cursor at start
- **THEN** the previous input SHALL replace current input
- **WHEN** user presses `Down` at the last history entry
- **THEN** the input SHALL be cleared

### Requirement: Slash command menu
When user types `/` as the first character, the system SHALL display a filtered list of available commands. Selection SHALL auto-complete the command.

#### Scenario: Slash command autocomplete
- **WHEN** user types `/he`
- **THEN** a filtered list showing matching commands (e.g., `/help`) SHALL appear above the input
- **WHEN** user presses `Tab` or `Enter` on a selection
- **THEN** the input SHALL be filled with the selected command

### Requirement: Vim mode support
The input area SHALL support a basic Vim mode: `Esc` to enter normal mode, `i` to enter insert mode. Normal mode SHALL support `h`/`j`/`k`/`l` navigation, `dd` to delete line, `x` to delete char.

#### Scenario: Toggle Vim mode
- **WHEN** user presses `Esc` in insert mode
- **THEN** the input SHALL switch to normal mode and cursor SHALL change to block style
- **WHEN** user presses `i` in normal mode
- **THEN** the input SHALL switch to insert mode
