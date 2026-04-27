## MODIFIED Requirements

### Requirement: Permission prompt rendering
The system SHALL render permission prompts using `render_permission_prompt()` from
`widgets/permission.rs` instead of inline rendering in `messages.rs`. The inline
implementation at `messages.rs` lines ~87-104 SHALL be removed.

#### Scenario: Pending permission displayed
- **WHEN** `pending_permission` is set in `SessionView`
- **THEN** the system SHALL render the permission dialog using `render_permission_prompt()`
- **AND** the inline permission rendering in `messages.rs` SHALL be deleted
- **AND** `#[allow(dead_code)]` SHALL be removed from `render_permission_prompt()`

#### Scenario: No pending permission
- **WHEN** `pending_permission` is `None`
- **THEN** no permission widget SHALL be rendered (existing behavior unchanged)
