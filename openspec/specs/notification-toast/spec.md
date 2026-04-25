## ADDED Requirements

### Requirement: Transient notification toasts
The TUI SHALL render transient notification messages as styled inline toasts that auto-dismiss after a configurable duration. Toasts SHALL appear below the status bar or above the input area.

#### Scenario: Theme change notification
- **WHEN** session sends a `SystemMessageAdded` with level `Info` and content "Theme changed to dark"
- **THEN** a styled toast appears with the message text and auto-dismisses after 3 seconds

#### Scenario: Error notification
- **WHEN** session sends a `SystemMessageAdded` with level `Error`
- **THEN** a red-styled toast appears and auto-dismisses after 5 seconds

### Requirement: Maximum visible toasts limited
The TUI SHALL display at most 2 visible toasts simultaneously. Older toasts auto-dismiss faster when the limit is reached.

#### Scenario: Three notifications queued
- **WHEN** three notifications arrive within 1 second
- **THEN** the first two are displayed and the third is queued, appearing when the first dismisses

### Requirement: Toast trigger does not leak prefix into transcript
When a notification toast is created, the `[toast]` prefix must not appear in the visible system message in the transcript.

#### Scenario: Notify command creates toast without prefix leak
- **WHEN** `/notify` command is issued
- **THEN** a notification toast appears above the input bar
- **AND** the system message in the transcript shows clean text without `[toast]` prefix

### Requirement: Toast detection survives message list shrinkage
The notification detection logic must not break when the message list shrinks due to view normalization.

#### Scenario: View normalization reduces message count
- **WHEN** `sync_from_active_view` replaces messages and the new list is shorter
- **THEN** toast detection continues to work for subsequent new messages
- **AND** `last_seen_message_count` is clamped to the new length
