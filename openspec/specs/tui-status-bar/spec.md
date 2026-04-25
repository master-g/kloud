## ADDED Requirements

### Requirement: Status bar displays session cost
The status bar SHALL display the current session cost in USD, formatted as `$X.XX` (2 decimal places). The cost section SHALL appear after the model name.

#### Scenario: Cost displayed after tokens
- **WHEN** session cost is $0.15
- **THEN** status bar includes `$0.15` in the cost section

### Requirement: Status bar displays session duration
The status bar SHALL display session elapsed time formatted as `Xm Xs` (or `Xs` for under a minute). The duration section SHALL appear after the cost.

#### Scenario: Duration over one minute
- **WHEN** session has been running for 95 seconds
- **THEN** status bar shows `1m 35s`

#### Scenario: Duration under one minute
- **WHEN** session has been running for 42 seconds
- **THEN** status bar shows `42s`

### Requirement: Status bar sections truncate on narrow terminals
When the terminal width is less than 80 columns, the status bar SHALL hide cost and duration sections, showing only model name and token counts.

#### Scenario: Narrow terminal
- **WHEN** terminal width is 60 columns
- **THEN** status bar shows only model name and context meter, hiding cost and duration
