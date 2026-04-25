## ADDED Requirements

### Requirement: UiChannels supports optional key injection receiver
`UiChannels` SHALL include an `key_inject_rx: Option<mpsc::Receiver<crossterm::Event>>` field. When `Some`, the TUI event loop SHALL poll this channel alongside the real crossterm event stream. When `None`, the poll SHALL be a no-op (pending future). Production code creates channels with `key_inject_rx: None`.

#### Scenario: Production channels have no injection
- **WHEN** `create_ui_channels()` is called
- **THEN** `key_inject_rx` is `None` and the TUI event loop ignores the injection branch

#### Scenario: Demo channels accept injected events
- **WHEN** demo code creates channels with `key_inject_rx: Some(rx)`
- **AND** demo sends a `crossterm::Event::Key(KeyEvent { code: Char('/'), .. })` via `key_inject_tx`
- **THEN** the TUI event loop processes it identically to a real keyboard event

### Requirement: Injected events are processed through handle_event
Every event received from `key_inject_rx` SHALL be passed to `input::handle_event()` with the same semantics as real crossterm events. The resulting `UiAction`, if any, SHALL be forwarded to `channels.action_tx`.

#### Scenario: Injected search activation
- **WHEN** demo injects `KeyCode::Char('/')` while input is empty and status is Idle
- **THEN** TUI transitions to `Screen::Search` and sends `UiAction::SearchActivate`

#### Scenario: Injected search query and navigation
- **WHEN** demo injects sequence: `'t'`, `'e'`, `'x'`, `'t'`, `Enter`, `'n'`, `Esc`
- **THEN** search activates, query builds to "text", results display, next match navigates, search exits

#### Scenario: Injected multi-line input
- **WHEN** demo injects `Char('a')`, `Enter` with `Shift` modifier, `Char('b')`, `Enter` with no modifier
- **THEN** text area contains "a\nb" and `SendMessage("a\nb")` is sent

### Requirement: Demo coroutine holds the injection sender
`run_showcase` (or equivalent demo driver) SHALL hold `key_inject_tx: mpsc::Sender<crossterm::Event>` and use it to synthesize keyboard event sequences at scripted delays.

#### Scenario: Search demo phase
- **WHEN** demo script reaches the search phase
- **THEN** it injects `'/'` key, types "think", injects `Enter`, injects `'n'`, waits, injects `Esc`
- **AND** the TUI renders search mode, results, and exit back to prompt

#### Scenario: Vim mode demo phase
- **WHEN** demo script reaches the vim phase
- **THEN** it injects text chars, injects `Esc` (enter normal mode), injects `'k'`/'`j'` (move), injects `'i'` (back to insert), injects `Enter`
- **AND** the TUI shows vim mode transitions in the input area
