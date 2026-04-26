## MODIFIED Requirements

### Requirement: Toast notification rendering
The system SHALL render toast notifications using `render_toasts()` wired to
`pending_toasts` from `SessionView`. Toasts SHALL appear above the input area.

#### Scenario: Session sends a toast
- **WHEN** `pending_toasts` in `SessionView` contains one or more entries
- **THEN** `render_toasts()` SHALL render them above the input area
- **AND** `#[allow(dead_code)]` SHALL be removed from `render_toasts()`

#### Scenario: No pending toasts
- **WHEN** `pending_toasts` is empty
- **THEN** no toast area SHALL be rendered (existing behavior unchanged)
