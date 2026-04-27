## MODIFIED Requirements

### Requirement: No timing overlap between demos
The delay between consecutive demos in `run_showcase` SHALL be longer than the maximum duration of the preceding demo's stream (including tool execution follow-up streams). No `SendMessage` or `SlashCommand` SHALL arrive while the session is in `TurnState::Streaming`. The `latest_tool_result()` function SHALL only inspect the last user message to prevent stale tool results from earlier demos being matched.

#### Scenario: All stream demos complete before next prompt
- **WHEN** the autoplay sends a streaming demo prompt
- **THEN** the demo's full stream (including tool follow-up) completes
- **AND** the session transitions to `TurnState::Idle` before the next prompt arrives

#### Scenario: Stale tool results do not match subsequent demos
- **WHEN** the "read demo" completes and its tool result is in message history
- **AND** the next demo "error demo" is sent as a new user text message
- **THEN** `latest_tool_result()` SHALL return None because the last user message contains text, not a ToolResult
- **AND** the error demo's text-based match arm SHALL execute instead of the read demo's follow-up text
