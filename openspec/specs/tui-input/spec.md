## ADDED Requirements

### Requirement: Multi-line text input
The input area SHALL support multi-line text entry. Pressing `Enter` SHALL submit only when the cursor is on the first and only line. `Shift+Enter` or `Alt+Enter` SHALL insert a newline. `input_height()` SHALL clamp to a minimum of 3 rows even when terminal height is very small.

#### Scenario: Single line submit
- **WHEN** user types text and presses `Enter` on a single-line input
- **THEN** the message SHALL be submitted

#### Scenario: Multi-line with newline
- **WHEN** user presses `Shift+Enter`
- **THEN** a newline SHALL be inserted without submitting

#### Scenario: Small terminal height guard
- **WHEN** terminal height is 1 or 2 rows
- **THEN** `input_height()` SHALL still return at least 3
- **AND** SHALL NOT panic or underflow

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

### Requirement: Permission prompt allows emergency exit
The permission prompt key handler must allow Ctrl+C to exit the application even when the prompt is shown.

#### Scenario: User presses Ctrl+C during permission prompt
- **WHEN** `pending_permission` is Some and `status` is Idle
- **AND** the user presses Ctrl+C
- **THEN** `UiAction::Exit` is returned instead of being blocked
- **AND** the application quits normally

### Requirement: Slash command hints crate-visible
`SLASH_COMMAND_HINTS` SHALL be `pub(crate)` to allow access from `state/mod.rs` without violating visibility intent.

#### Scenario: Cross-module access
- **WHEN** `state/mod.rs` references `input_state::SLASH_COMMAND_HINTS`
- **THEN** the reference SHALL compile without visibility errors

### Requirement: Scroll actions use ScrollState methods
All scroll-triggering key handlers in `input.rs` SHALL route through `TuiState` delegation methods (`scroll_messages_up`, `scroll_messages_down`) or `ScrollState` methods (`scroll_up`, `scroll_down`, `scroll_down_by`, `jump_to_bottom`). Direct field mutation of `scroll.offset` SHALL NOT be used. `scroll_messages_down` SHALL accept only the line count parameter `(n: u16)`.

#### Scenario: PageDown uses scroll_messages_down
- **WHEN** user presses PageDown in prompt mode
- **THEN** `state.scroll_messages_down(10)` SHALL be called
- **AND** no content-height or visible-height parameters SHALL be passed

#### Scenario: End key uses jump_to_bottom
- **WHEN** user presses End in transcript mode
- **THEN** `state.scroll.jump_to_bottom()` SHALL be called
- **AND** no direct `state.scroll.offset = usize::MAX` SHALL be present
