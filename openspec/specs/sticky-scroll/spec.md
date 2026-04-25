## ADDED Requirements

### Requirement: Sticky scroll default on
The TUI message area SHALL auto-scroll to the bottom whenever content grows beyond the viewport, regardless of the assistant status (Idle, Streaming, Cancelling). Auto-scroll SHALL only be suppressed when `auto_scroll_paused` is true.

#### Scenario: Idle state shows latest content
- **WHEN** the assistant has finished responding and status is Idle
- **AND** message content exceeds one viewport height
- **THEN** the message area SHALL display the bottom of the content (latest messages)
- **AND** `state.scroll` SHALL reflect the bottom position

#### Scenario: Streaming shows latest content
- **WHEN** the assistant is streaming a response
- **AND** content grows beyond viewport
- **THEN** the message area SHALL auto-scroll to bottom continuously

#### Scenario: User scrolls up pauses auto-scroll
- **WHEN** the user presses PageUp or scrolls up
- **THEN** `auto_scroll_paused` SHALL be set to true
- **AND** the message area SHALL stop auto-scrolling to bottom
- **AND** the scroll position SHALL remain at the user's chosen position

#### Scenario: User scrolls to bottom resumes auto-scroll
- **WHEN** the user scrolls down to the bottom of the content
- **THEN** `auto_scroll_paused` SHALL be reset to false
- **AND** auto-scroll behavior SHALL resume

### Requirement: Auto-scroll paused indicator
The "auto-scroll paused" indicator SHALL only be displayed when `auto_scroll_paused` is true AND the assistant is streaming. This avoids visual noise during Idle browsing.

#### Scenario: Indicator during streaming
- **WHEN** the user has scrolled up during streaming
- **THEN** the "↓ auto-scroll paused" indicator SHALL be visible at the bottom of the message area

#### Scenario: No indicator during idle
- **WHEN** the user has scrolled up but the assistant is Idle
- **THEN** no "auto-scroll paused" indicator SHALL be displayed
- **AND** the user can browse history freely

### Requirement: Scroll-up pauses in any status
`scroll_messages_up` SHALL set `auto_scroll_paused = true` regardless of the current assistant status, not only during Streaming/Cancelling.

#### Scenario: Idle scroll-up pauses
- **WHEN** the user scrolls up while the assistant is Idle
- **THEN** `auto_scroll_paused` SHALL be set to true
- **AND** subsequent view updates SHALL NOT reset scroll to bottom
