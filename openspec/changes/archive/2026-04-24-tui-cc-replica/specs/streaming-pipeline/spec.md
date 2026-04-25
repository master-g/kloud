## MODIFIED Requirements

### Requirement: Stream events carry UI rendering metadata
StreamEvent handling SHALL emit additional metadata for TUI animations: per-delta token count increment, stall detection signal, and thinking duration.

#### Scenario: Token count delta in stream event
- **WHEN** a ContentBlockDelta(TextDelta) event is processed
- **THEN** the emitted SessionEvent SHALL include a cumulative token count for the current turn
- **THEN** the TUI SHALL use this count for animated token display

#### Scenario: Stall detection signal
- **WHEN** no ContentBlockDelta arrives for 3000ms during streaming
- **THEN** the Session SHALL emit a `StreamStalled` event
- **WHEN** a new delta arrives after stall
- **THEN** the Session SHALL emit a `StreamResumed` event

#### Scenario: Thinking duration tracking
- **WHEN** a ContentBlockStart(Thinking) event is received
- **THEN** the Session SHALL record the start timestamp
- **WHEN** the thinking block completes (ContentBlockStop)
- **THEN** the SessionEvent SHALL include the thinking duration in milliseconds
