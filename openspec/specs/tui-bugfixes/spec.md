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

## Demo Scenario Requirements (from tui-demo-expansion)

### Requirement: DemoClient supports batch tools scenario
The `DemoClient.stream_plan()` SHALL recognize a "batch tools" keyword and return an assistant message containing 3 consecutive `ToolUse` content blocks targeting the `echo` tool.

#### Scenario: Batch tools demo triggers batch header
- **WHEN** user message contains "batch tools demo"
- **THEN** DemoClient returns 3 ToolUse blocks for "echo" tool
- **AND** Session dispatches all 3, producing "3 tools (3/3)" batch header in TUI

### Requirement: DemoClient supports MCP tool scenario
The `DemoClient.stream_plan()` SHALL recognize an "mcp demo" keyword and return a `ToolUse` block. The stream handler SHALL populate `server_names_by_id` with `Some("github-mcp")` for this tool's ID, resulting in `server_name: Some("github-mcp")` in the rendered `DisplayBlock::ToolUse`.

#### Scenario: MCP tool renders server name
- **WHEN** user message contains "mcp demo"
- **THEN** the tool use line renders with server name "github-mcp" prefix

### Requirement: DemoClient supports MaxTokens stop scenario
The `DemoClient.stream_plan()` SHALL recognize a "max tokens demo" keyword and return a text stream ending with `StopReason::MaxTokens`.

#### Scenario: MaxTokens shown in status bar
- **WHEN** user message contains "max tokens demo"
- **THEN** status bar displays "stop: max_tokens" after the stream completes

### Requirement: DemoClient supports Refusal stop scenario
The `DemoClient.stream_plan()` SHALL recognize a "refusal demo" keyword and return a text stream ending with `StopReason::Refusal`.

#### Scenario: Refusal shown in status bar
- **WHEN** user message contains "refusal demo"
- **THEN** status bar displays "stop: refusal" after the stream completes

### Requirement: DemoClient supports long text stress scenario
The `DemoClient.stream_plan()` SHALL recognize a "long text demo" keyword and return a very large text output (repeated paragraphs, total >100 lines) to exercise virtual scroll rendering.

#### Scenario: Long text triggers virtual scroll path
- **WHEN** user message contains "long text demo"
- **THEN** message count exceeds 100 and virtual scroll cache is populated during rendering

### Requirement: DemoClient supports context meter full scenario
The `DemoClient.stream_plan()` SHALL recognize a "context demo" keyword and return a stream with `input_tokens` set to a value >= 90% of `max_context_length`, triggering the red warning style on the context meter.

#### Scenario: Context meter turns red
- **WHEN** user message contains "context demo"
- **THEN** context meter fills to >=90% and renders with error style

### Requirement: Showcase script sends slash commands for system messages
The `run_showcase` coroutine SHALL send `UiAction::SlashCommand` variants to trigger system info and error messages without going through the LLM.

#### Scenario: Help command triggers info system message
- **WHEN** showcase sends `SlashCommand { command: "help", args: "" }`
- **THEN** a system message with level Info appears in the transcript

#### Scenario: Unknown command triggers error system message
- **WHEN** showcase sends `SlashCommand { command: "unknown_cmd", args: "" }`
- **THEN** a system message with level Error appears in the transcript

### Requirement: Showcase script sends CancelTurn during streaming
The `run_showcase` coroutine SHALL send `UiAction::CancelTurn` while a thinking stream is in progress, triggering the cancelling → idle transition.

#### Scenario: Cancel mid-thinking
- **WHEN** showcase sends "cancel demo" and waits 1.5 seconds then sends `CancelTurn`
- **THEN** the TUI shows "cancelling..." state then returns to idle with "[Request interrupted by user]" system message

### Requirement: Showcase script exercises transcript mode
The `run_showcase` coroutine SHALL send `UiAction::SetScreen(Screen::Transcript)` and later `SetScreen(Screen::Prompt)` to exercise transcript mode rendering.

#### Scenario: Transcript mode shows transcript footer
- **WHEN** showcase sends `SetScreen(Transcript)`
- **THEN** TUI renders transcript footer with "Showing detailed transcript" text
- **AND** when showcase sends `SetScreen(Prompt)`, TUI returns to normal prompt view

### Requirement: Showcase enables title bar
The showcase SHALL set `show_title_bar: true` on `RatatuiBackend` so the title bar widget is exercised during the demo.

#### Scenario: Title bar renders workspace info
- **WHEN** demo runs with `show_title_bar: true`
- **THEN** the first line of the terminal shows "Kloud" brand badge, model name, effort, branch, and workspace name
