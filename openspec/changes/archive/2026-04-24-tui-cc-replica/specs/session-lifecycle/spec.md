## MODIFIED Requirements

### Requirement: SessionView supports TUI mode state
SessionView SHALL include the current UI mode (Normal, Search, Transcript) and mode-specific data (search query, match positions, transcript scroll offset).

#### Scenario: Switch to search mode
- **WHEN** user activates search mode
- **THEN** the SessionView SHALL include `screen: Screen::Search` with an empty query
- **THEN** subsequent keystrokes SHALL update the query in the view

#### Scenario: Switch to transcript mode
- **WHEN** user presses `Ctrl+O`
- **THEN** the SessionView SHALL include `screen: Screen::Transcript` with full message history
- **WHEN** user presses `Ctrl+O` again
- **THEN** the SessionView SHALL revert to `screen: Screen::Prompt`

### Requirement: AppEvent and UiAction extended for new UI features
AppEvent SHALL include new variants: `ThemeChanged(String)`, `SearchQuery(String)`, `SearchNavigateNext`, `SearchNavigatePrev`, `ToggleToolCollapse(usize)`. UiAction SHALL include corresponding user action variants.

#### Scenario: Theme change event
- **WHEN** user issues `/theme dark`
- **THEN** a `UiAction::ChangeTheme("dark")` SHALL be sent to Session
- **THEN** Session SHALL emit `AppEvent::ThemeChanged("dark")` back to UI
- **THEN** the UI SHALL re-render with new theme colors

#### Scenario: Tool collapse toggle
- **WHEN** user presses `Tab` on a focused tool block
- **THEN** a `UiAction::ToggleToolCollapse(block_index)` SHALL be sent
- **THEN** the SessionView SHALL update the collapse state for that block
