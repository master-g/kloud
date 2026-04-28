## Why

T2 is the TUI roadmap's "Visual Core" phase. The message area is the primary surface users
interact with, yet several visual gaps remain: no markdown tables, no syntax highlighting
(syntect is a dependency but unused), no tool block borders, thinking blocks are plain text
without shimmer/collapse, and diff rendering exists but is unwired. Tool pre-rendering was
just completed (Tool trait `render_*` methods), so all prerequisites are met.

## What Changes

- Add markdown table rendering to `render_markdown()` (pulldown-cmark `Tag::Table` branch)
- Wire syntect into code block rendering for syntax highlighting
- Add bordered container for tool blocks (rounded corners, tool name in purple, status icons)
- Add pink border variant for bash tool output
- Enhance thinking blocks: shimmer animation + collapsible (expand/collapse like tool results)
- Wire existing `diff.rs` into tool result rendering pipeline
- Add timestamps to `DisplayMessage` (optional display, controlled by config)

## Capabilities

### New Capabilities
- `markdown-tables`: Table rendering in pulldown-cmark to ratatui Lines
- `syntax-highlighting`: Syntect-based code block syntax coloring
- `tool-block-borders`: Bordered container with rounded corners for tool use/result blocks
- `thinking-enhance`: Shimmer plus collapse for thinking blocks
- `message-timestamps`: Timestamp display on messages

### Modified Capabilities
- `tui-tool-rendering`: Tool pre-rendering now outputs bordered content, not plain lines
- `diff-renderer`: Wire render_diff into tool result pipeline (spec says detect and render)

## Impact

- **`src/ui/tui/widgets/messages.rs`** -- markdown table branch, thinking enhancement, timestamp rendering
- **`src/ui/tui/widgets/tool_block.rs`** -- bordered container, bash pink variant
- **`src/ui/tui/widgets/diff.rs`** -- wire into tool result rendering
- **`src/tools/traits.rs`** -- render_tool_result_message may need to output bordered structure
- **`src/ui/tui/state/mod.rs`** -- DisplayMessage gains optional timestamp field
- **Dependencies**: syntect already in Cargo.toml, no new deps needed
