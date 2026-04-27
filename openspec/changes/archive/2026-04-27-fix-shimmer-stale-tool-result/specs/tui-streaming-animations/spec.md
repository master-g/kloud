## MODIFIED Requirements

### Requirement: Stall detection and visual indicator
The system SHALL detect streaming stalls (no token received for 3+ seconds within the current query). The stall detector SHALL reset when a new query begins (status transitions to Streaming). When stalled, the activity indicator SHALL change to a distinct visual state (e.g., blinking or color change).

#### Scenario: Streaming stalls for 3 seconds
- **WHEN** no ContentBlockDelta event arrives for 3000ms during active streaming
- **THEN** the activity line SHALL show a stalled indicator (blinking amber)
- **WHEN** a new delta arrives
- **THEN** the stalled indicator SHALL immediately clear

#### Scenario: Stall detector resets on new query
- **WHEN** a streaming query completes and a new query begins
- **THEN** the stall detector SHALL reset its last_response_length to 0 and last_token_at to the current instant
- **AND** the stall detector SHALL NOT trigger false stall detection during the new query's initial streaming phase

#### Scenario: Previous query's token count does not cause false stall
- **WHEN** query N produces 500 characters and completes
- **AND** query N+1 starts with response_char_count reset to 0
- **THEN** the stall detector SHALL treat 0 as a fresh baseline, not compare against the previous query's 500
- **AND** the shimmer SHALL render normally without red color interpolation for at least 3 seconds
