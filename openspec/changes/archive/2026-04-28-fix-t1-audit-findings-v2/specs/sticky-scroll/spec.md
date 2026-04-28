## MODIFIED Requirements

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
