## ADDED Requirements

### Requirement: Contextual keybinding hints bar
The TUI SHALL render a one-line hints bar below the input area showing context-sensitive keybinding suggestions. Hints SHALL update based on current screen, streaming state, and input mode.

#### Scenario: Idle state hints
- **WHEN** status is Idle and input is empty in insert mode
- **THEN** hints bar shows "Enter: Send | Shift+Enter: New line | /: Command | Esc: Vim mode | Ctrl+O: Transcript"

#### Scenario: Streaming state hints
- **WHEN** status is Streaming
- **THEN** hints bar shows "Ctrl+C: Cancel"

#### Scenario: Search mode hints
- **WHEN** screen is Search
- **THEN** hints bar shows "Enter: Search | n/N: Next/Prev | Esc: Exit"

#### Scenario: Vim normal mode hints
- **WHEN** input mode is Normal
- **THEN** hints bar shows "i: Insert | h/j/k/l: Move | x: Delete | dd: Delete line"
