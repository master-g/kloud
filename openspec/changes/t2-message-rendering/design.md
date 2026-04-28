## Context

TUI message area renders assistant/user messages, tool blocks, thinking blocks, and markdown
content. Current `render_markdown()` uses pulldown-cmark but skips tables. `syntect` is a
dependency but zero code uses it. Tool blocks have status dots but no border containers.
Thinking blocks show plain text. `diff.rs` exists but is dead code.

Tool pre-rendering (Tool trait `render_tool_use_message`/`render_tool_result_message`) was
just completed. Tools emit `Vec<Line<'static>>` at the store layer. The TUI widget layer
receives pre-rendered content and only adds layout (status dot, prefix, collapse).

## Goals / Non-Goals

**Goals:**
- Markdown tables rendered as aligned ASCII tables
- Syntax-highlighted code blocks (syntect)
- Tool blocks wrapped in bordered containers with rounded corners
- Bash tool blocks distinguished by pink border
- Thinking blocks with shimmer effect and collapse
- Diff rendering wired into tool result pipeline

**Non-Goals:**
- Tree-sitter (syntect is sufficient for TUI)
- Inline editing or interactive table navigation
- Configurable border styles (use CC-matching style)
- Custom color themes per language (use syntect defaults mapped to theme)

## Decisions

### D1: Table rendering strategy

**Decision**: Manual column alignment in `render_markdown()` table branch.

Pulldown-cmark emits `Tag::Table(alignment)` → `Tag::TableHead` → `Tag::TableRow` → `Tag::TableCell`.
We collect cell widths per column, pad to max width, and render with `│` / `─` / `┌┐└┘` borders.

**Alternative considered**: `comfy-table` or `tabled` crate. Rejected: adds dependency for
something achievable in ~50 lines with pulldown-cmark events.

### D2: Syntax highlighting integration

**Decision**: Syntect with `HighlightLines` per code block, mapped to ratatui `Style`.

On `Tag::CodeBlock(Fenced(lang))`, detect language, create `HighlightLines`, apply styles
to each text event. Map syntect `Color` → `ratatui::Color::Rgb`. Fallback to plain rendering
if language not recognized.

**Alternative considered**: `bat` library. Rejected: syntect already in Cargo.toml, bat pulls
more deps.

### D3: Tool block border rendering location

**Decision**: Borders rendered at TUI widget layer (`tool_block.rs`), NOT at Tool trait level.

Tool trait `render_tool_use_message()` returns content lines. The widget wraps those lines
in a bordered `Paragraph`. This separates content (tool-specific) from presentation
(bordered container).

**Alternative considered**: Tool trait returns bordered content. Rejected: mixes content and
layout concerns, makes testing harder.

### D4: Thinking block shimmer

**Decision**: Reuse existing `glimmer_spans()` + `compute_glimmer_index()` from `glimmer.rs`.

Thinking blocks get a shimmer effect on the `∴ Thinking` label during streaming, using
`glimmer_spans()` for the 3-segment split (before|shimmer|after). After streaming
completes, thinking content becomes collapsible (same pattern as tool result collapse:
threshold + "Tab to expand").

### D5: Diff wiring

**Decision**: Detect diff content in `Tool::render_tool_result_message()` and route to
`render_diff()` from `diff.rs`.

Each tool's `render_tool_result_message()` can call `render_diff()` when output contains
unified diff markers. Alternatively, a shared utility function detects and routes.

### D6: Timestamps

**Decision**: Add `created_at: Option<Instant>` to `DisplayMessage`. Render as relative time
(`2m ago`) in dim style, right-aligned on the message header line. Only shown when
`show_timestamps` config is enabled (default: off).

## Risks / Trade-offs

- **Syntect theme mismatch**: syntect uses its own color scheme. Need a mapping layer from
  syntect theme to our `Theme` semantic colors. Risk: colors look inconsistent.
  Mitigation: start with syntect's `base16-eighties.dark` which is close to CC's dark theme.

- **Table rendering width**: Wide tables may exceed terminal width. Mitigation: truncate
  cells to available width with `…` ellipsis, same as CC behavior.

- **diff.rs wiring**: `diff.rs` contains clean, tested functions (`looks_like_diff`,
  `render_diff`) that are simply never called yet. No dead_code markers to remove —
  wiring them into the tool result pipeline is straightforward.

- **Performance**: syntect highlighting + table layout + diff detection per render could
  be slow on large messages. Mitigation: content is pre-rendered once at store layer;
  widget layer only adds layout (borders, prefixes). Pre-rendered content is cached in
  `DisplayBlock` fields.
