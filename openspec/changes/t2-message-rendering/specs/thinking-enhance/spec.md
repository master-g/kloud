## ADDED Requirements

### Requirement: Thinking block shimmer effect
The system SHALL apply shimmer animation to the thinking block label during streaming.

#### Scenario: Streaming thinking block
- **WHEN** an assistant message contains a `Thinking` block and the assistant is still streaming
- **THEN** the `∴ Thinking` label SHALL use the glimmer shimmer effect from `glimmer.rs`
- **AND** thinking content lines SHALL render with dim style

#### Scenario: Completed thinking block
- **WHEN** the assistant has finished streaming
- **THEN** the shimmer SHALL stop and the label SHALL render with static dim italic style

### Requirement: Thinking block collapse
Thinking blocks with more than 5 lines of content SHALL be collapsible.

#### Scenario: Long thinking block collapsed
- **WHEN** a thinking block has more than 5 lines and is collapsed
- **THEN** the system SHALL show only the first 3 lines
- **AND** display "... N more lines (Tab to expand)" hint

#### Scenario: Expanded thinking block
- **WHEN** a thinking block is expanded
- **THEN** all content lines SHALL be visible
- **AND** display "(Tab to collapse)" hint at the bottom
