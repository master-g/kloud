## MODIFIED Requirements

### Requirement: Scroll-up pauses in any status
`scroll_messages_up` SHALL set `auto_scroll_paused = true` regardless of the current assistant status, not only during Streaming/Cancelling. `scroll_up()` SHALL decrement (not increment) the scroll offset.

#### Scenario: Idle scroll-up pauses
- **WHEN** the user scrolls up while the assistant is Idle
- **THEN** `auto_scroll_paused` SHALL be set to true
- **AND** subsequent view updates SHALL NOT reset scroll to bottom

#### Scenario: Scroll-up decreases offset
- **WHEN** `scroll_up(n)` is called with offset = 10 and n = 3
- **THEN** the offset SHALL become 7 (decremented)
- **AND** `auto_scroll_paused` SHALL be true
