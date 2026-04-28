## ADDED Requirements

### Requirement: Message timestamp display
The system SHALL optionally display relative timestamps on messages.

#### Scenario: Timestamps enabled
- **WHEN** `show_timestamps` is enabled in config
- **THEN** each message SHALL display a relative time (e.g. `2m ago`) in dim style
- **AND** the timestamp SHALL be right-aligned on the first line of the message

#### Scenario: Timestamps disabled (default)
- **WHEN** `show_timestamps` is not enabled
- **THEN** no timestamps SHALL be displayed (existing behavior)

#### Scenario: Very recent message
- **WHEN** a message was created less than 5 seconds ago
- **THEN** the system SHALL display `now` instead of `0s ago`
