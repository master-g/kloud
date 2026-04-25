## ADDED Requirements

### Requirement: Permission prompt demo stream
The showcase must include a demo that triggers the permission prompt UI, showing the warning icon, label, description, and [y] Allow / [n] Deny buttons, then auto-responds.

#### Scenario: Permission prompt appears and is accepted
- **WHEN** the autoplay sends "show permission demo"
- **THEN** the assistant responds with text, the permission prompt appears in the message area
- **AND** after a delay, key injection sends 'y' to accept

#### Scenario: Permission prompt appears and is denied
- **WHEN** the autoplay sends "show permission deny demo"
- **THEN** the permission prompt appears
- **AND** after a delay, key injection sends 'n' to deny

### Requirement: Diff output demo stream
The showcase must include a demo where a tool result contains unified diff text, exercising the diff renderer with colored additions, deletions, and headers.

#### Scenario: Tool result shows colored diff
- **WHEN** the autoplay sends "show diff demo"
- **THEN** the assistant calls a write-like tool, the tool result contains a unified diff string
- **AND** the diff lines are colored: green additions, red deletions, blue headers

### Requirement: Notification toast demo
The showcase must trigger notification toasts that appear above the input area and auto-dismiss after a timeout.

#### Scenario: Info notification appears and dismisses
- **WHEN** the autoplay sends "show toast demo"
- **THEN** a notification toast appears above the input bar with info styling
- **AND** after the dismiss timeout, the toast disappears

### Requirement: Search mode full cycle demo
The showcase must exercise the complete search lifecycle: activate, type query, navigate matches, and exit.

#### Scenario: Search activate, navigate, exit
- **WHEN** the autoplay injects Ctrl+F
- **THEN** search mode activates with a search bar at top of messages area
- **WHEN** text is typed and submitted
- **THEN** matching messages are highlighted with match count display
- **WHEN** n/N keys are pressed
- **THEN** the current match navigates forward/backward
- **WHEN** Esc is pressed
- **THEN** search mode exits and normal prompt returns

### Requirement: Additional stop reason demos
The showcase must demonstrate StopReason::PauseTurn and StopReason::StopSequence.

#### Scenario: Pause turn stop reason
- **WHEN** the autoplay sends "show pause turn demo"
- **THEN** the assistant responds ending with stop_reason pause_turn
- **AND** the status bar shows "stop: pause_turn"

#### Scenario: Stop sequence stop reason
- **WHEN** the autoplay sends "show stop sequence demo"
- **THEN** the assistant responds ending with stop_reason stop_sequence
- **AND** the status bar shows "stop: stop_sequence"
