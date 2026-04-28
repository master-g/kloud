## ADDED Requirements

### Requirement: Code blocks render with line number gutter
Fenced code blocks in markdown SHALL render with right-aligned line numbers as a gutter. The system SHALL NOT render `┌─ lang` / `└─` border markers.

#### Scenario: Code block with language tag
- **WHEN** a fenced code block with language "rust" and 3 lines is rendered
- **THEN** a dim "rust" label line appears first
- **AND** each code line is prefixed with right-aligned line number (`  1  `, `  2  `, `  3  `) in `theme.subtle` style
- **AND** syntax highlighting is applied to code content

#### Scenario: Code block without language
- **WHEN** a fenced code block with no language marker is rendered
- **THEN** each line has line number gutter with no language label line
- **AND** code content uses `theme.text` default style

#### Scenario: Code block line number width
- **WHEN** a code block has 100+ lines
- **THEN** line numbers use 3-digit width (`  1` to `100`)
- **AND** shorter line numbers are right-padded to align with gutter width

### Requirement: Code blocks do not use border markers
Code blocks SHALL NOT use `┌─`, `└─`, `╭`, `╰`, or any border characters as delimiters.

#### Scenario: No border markers in output
- **WHEN** any code block is rendered
- **THEN** no `┌─` top marker line appears
- **AND** no `└─` bottom marker line appears

### Requirement: Inline code unchanged
Inline code (single backtick) rendering SHALL remain unchanged.

#### Scenario: Inline code rendering
- **WHEN** inline code `variable` appears in markdown text
- **THEN** it renders with DIM modifier as backtick-wrapped text

### Requirement: Syntect highlighting preserved
Syntax highlighting via syntect SHALL continue to work with the new gutter layout.

#### Scenario: Rust code highlighting
- **WHEN** a rust code block is rendered
- **THEN** syntect highlights keywords, strings, and types
- **AND** highlighted spans appear after the gutter on each line

#### Scenario: Unknown language fallback
- **WHEN** a code block specifies an unsupported language
- **THEN** code content renders as plain text with line number gutter
- **AND** language label still appears as dim text
