## MODIFIED Requirements

### Requirement: Keybinding hints display
The system SHALL render context-sensitive keybinding hints using `render_hints()` in
the status bar area. Hints SHALL be width-gated — hidden when terminal width is
insufficient.

#### Scenario: Terminal is wide enough for hints
- **WHEN** terminal width exceeds the threshold for hint display
- **THEN** `render_hints()` SHALL show relevant keybindings based on current state
- **AND** `#[allow(dead_code)]` SHALL be removed from `render_hints()`

#### Scenario: Terminal is too narrow for hints
- **WHEN** terminal width is below the threshold
- **THEN** hints SHALL not be rendered to avoid layout overflow
