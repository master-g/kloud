## 1. Markdown Table Rendering

- [x] 1.1 Add `Tag::Table` / `Tag::TableHead` / `Tag::TableRow` / `Tag::TableCell` handling to `render_markdown()`
- [x] 1.2 Implement column width calculation and cell padding with alignment support
- [x] 1.3 Add `╭─╮` / `│` / `╰─╯` / `─` border rendering for table structure
- [x] 1.4 Add width truncation with ellipsis for cells exceeding terminal width
- [x] 1.5 Test with various table inputs (simple, aligned, wide, empty rows)

## 2. Syntax Highlighting

- [x] 2.1 Create `src/ui/tui/widgets/highlight.rs` module wrapping syntect initialization
- [x] 2.2 Implement syntect `HighlightLines` → ratatui `Style` color mapping
- [x] 2.3 Wire syntax highlighting into `Tag::CodeBlock(Fenced(lang))` branch in `render_markdown()`
- [x] 2.4 Handle fallback for unrecognized languages (plain rendering)
- [x] 2.5 Test with Rust, Python, JSON, and unknown language code blocks

## 3. Tool Block Borders

- [x] 3.1 Add bordered container rendering to `render_tool_use_line()` in `tool_block.rs`
- [x] 3.2 Implement `╭─ name ─╮` top border with tool name in purple bold and status dot
- [x] 3.3 Add `│` side borders to tool content lines
- [x] 3.4 Add `╰─╯` bottom border
- [x] 3.5 Add pink (bright magenta) border variant for bash tool (`theme.bash_border`)
- [x] 3.6 Update `render_tool_result_block()` to render inside bordered container
- [x] 3.7 Handle collapse state within bordered container (show/hide lines, keep border)
- [x] 3.8 Update `Theme` struct with `bash_border: Style` field across all 6 themes

## 4. Thinking Block Enhancement

- [x] 4.1 Add shimmer effect to thinking block label using `glimmer_spans()` + `compute_glimmer_index()` from `glimmer.rs`
- [x] 4.2 Condition shimmer on streaming state (shimmer during, static after)
- [x] 4.3 Implement thinking block collapse (threshold: 5 lines, same pattern as tool result)
- [x] 4.4 Add `collapsed_thinking: HashSet<usize>` to `AppState` for tracking (same pattern as `collapsed_tools`)
- [x] 4.5 Wire Tab key toggle for thinking blocks in `input.rs`

## 5. Diff Rendering Wiring

- [x] 5.1 Add diff detection utility function (check for `@@` hunk markers in output)
- [x] 5.2 Wire `render_diff()` into tool result rendering pipeline (call from widget or Tool trait)
- [x] 5.3 Test diff rendering with unified diff tool output

## 6. Message Timestamps

- [x] 6.1 Add `created_at: Option<Instant>` field to `DisplayMessage` in `state/mod.rs`
- [x] 6.2 Populate `created_at` during message normalization in `SessionStore`
- [x] 6.3 Add `show_timestamps: bool` to `Config` (default: false)
- [x] 6.4 Render relative timestamp (right-aligned, dim) on message first line when enabled
- [x] 6.5 Implement relative time formatting (`now` for <5s, `Xs/m/h ago`)

## 7. Integration & Polish

- [x] 7.1 Run `cargo fmt --all && cargo clippy -- -W warnings` clean
- [x] 7.2 Run `cargo test` all passing
- [ ] 7.3 Visual verification with `tui_showcase` example
