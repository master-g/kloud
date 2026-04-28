## Requirements

### Requirement: Input area height adapts to content
The input area height SHALL be computed dynamically based on the number of lines in the TextArea. The minimum height SHALL be 3 lines. The maximum height SHALL be 50% of the terminal height.

#### Scenario: Single-line input
- **WHEN** TextArea contains 1 line of text
- **THEN** input area SHALL be 3 lines tall (1 content + 2 border/padding)

#### Scenario: Multi-line input expansion
- **WHEN** TextArea contains 5 lines of text
- **THEN** input area SHALL be 7 lines tall (5 content + 2 border/padding)
- **AND** message area SHALL shrink to accommodate

#### Scenario: Input hits maximum height
- **WHEN** TextArea contains 40 lines and terminal is 50 rows
- **THEN** input area SHALL be capped at 25 lines (50% of terminal)
- **AND** TextArea SHALL scroll internally

#### Scenario: Input shrinks back
- **WHEN** user deletes lines from multi-line input
- **THEN** input area height SHALL decrease accordingly
- **AND** message area SHALL expand to reclaim space

### Requirement: Dynamic layout reflow
The message area `Constraint` SHALL use `Min(1)` to absorb remaining space after other sections are allocated. The input area `Constraint` SHALL use `Length(computed_height)`.

#### Scenario: Terminal resize with multi-line input
- **WHEN** terminal is resized while TextArea has 4 lines
- **THEN** input area SHALL remain 6 lines tall
- **AND** message area SHALL reflow to fill remaining space
