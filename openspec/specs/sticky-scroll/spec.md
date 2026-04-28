## ADDED Requirements

### Requirement: Sticky scroll default on
The TUI message area SHALL auto-scroll to the bottom whenever content grows beyond the viewport, regardless of the assistant status (Idle, Streaming, Cancelling). Auto-scroll SHALL only be suppressed when `auto_scroll_paused` is true. Scroll position SHALL be tracked exclusively through VirtualScroll.

#### Scenario: Idle state shows latest content
- **WHEN** the assistant has finished responding and status is Idle
- **AND** message content exceeds one viewport height
- **THEN** the message area SHALL display the bottom of the content (latest messages)
- **AND** VirtualScroll.scroll_offset SHALL reflect the bottom position

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
`scroll_messages_up` SHALL set `auto_scroll_paused = true` regardless of the current assistant status, not only during Streaming/Cancelling. This operation SHALL route through VirtualScroll.

#### Scenario: Idle scroll-up pauses
- **WHEN** the user scrolls up while the assistant is Idle
- **THEN** `auto_scroll_paused` SHALL be set to true
- **AND** subsequent view updates SHALL NOT reset scroll to bottom

### Requirement: ScrollState jump_to_bottom method
`ScrollState` SHALL expose a `jump_to_bottom()` method that sets a `jump_to_bottom: bool` flag to `true` without modifying `offset` or `auto_scroll_paused`. The render loop SHALL check this flag: when true, set offset to `max_scroll`, clear the flag. `offset` SHALL never hold the sentinel value `usize::MAX`.

#### Scenario: Jump to bottom in transcript mode
- **WHEN** user presses End key in transcript mode
- **THEN** `jump_to_bottom` flag SHALL be set to `true`
- **AND** `offset` SHALL remain unchanged until render
- **AND** the render loop SHALL set offset to `max_scroll` and clear the flag

#### Scenario: No unbounded intermediate offset
- **WHEN** `jump_to_bottom()` is called
- **AND** `offset_u16()` is called before the next render
- **THEN** the returned value SHALL reflect the previous offset (not `u16::MAX`)

### Requirement: ScrollState scroll_down_by method
`ScrollState` SHALL expose a `scroll_down_by(n: usize)` method that increments the offset by `n` without content-height clamping. This is for call sites that do not know the content height (input event handlers).

#### Scenario: Incremental scroll down
- **WHEN** `scroll_down_by(10)` is called with offset = 5
- **THEN** offset SHALL become 15
- **AND** `auto_scroll_paused` SHALL be unchanged

### Requirement: No direct field mutation of ScrollState
All mutation of `ScrollState.offset` SHALL go through methods (`scroll_up`, `scroll_down`, `scroll_down_by`, `reset`, `jump_to_bottom`). Direct field assignment `state.scroll.offset = X` SHALL NOT exist in input handlers or widget code.

#### Scenario: End key uses method
- **WHEN** End key is pressed in any screen mode
- **THEN** `jump_to_bottom()` SHALL be called
- **AND** no direct `offset =` assignment SHALL appear in the handler
