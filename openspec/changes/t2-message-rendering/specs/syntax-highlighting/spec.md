## ADDED Requirements

### Requirement: Code block syntax highlighting
The system SHALL apply syntax highlighting to fenced code blocks using syntect.

#### Scenario: Fenced code block with recognized language
- **WHEN** `render_markdown()` encounters a fenced code block with a recognized language (e.g. `rust`, `python`)
- **THEN** the system SHALL apply syntect syntax highlighting to code content
- **AND** syntect colors SHALL be mapped to ratatui `Color::Rgb` styles

#### Scenario: Fenced code block with unrecognized language
- **WHEN** the language is not recognized by syntect
- **THEN** the system SHALL render the code block with plain monospace styling (existing behavior)

#### Scenario: Indented code block
- **WHEN** an indented code block is encountered (no language tag)
- **THEN** the system SHALL render without syntax highlighting (existing behavior)

#### Scenario: Inline code
- **WHEN** inline code (backtick) is encountered
- **THEN** the system SHALL render with dim style (existing behavior, unchanged)
