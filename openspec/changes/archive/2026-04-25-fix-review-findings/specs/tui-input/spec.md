## MODIFIED Requirements

### Requirement: Permission prompt allows emergency exit
The permission prompt key handler must allow Ctrl+C to exit the application even when the prompt is shown.

#### Scenario: User presses Ctrl+C during permission prompt
- **WHEN** `pending_permission` is Some and `status` is Idle
- **AND** the user presses Ctrl+C
- **THEN** `UiAction::Exit` is returned instead of being blocked
- **AND** the application quits normally
