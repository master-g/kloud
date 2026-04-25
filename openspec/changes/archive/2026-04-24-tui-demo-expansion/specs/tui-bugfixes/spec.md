## ADDED Requirements

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
