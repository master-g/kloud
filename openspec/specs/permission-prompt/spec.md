## ADDED Requirements

### Requirement: Permission prompt rendered when pending
The TUI SHALL render an inline permission prompt widget below the last assistant message when `SessionView::pending_permission` is `Some`. The prompt SHALL display the label, description, and action buttons.

#### Scenario: Permission prompt appears
- **WHEN** `pending_permission` becomes `Some(PendingPermissionView { label: "Write file", description: "Allow writing to src/main.rs?" })`
- **THEN** an inline block appears below the last message showing "Write file" as title, "Allow writing to src/main.rs?" as description, and `[y] Allow / [n] Deny` buttons

### Requirement: Permission prompt accepts y/n key responses
The TUI SHALL capture `y` and `n` keys when a permission prompt is active and send the corresponding `UiAction` to the session.

#### Scenario: User allows permission
- **WHEN** permission prompt is visible and user presses `y`
- **THEN** TUI sends `UiAction::PermissionResponse { allowed: true }` and the prompt disappears

#### Scenario: User denies permission
- **WHEN** permission prompt is visible and user presses `n`
- **THEN** TUI sends `UiAction::PermissionResponse { allowed: false }` and the prompt disappears

### Requirement: Permission prompt shows diff preview for write operations
The permission prompt SHALL display a unified diff preview when the pending permission includes proposed file changes.

#### Scenario: File write with diff
- **WHEN** `pending_permission` includes a `diff` field containing unified diff content
- **THEN** the permission prompt renders the diff with `+` lines in green, `-` lines in red, and context lines in default color
