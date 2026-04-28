## MODIFIED Requirements

### Requirement: Offset-based virtual scrolling for message list
The message list SHALL implement virtual scrolling that only renders messages within the visible viewport plus a small buffer. Messages outside the viewport SHALL NOT be rendered. All scroll state SHALL be managed exclusively through VirtualScroll — no duplicate scroll offset fields SHALL exist in TuiState.

#### Scenario: 1000 messages displayed smoothly
- **WHEN** the conversation contains 1000 messages
- **THEN** only visible messages (viewport + 5 buffer above/below) SHALL be rendered per frame
- **THEN** scrolling SHALL be smooth at 20fps with no perceptible lag

#### Scenario: New message auto-scroll
- **WHEN** a new message arrives while scrolled to bottom
- **THEN** the view SHALL auto-scroll to show the new message
- **THEN** auto-scroll SHALL NOT trigger if user has scrolled up

#### Scenario: No duplicate scroll state
- **WHEN** scroll operations are performed via keyboard
- **THEN** all scroll state changes SHALL go through VirtualScroll methods
- **AND** no separate `scroll: u16` field SHALL exist on TuiState
