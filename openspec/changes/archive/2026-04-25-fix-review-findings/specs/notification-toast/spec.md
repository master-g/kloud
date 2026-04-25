## MODIFIED Requirements

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
