## ADDED Requirements

### Requirement: Virtual scroll cache initialized before range computation
The system SHALL call `ensure_cache` on the virtual scroll state before computing `visible_message_range` when rendering messages.

#### Scenario: Messages exceed virtual scroll threshold
- **WHEN** more than 100 messages are displayed
- **THEN** the virtual scroll cache is populated with line heights and the visible range returns correct (start, offset, end) indices

### Requirement: Message index accounts for virtual scroll offset
The system SHALL add the virtual scroll start index to the enumeration index when looking up collapse state for tool blocks.

#### Scenario: Virtual scroll active with collapsed tool
- **WHEN** virtual scroll filters messages to a slice starting at index 50
- **THEN** tool collapse state is looked up using index 50 + local enumerate index, not the local index alone

### Requirement: Batch tool header printed once per message
The system SHALL print the batch tool summary header at most once per message, regardless of how many ToolUse blocks the message contains.

#### Scenario: Message with 5 consecutive ToolUse blocks
- **WHEN** a message contains 5 ToolUse blocks
- **THEN** the "N tools (M/N)" header appears exactly once, not 5 times

### Requirement: Tab toggle auto-populates first collapsible tool
The system SHALL find and toggle the nearest tool result block exceeding the collapse threshold when Tab is pressed and no tools are currently tracked in the collapse map.

#### Scenario: First Tab press with no prior collapse state
- **WHEN** the collapse map is empty and the user presses Tab
- **THEN** the system scans for the nearest tool result with >5 lines and inserts it as collapsed

#### Scenario: No collapsible tools exist
- **WHEN** no tool results exceed 5 lines and the user presses Tab
- **THEN** the press is a no-op

### Requirement: Search screen preserved across view syncs
The system SHALL NOT overwrite `Screen::Search` with the incoming view's screen value during `sync_from_active_view`.

#### Scenario: Active search receives stream delta
- **WHEN** the user is in Search screen and a stream delta arrives
- **THEN** the screen remains `Screen::Search`

### Requirement: PageDown only enables auto-scroll at bottom
The system SHALL only re-enable auto-scroll when PageDown reaches the actual bottom of the content, not unconditionally.

#### Scenario: PageDown mid-content
- **WHEN** user presses PageDown while scrolled to the middle of content
- **THEN** auto-scroll remains disabled and the view scrolls down by half a page

#### Scenario: PageDown at bottom
- **WHEN** user presses PageDown and the scroll position reaches the bottom
- **THEN** auto-scroll re-enables

### Requirement: Daltonized themes distinguish warning from success by luminance
The system SHALL ensure warning and success colors in daltonized themes have a luminance delta of at least 30.

#### Scenario: Color-blind user views tool results
- **WHEN** a daltonized theme is active and both warning and success tool results are visible
- **THEN** the two colors are distinguishable by brightness, not just hue

### Requirement: SearchState implements Default
The system SHALL derive `Default` for `SearchState`.

#### Scenario: Clippy check
- **WHEN** `cargo clippy` runs
- **THEN** no warning about missing `Default` impl on `SearchState`

### Requirement: scroll_down uses actual visible line count
The system SHALL pass the visible line count to `scroll_down` instead of hardcoding zero, so auto-scroll can correctly re-enable.

#### Scenario: Scroll down at bottom of content
- **WHEN** user scrolls down and reaches the bottom
- **THEN** auto-scroll re-enables correctly
