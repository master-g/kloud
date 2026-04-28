## MODIFIED Requirements

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
