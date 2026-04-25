## ADDED Requirements

### Requirement: Shimmer effect for streaming activity
During LLM response streaming, the activity line SHALL display a shimmer animation (left-to-right gradient sweep). The shimmer SHALL use the brand color with alpha fade.

#### Scenario: Text streaming shimmer
- **WHEN** the assistant is actively streaming text content
- **THEN** a left-to-right shimmer gradient SHALL animate across the activity line
- **THEN** the shimmer SHALL reset and repeat continuously

#### Scenario: Thinking shimmer
- **WHEN** the model is in a thinking state
- **THEN** a breathing effect (slow opacity pulse) SHALL display on the thinking indicator
- **THEN** after 3 seconds of thinking, a rainbow gradient SHALL overlay the thinking block

### Requirement: Stall detection and visual indicator
The system SHALL detect streaming stalls (no token received for 3+ seconds). When stalled, the activity indicator SHALL change to a distinct visual state (e.g., blinking or color change).

#### Scenario: Streaming stalls for 3 seconds
- **WHEN** no ContentBlockDelta event arrives for 3000ms during active streaming
- **THEN** the activity line SHALL show a stalled indicator (blinking amber)
- **WHEN** a new delta arrives
- **THEN** the stalled indicator SHALL immediately clear

### Requirement: Token count animation
The displayed token count SHALL animate smoothly toward the target value rather than jumping instantly. Animation SHALL advance ~10% per tick toward target.

#### Scenario: Token count updates
- **WHEN** usage reports 1000 tokens used
- **THEN** displayed count SHALL animate from current displayed value to 1000 over several frames
- **THEN** the animation SHALL not overshoot the target

### Requirement: Activity fade-out after turn completion
When a streaming turn completes, the activity line SHALL fade out over 1 second (20 frames) before disappearing.

#### Scenario: Turn ends
- **WHEN** streaming ends (stop_reason received)
- **THEN** the activity line SHALL smoothly fade from full opacity to invisible
- **THEN** the fade SHALL take approximately 1 second

### Requirement: Tool execution progress animation
During tool execution, the tool indicator SHALL blink (alternate between filled and hollow circle every 500ms).

#### Scenario: Tool running
- **WHEN** a tool is actively executing
- **THEN** its status indicator SHALL blink between `●` (filled) and `○` (hollow) every 500ms
- **WHEN** the tool completes
- **THEN** the indicator SHALL show a steady `●` (success) or `✗` (error)
