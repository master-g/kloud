## MODIFIED Requirements

### Requirement: Diff output demo stream
The showcase must include a demo where a tool result contains unified diff text, exercising the diff renderer with colored additions, deletions, and headers. The `DemoClient::stream_plan` match SHALL route "show diff demo" to the `diff_demo` method.

#### Scenario: Tool result shows colored diff
- **WHEN** the autoplay sends "show diff demo"
- **THEN** the assistant streams diff-colored text with green additions, red deletions, and blue headers
- **AND** the response ends with `StopReason::EndTurn`

### Requirement: Additional stop reason demos
The showcase must demonstrate StopReason::PauseTurn and StopReason::StopSequence. The `DemoClient::stream_plan` match SHALL route these prompts to their respective methods.

#### Scenario: Pause turn stop reason
- **WHEN** the autoplay sends "show pause turn demo"
- **THEN** the assistant responds with text ending in `StopReason::PauseTurn`
- **AND** the status bar shows "stop: pause_turn"

#### Scenario: Stop sequence stop reason
- **WHEN** the autoplay sends "show stop sequence demo"
- **THEN** the assistant responds with text ending in `StopReason::StopSequence`
- **AND** the status bar shows "stop: stop_sequence"

### Requirement: Search mode full cycle demo
The showcase must exercise the complete search lifecycle: activate, type query, navigate matches, and exit. The text area SHALL be empty before search activation so '/' triggers `SearchActivate`.

#### Scenario: Search activate, navigate, exit
- **WHEN** the autoplay injects '/' with empty text area
- **THEN** search mode activates with a search bar at top of messages area
- **WHEN** text is typed and submitted
- **THEN** matching messages are highlighted with match count display
- **WHEN** n/N keys are pressed
- **THEN** the current match navigates forward/backward
- **WHEN** Esc is pressed
- **THEN** search mode exits and normal prompt returns

## ADDED Requirements

### Requirement: No timing overlap between demos
The delay between consecutive demos in `run_showcase` SHALL be longer than the maximum duration of the preceding demo's stream (including tool execution follow-up streams). No `SendMessage` or `SlashCommand` SHALL arrive while the session is in `TurnState::Streaming`.

#### Scenario: All stream demos complete before next prompt
- **WHEN** the autoplay sends a streaming demo prompt
- **THEN** the demo's full stream (including tool follow-up) completes
- **AND** the session transitions to `TurnState::Idle` before the next prompt arrives

### Requirement: Text area cleanup after multi-line input
After the multi-line input demonstration, the showcase SHALL clear the text area completely so subsequent key injection phases (autocomplete, search) start with an empty input.

#### Scenario: Text area empty after multi-line demo
- **WHEN** the multi-line input phase completes
- **THEN** the text area content is empty
- **AND** the input mode is `Insert` (not Vim Normal)
- **AND** subsequent '/' key injection triggers `SearchActivate` or autocomplete as appropriate
