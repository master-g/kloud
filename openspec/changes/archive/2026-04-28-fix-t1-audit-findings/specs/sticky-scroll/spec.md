## ADDED Requirements

### Requirement: ScrollState jump_to_bottom method
`ScrollState` SHALL expose a `jump_to_bottom()` method that sets the offset to `usize::MAX` without modifying `auto_scroll_paused`. The actual clamping to `max_scroll` SHALL happen at render time.

#### Scenario: Jump to bottom in transcript mode
- **WHEN** user presses End key in transcript mode
- **THEN** `scroll.offset` SHALL be set to `usize::MAX`
- **AND** `auto_scroll_paused` SHALL retain its current value
- **AND** the render loop SHALL clamp offset to `max_scroll` on the next draw

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
