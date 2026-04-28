## ADDED Requirements

### Requirement: Markdown table rendering
The system SHALL render markdown tables as aligned ASCII tables with bordered cells.

#### Scenario: Simple table with headers
- **WHEN** `render_markdown()` encounters `Tag::Table` with headers and rows
- **THEN** the system SHALL render column headers separated by `│` with `─` row separator
- **AND** each row SHALL align cells to the maximum column width

#### Scenario: Table with alignment
- **WHEN** a table column has `Alignment::Center` / `Alignment::Right`
- **THEN** the system SHALL pad cell content accordingly (center / right-align)

#### Scenario: Wide table exceeds terminal width
- **WHEN** total table width exceeds available terminal width
- **THEN** the system SHALL truncate cells with ellipsis to fit

#### Scenario: Table with no rows
- **WHEN** a table has headers but zero rows
- **THEN** the system SHALL render only the header row with separator
