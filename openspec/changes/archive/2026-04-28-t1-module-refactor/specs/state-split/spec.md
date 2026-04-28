## ADDED Requirements

### Requirement: TuiState decomposed into sub-structs
`TuiState` SHALL organize its fields into 5 sub-structs: `AppState`, `InputState`, `ScrollState`, `AnimState`, `ThemeState`. The `TuiState` struct SHALL remain as the single public facade containing these sub-structs.

#### Scenario: Field access through sub-structs
- **WHEN** render code accesses a message-related field
- **THEN** it SHALL use `state.app.messages` instead of `state.messages`
- **AND** the field SHALL have the same type and semantics as before the split

#### Scenario: All existing tests pass
- **WHEN** `cargo test` is run after the refactor
- **THEN** all existing tests SHALL pass without test code modification
- **AND** only the field access paths SHALL have changed

### Requirement: Animation types in dedicated module
`ActivityClock`, `StalledState`, `TokenCounter`, `ThinkingStatus` SHALL be defined in `state/anim_state.rs`. These types SHALL remain self-contained structs with their own fields and methods.

#### Scenario: tick() delegation unchanged
- **WHEN** `TuiState::tick()` is called
- **THEN** it SHALL delegate to `state.anim.activity_clock.try_tick()`, `state.anim.token_counter.advance()`, etc.
- **AND** behavior SHALL be identical to pre-refactor

### Requirement: Input state in dedicated module
`SearchState`, `AutocompleteState`, `TextArea`, `history`, `history_index` SHALL be fields of `InputState` in `state/input_state.rs`.

#### Scenario: Input operations accessible
- **WHEN** input handling code accesses the text area
- **THEN** it SHALL use `state.input.text_area` instead of `state.text_area`
- **AND** all input operations (vim mode, history nav, autocomplete) SHALL work identically

### Requirement: App state in dedicated module
`messages`, `screen`, `pending_permission`, `notifications`, `collapsed_tools`, `active_tools`, `tool_start_times`, `recent_activity`, `live_activity`, `activity_snapshot` SHALL be fields of `AppState` in `state/app_state.rs`.

#### Scenario: Message rendering access
- **WHEN** message rendering code reads messages
- **THEN** it SHALL use `state.app.messages`
- **AND** message display SHALL be identical to pre-refactor

### Requirement: Theme state in dedicated module
Theme switching state SHALL be in `state/theme_state.rs`.

#### Scenario: Theme toggle
- **WHEN** user toggles theme
- **THEN** `state.theme` SHALL manage the active theme
- **AND** all colors SHALL update identically to pre-refactor

### Requirement: Constants relocated
UI constants SHALL be moved from `src/ui/constants.rs` to `src/ui/tui/constants.rs`.

#### Scenario: Constants accessible
- **WHEN** TUI code references a constant (FPS, spinner verbs, timing thresholds)
- **THEN** it SHALL import from `crate::ui::tui::constants`
- **AND** values SHALL be identical to pre-refactor
