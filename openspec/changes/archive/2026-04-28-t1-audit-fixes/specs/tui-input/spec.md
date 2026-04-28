## MODIFIED Requirements

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

## ADDED Requirements

### Requirement: Slash command hints crate-visible
`SLASH_COMMAND_HINTS` SHALL be `pub(crate)` to allow access from `state/mod.rs` without violating visibility intent.

#### Scenario: Cross-module access
- **WHEN** `state/mod.rs` references `input_state::SLASH_COMMAND_HINTS`
- **THEN** the reference SHALL compile without visibility errors
