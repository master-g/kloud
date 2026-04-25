## ADDED Requirements

### Requirement: Theme registry with 6 built-in themes
The system SHALL provide 6 predefined themes: `dark`, `light`, `dark-ansi`, `light-ansi`, `dark-daltonized`, `light-daltonized`. Each theme SHALL define a complete set of semantic colors (text, brand, success, error, warning, subtle, inactive, tool, suggestion, prompt_border, etc.).

#### Scenario: Default theme loaded on startup
- **WHEN** the TUI starts without user configuration
- **THEN** the `dark` theme SHALL be loaded as default

#### Scenario: User switches theme at runtime
- **WHEN** user issues the `/theme <name>` command
- **THEN** the TUI SHALL immediately re-render with the new theme colors
- **THEN** all widgets SHALL reflect the new theme without restart

### Requirement: Semantic color abstraction
All widgets SHALL reference colors through semantic names (e.g., `theme.colors.text`, `theme.colors.success`), NOT through raw color values. The `ThemeColors` struct SHALL contain at minimum 20 semantic color fields matching Claude Code's theme.ts definitions.

#### Scenario: Widget uses semantic color
- **WHEN** a widget renders text
- **THEN** it SHALL use `theme.colors.text` not a hardcoded `Color::White`

### Requirement: ANSI fallback for 256-color terminals
The `dark-ansi` and `light-ansi` themes SHALL use only ANSI 256-color palette values. When the terminal reports < 16 million color support, the system SHALL auto-select the ANSI variant.

#### Scenario: 256-color terminal detected
- **WHEN** terminal COLORTERM is not set to "truecolor" or "24bit"
- **THEN** the system SHALL use `dark-ansi` instead of `dark` as default

### Requirement: Brand color consistency
All themes SHALL define `claude` brand color as the primary accent. The RGB value for the default dark theme SHALL be `rgb(215,119,87)` matching Claude Code exactly.

#### Scenario: Brand color used for assistant prefix
- **WHEN** an assistant message is rendered with the `●` prefix
- **THEN** the prefix SHALL use `theme.colors.claude` color
