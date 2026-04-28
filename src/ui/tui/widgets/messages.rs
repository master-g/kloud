//! Message rendering: conversation messages and markdown formatting.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::activity_line::render_live_assistant_header;
use super::layout::{truncate_path, truncate_to_width, workspace_name};
use super::tool_block::{render_tool_result_block, render_tool_use_line};
use crate::ui::tui::constants::{DASHBOARD_LOGO, TOOL_CIRCLE, VERSION};
use crate::ui::tui::state::{
    AssistantStatus, DisplayBlock, MessageLevel, MessageType, ToolStatus, TuiState,
};
use crate::ui::tui::theme::Theme;

/// Render the scrollable messages area with a logo header at the top.
pub(super) fn render_messages(frame: &mut Frame, state: &mut TuiState, theme: &Theme, area: Rect) {
    let mut lines: Vec<Line<'_>> = Vec::new();

    // Populate virtual scroll cache before any immutable borrows of state.
    let msg_count = state.app.messages.len();
    let show_range = if msg_count > 100 {
        state.scroll.virtual_scroll.ensure_cache(&state.app.messages, area.width);
        let (s, _, e) = state.scroll.virtual_scroll.visible_message_range(area.height, 0);
        Some((s, e))
    } else {
        None
    };

    // Search bar at top of messages area
    if state.app.screen == crate::ui::tui::state::Screen::Search {
        let match_info = state.input.search.match_display();
        let wrapped = if state.input.search.wrapped {
            " (wrapped)"
        } else {
            ""
        };
        lines.push(Line::from(vec![
            Span::styled(" /", theme.claude_bold),
            Span::styled(state.input.search.query.clone(), theme.text),
            Span::styled(format!(" {match_info}{wrapped}"), theme.suggestion),
        ]));
    }

    if state.app.screen != crate::ui::tui::state::Screen::Transcript {
        render_logo_header(state, theme, area.width, &mut lines);
    }

    let messages: Vec<&crate::ui::tui::state::DisplayMessage> =
        if let Some((start, end)) = show_range {
            state.app.messages.iter().skip(start).take(end.saturating_sub(start) + 1).collect()
        } else {
            state.app.messages.iter().collect()
        };

    let msg_offset = show_range.map(|(s, _)| s).unwrap_or(0);
    let show_timestamps = state.app.show_timestamps;
    for (local_idx, msg) in messages.iter().enumerate() {
        let msg_idx = local_idx + msg_offset;
        match &msg.message_type {
            MessageType::User => {
                render_user_message(msg, theme, show_timestamps, area.width, &mut lines);
            }
            MessageType::Assistant => {
                render_assistant_message(msg, msg_idx, state, theme, area.width, &mut lines);
            }
            MessageType::System {
                level,
            } => {
                render_system_message(msg, level, theme, &mut lines);
            }
            MessageType::Progress {
                ..
            } => {
                render_progress_message(msg, theme, &mut lines);
            }
        }
        lines.push(Line::from(""));
    }

    // Keep activity line inside transcript flow so it scrolls with messages.
    if state.displayed_activity().is_some() || state.status == AssistantStatus::Cancelling {
        lines.push(render_live_assistant_header(state, theme, area.width));
    }

    // Tool spinners (inline in message flow, after activity header)
    if let Some(spinner_line) = super::activity_line::render_tool_spinners(state, theme) {
        lines.push(spinner_line);
    }

    // Permission prompt (render as last element when pending)
    if let Some(perm) = &state.app.pending_permission {
        lines.extend(super::permission::render_permission_prompt(theme, perm));
    }

    let content_height = lines.len() as u16;
    let visible_height = area.height;
    let max_scroll = content_height.saturating_sub(visible_height);

    let is_streaming = state.app.screen != crate::ui::tui::state::Screen::Transcript
        && matches!(state.status, AssistantStatus::Streaming | AssistantStatus::Cancelling);

    let should_auto_scroll = !state.scroll.auto_scroll_paused;
    let jump_flag = state.scroll.should_jump_to_bottom();
    let use_max = jump_flag || should_auto_scroll;
    let scroll = if use_max {
        max_scroll
    } else {
        state.scroll.offset_u16().min(max_scroll)
    };

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap {
            trim: false,
        })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);

    if jump_flag {
        state.scroll.offset = max_scroll as usize;
        state.scroll.clear_jump_flag();
    } else if should_auto_scroll {
        state.scroll.offset = max_scroll as usize;
    }

    // Auto-scroll paused indicator
    if state.scroll.auto_scroll_paused && is_streaming && visible_height > 2 {
        let indicator_y = area.y + visible_height - 2;
        let label = " ↓ auto-scroll paused ";
        let label_width = label.len() as u16;
        let indicator = Paragraph::new(Line::from(vec![Span::styled(label, theme.warning)]));
        frame.render_widget(
            indicator,
            Rect {
                x: area.x + (area.width.saturating_sub(label_width)) / 2,
                y: indicator_y,
                width: label_width.min(area.width),
                height: 1,
            },
        );
    }
}

// ============================================================================
// Logo header (rendered at top of the scroll area, like CC's CondensedLogo)
// ============================================================================

fn render_logo_header<'a>(
    state: &'a TuiState,
    theme: &'a Theme,
    width: u16,
    lines: &mut Vec<Line<'a>>,
) {
    let ws_name = workspace_name(&state.workspace);

    lines.push(Line::from(vec![
        Span::styled(" Kloud ", theme.claude_bold),
        Span::styled(format!("v{VERSION}"), theme.inactive),
    ]));
    lines.push(Line::from(""));

    for row in DASHBOARD_LOGO {
        lines.push(Line::from(Span::styled(*row, theme.claude)));
    }
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(format!("Welcome to {ws_name}!"), theme.text_bold)));
    lines.push(Line::from(""));

    let model_effort = format!("{} \u{00B7} {}", state.model, state.effort);
    lines.push(Line::from(Span::styled(
        truncate_to_width(&model_effort, width as usize),
        theme.inactive,
    )));

    lines.push(Line::from(Span::styled(
        truncate_path(&state.workspace, width as usize),
        theme.inactive,
    )));
    lines.push(Line::from(""));
}

// ============================================================================
// Per-message-type renderers
// ============================================================================

fn render_user_message<'a>(
    msg: &'a crate::ui::tui::state::DisplayMessage,
    theme: &'a Theme,
    show_timestamps: bool,
    width: u16,
    lines: &mut Vec<Line<'a>>,
) {
    for block in &msg.blocks {
        if let DisplayBlock::Text(text) = block {
            let mut text_lines = text.lines();
            if let Some(first_line) = text_lines.next() {
                let mut spans = vec![
                    Span::styled("❯ ", theme.inactive),
                    Span::styled(first_line.to_string(), theme.text),
                ];
                if show_timestamps {
                    append_timestamp_spans(&mut spans, msg, theme, width as usize);
                }
                lines.push(Line::from(spans));
            }
            for line in text_lines {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(line.to_string(), theme.inactive),
                ]));
            }
        }
    }
}

fn render_assistant_message<'a>(
    msg: &'a crate::ui::tui::state::DisplayMessage,
    msg_index: usize,
    state: &'a TuiState,
    theme: &'a Theme,
    width: u16,
    lines: &mut Vec<Line<'a>>,
) {
    let mut dot_placed = false;
    let tick = state.activity_clock.tick;
    let is_collapsed = state.app.collapsed_tools.contains_key(&msg_index);

    // Render batch tool header once if this message has multiple consecutive tool blocks.
    let mut batch_header_printed = false;
    for block in &msg.blocks {
        match block {
            DisplayBlock::Text(text) => {
                let md_lines = render_markdown(text, theme.text, theme, Some(width));
                for md_line in &md_lines {
                    let has_content =
                        md_line.spans.iter().any(|span| !span.content.as_ref().trim().is_empty());
                    if !dot_placed && has_content {
                        let mut prefixed =
                            vec![Span::styled(format!("{TOOL_CIRCLE} "), theme.text)];
                        prefixed.extend_from_slice(&md_line.spans);
                        lines.push(Line::from(prefixed));
                        dot_placed = true;
                    } else {
                        lines.push(Line::from(md_line.spans.clone()));
                    }
                }
            }
            DisplayBlock::Thinking(text) => {
                let is_streaming = matches!(state.status, AssistantStatus::Streaming);
                let is_thinking_collapsed = state.app.collapsed_thinking.contains(&msg_index);
                render_thinking_block(
                    text,
                    false,
                    is_streaming,
                    is_thinking_collapsed,
                    tick,
                    theme,
                    lines,
                );
            }
            DisplayBlock::RedactedThinking(text) => {
                let is_streaming = matches!(state.status, AssistantStatus::Streaming);
                let is_thinking_collapsed = state.app.collapsed_thinking.contains(&msg_index);
                render_thinking_block(
                    text,
                    true,
                    is_streaming,
                    is_thinking_collapsed,
                    tick,
                    theme,
                    lines,
                );
            }
            DisplayBlock::ToolUse {
                display_name,
                server_name,
                rendered_use,
                status,
                progress_text,
                ..
            } => {
                if !batch_header_printed {
                    let tool_blocks: Vec<_> = msg
                        .blocks
                        .iter()
                        .skip_while(|b| !matches!(b, DisplayBlock::ToolUse { .. }))
                        .take_while(|b| matches!(b, DisplayBlock::ToolUse { .. }))
                        .collect();
                    if tool_blocks.len() > 1 {
                        let done = tool_blocks
                            .iter()
                            .filter(|b| {
                                matches!(
                                    b,
                                    DisplayBlock::ToolUse {
                                        status: ToolStatus::Done | ToolStatus::Errored,
                                        ..
                                    }
                                )
                            })
                            .count();
                        let running = tool_blocks
                            .iter()
                            .filter(|b| {
                                matches!(
                                    b,
                                    DisplayBlock::ToolUse {
                                        status: ToolStatus::Running,
                                        ..
                                    }
                                )
                            })
                            .count();
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("{} tools", tool_blocks.len()),
                                theme.tool.add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!(" ({done}/{})", tool_blocks.len()),
                                theme.inactive,
                            ),
                            if running > 0 {
                                Span::styled(format!(" {running} running"), theme.warning)
                            } else {
                                Span::raw("")
                            },
                        ]));
                    }
                    batch_header_printed = true;
                }
                render_tool_use_line(
                    display_name,
                    server_name,
                    rendered_use,
                    status,
                    tick,
                    theme,
                    lines,
                );
                if matches!(status, ToolStatus::Running)
                    && let Some(pt) = progress_text
                {
                    lines.push(Line::from(vec![
                        ratatui::text::Span::raw("  "),
                        ratatui::text::Span::styled(pt.clone(), theme.inactive),
                    ]));
                }
            }
            DisplayBlock::ToolResult {
                rendered_result,
                output,
                kind,
                ..
            } => {
                render_tool_result_block(
                    rendered_result,
                    output,
                    *kind,
                    is_collapsed,
                    theme,
                    lines,
                );
            }
        }
    }
}

fn render_system_message<'a>(
    msg: &'a crate::ui::tui::state::DisplayMessage,
    level: &MessageLevel,
    theme: &'a Theme,
    lines: &mut Vec<Line<'a>>,
) {
    let (dot_style, text_style) = match level {
        MessageLevel::Error => (theme.error, theme.error),
        MessageLevel::Warning => (theme.warning, theme.warning),
        MessageLevel::Info => (theme.inactive, theme.inactive),
    };

    for block in &msg.blocks {
        if let DisplayBlock::Text(text) = block {
            let mut first = true;
            for line in text.lines() {
                if first {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{TOOL_CIRCLE} "), dot_style),
                        Span::styled(line.to_string(), text_style),
                    ]));
                    first = false;
                } else {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(line.to_string(), text_style),
                    ]));
                }
            }
        }
    }
}

fn render_progress_message<'a>(
    msg: &'a crate::ui::tui::state::DisplayMessage,
    theme: &'a Theme,
    lines: &mut Vec<Line<'a>>,
) {
    for block in &msg.blocks {
        if let DisplayBlock::Text(text) = block {
            for (index, line) in text.lines().enumerate() {
                if index == 0 {
                    lines.push(Line::from(vec![
                        Span::styled(format!("{TOOL_CIRCLE} "), theme.inactive),
                        Span::styled(line.to_string(), theme.inactive),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(line.to_string(), theme.inactive),
                    ]));
                }
            }
        }
    }
}

// ============================================================================
// Block-level renderers
// ============================================================================

/// `∴ Thinking` prefix (dim + italic), matching CC's `AssistantThinkingMessage.tsx`.
const THEREFORE_SIGN: &str = "\u{2234}";
const THINKING_COLLAPSE_THRESHOLD: usize = 5;

fn render_thinking_block<'a>(
    text: &'a str,
    is_redacted: bool,
    is_streaming: bool,
    collapsed: bool,
    tick: u64,
    theme: &'a Theme,
    lines: &mut Vec<Line<'a>>,
) {
    let label = if is_redacted {
        format!("{THEREFORE_SIGN} Thinking (redacted)")
    } else {
        format!("{THEREFORE_SIGN} Thinking")
    };

    if is_streaming {
        // Shimmer effect during streaming
        use crate::agent::message::ActivityAccent;
        use crate::ui::tui::state::SpinnerMode;

        let gradient = super::helpers::activity_gradient(theme, ActivityAccent::Info);
        let text_width: usize = label.chars().count();
        let shimmer_index =
            super::glimmer::compute_glimmer_index(SpinnerMode::Thinking, tick, text_width);
        let spans = super::glimmer::glimmer_spans(&label, shimmer_index, &gradient, 0.0, 0.0);
        lines.push(Line::from(spans));
    } else {
        let style = theme.inactive.add_modifier(Modifier::ITALIC);
        lines.push(Line::from(Span::styled(label, style)));
    }

    if text.is_empty() {
        return;
    }

    let content_lines: Vec<&str> = text.lines().collect();
    let should_collapse = content_lines.len() > THINKING_COLLAPSE_THRESHOLD;

    if should_collapse && collapsed && !is_streaming {
        for line in content_lines.iter().take(3) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(line.to_string(), theme.inactive),
            ]));
        }
        let hidden = content_lines.len() - 3;
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("... {hidden} more lines (Tab to expand)"), theme.suggestion),
        ]));
    } else {
        for line in &content_lines {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(line.to_string(), theme.inactive),
            ]));
        }
        if should_collapse && !is_streaming {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("(Tab to collapse)", theme.suggestion),
            ]));
        }
    }
}

/// Render markdown text to a vector of Lines with styling.
///
/// Supports: emphasis (italic), strong (bold), headings, inline code,
/// fenced/indented code blocks, lists (ordered + unordered), blockquotes,
/// horizontal rules, and links.
fn render_markdown<'a>(
    text: &'a str,
    base_style: Style,
    theme: &'a Theme,
    available_width: Option<u16>,
) -> Vec<Line<'a>> {
    use pulldown_cmark::{Alignment, CodeBlockKind, Event, Parser, Tag, TagEnd};

    let mut lines = Vec::new();
    let mut current_line: Vec<Span<'_>> = Vec::new();

    // Style stack for nested emphasis/strong
    let mut style_stack: Vec<Style> = vec![base_style];
    let active_style = |stack: &[Style]| *stack.last().unwrap_or(&base_style);

    // List tracking
    let mut list_depth: usize = 0;
    let mut ordered_index: Vec<Option<u64>> = Vec::new();
    let mut item_first_line = false;

    // Blockquote depth
    let mut blockquote_depth: usize = 0;

    let mut in_code_block = false;
    let mut code_lang: Option<String> = None;
    let mut highlight_lines: Option<syntect::easy::HighlightLines<'static>> = None;
    let mut code_lines_buf = String::new();

    // Table state
    let mut in_table = false;
    let mut table_alignments: Vec<Alignment> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell = String::new();

    let mut opts = pulldown_cmark::Options::empty();
    opts.insert(pulldown_cmark::Options::ENABLE_TABLES);
    let parser = Parser::new_ext(text, opts);

    let subtle_style = theme.subtle;

    fn flush_line<'a>(
        current_line: &mut Vec<Span<'a>>,
        lines: &mut Vec<Line<'a>>,
        blockquote_depth: usize,
        subtle_style: Style,
    ) {
        let mut result: Vec<Span<'a>> = Vec::new();
        for _ in 0..blockquote_depth {
            result.push(Span::styled("│ ", subtle_style));
        }
        result.append(current_line);
        lines.push(Line::from(result));
    }

    for event in parser {
        match event {
            Event::Start(Tag::Emphasis) if !in_table => {
                let s = active_style(&style_stack).add_modifier(Modifier::ITALIC);
                style_stack.push(s);
            }
            Event::End(TagEnd::Emphasis)
            | Event::End(TagEnd::Strong)
            | Event::End(TagEnd::Link)
                if !in_table =>
            {
                style_stack.pop();
            }
            Event::Start(Tag::Strong) if !in_table => {
                let s = active_style(&style_stack).add_modifier(Modifier::BOLD);
                style_stack.push(s);
            }
            Event::Start(Tag::Heading {
                level,
                ..
            }) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                let prefix = "#".repeat(level as usize);
                current_line.push(Span::styled(
                    format!("{prefix} "),
                    theme.claude.add_modifier(Modifier::BOLD),
                ));
                let s = theme.claude.add_modifier(Modifier::BOLD);
                style_stack.push(s);
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                current_line.clear();
                style_stack.pop();
            }
            Event::Start(Tag::BlockQuote(_)) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                blockquote_depth += 1;
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                blockquote_depth = blockquote_depth.saturating_sub(1);
            }
            Event::Start(Tag::List(first_number)) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                list_depth += 1;
                ordered_index.push(first_number);
            }
            Event::End(TagEnd::List(_)) => {
                list_depth = list_depth.saturating_sub(1);
                ordered_index.pop();
            }
            Event::Start(Tag::Item) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                let indent = "  ".repeat(list_depth.saturating_sub(1));
                let bullet = if let Some(Some(n)) = ordered_index.last_mut() {
                    let b = format!("{n}. ");
                    *n += 1;
                    b
                } else {
                    "- ".to_string()
                };
                current_line
                    .push(Span::styled(format!("{indent}{bullet}"), active_style(&style_stack)));
                item_first_line = true;
            }
            Event::End(TagEnd::Item) | Event::SoftBreak | Event::HardBreak => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                item_first_line = false;
            }
            Event::Start(Tag::Link {
                dest_url,
                ..
            }) => {
                style_stack.push(active_style(&style_stack).add_modifier(Modifier::UNDERLINED));
                let _ = dest_url;
            }
            Event::Start(Tag::Paragraph) | Event::End(TagEnd::Paragraph)
                if !current_line.is_empty() =>
            {
                flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                current_line.clear();
            }
            Event::Text(s) => {
                if in_table {
                    current_cell.push_str(&s);
                } else if in_code_block {
                    // Buffer code text for gutter-width calculation at end of block
                    code_lines_buf.push_str(&s);
                } else {
                    current_line.push(Span::styled(s.to_string(), active_style(&style_stack)));
                }
            }
            Event::Code(s) => {
                if in_table {
                    current_cell.push_str(&s);
                } else {
                    current_line.push(Span::styled(
                        format!("`{s}`"),
                        active_style(&style_stack).add_modifier(Modifier::DIM),
                    ));
                }
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                in_code_block = true;
                if lang.is_empty() {
                    code_lang = None;
                    highlight_lines = None;
                } else {
                    let lang_str = lang.to_string();
                    let supported = super::highlight::is_language_supported(&lang_str);
                    if supported {
                        code_lang = Some(lang_str.clone());
                        highlight_lines = None; // lazy init on first line
                    } else {
                        code_lang = None;
                        highlight_lines = None;
                    }
                    // Language label as dim standalone line
                    lines.push(Line::from(vec![Span::styled(lang_str, theme.subtle)]));
                }
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                in_code_block = true;
                code_lang = None;
                highlight_lines = None;
            }
            Event::End(TagEnd::CodeBlock) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }

                // Render buffered code lines with line-number gutter
                let code_text = std::mem::take(&mut code_lines_buf);
                let code_lines: Vec<&str> = code_text.lines().collect();
                let gutter_width = code_lines.len().to_string().len().max(1);
                let gutter_prefix = |line_num: usize| -> Vec<Span<'a>> {
                    let num_str = format!("{:>width$}", line_num, width = gutter_width);
                    vec![Span::styled(num_str, theme.subtle), Span::styled("  ", theme.subtle)]
                };

                for (i, code_line) in code_lines.iter().enumerate() {
                    let highlighted = if let Some(ref lang) = code_lang {
                        super::highlight::highlight_line(lang, code_line, &mut highlight_lines)
                    } else {
                        vec![Span::styled(code_line.to_string(), theme.text)]
                    };
                    let mut spans = gutter_prefix(i + 1);
                    spans.extend(highlighted);
                    lines.push(Line::from(spans));
                }

                in_code_block = false;
                code_lang = None;
                highlight_lines = None;
            }
            Event::Rule => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                lines.push(Line::from(Span::styled("────────────", theme.subtle)));
            }
            // ── Table events ──
            Event::Start(Tag::Table(alignment)) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                in_table = true;
                table_alignments = alignment.to_vec();
                table_rows.clear();
                current_row.clear();
            }
            Event::Start(Tag::TableHead) | Event::Start(Tag::TableRow) => {
                current_row.clear();
            }
            Event::Start(Tag::TableCell) => {
                current_cell.clear();
            }
            Event::End(TagEnd::TableCell) => {
                current_row.push(current_cell.clone());
            }
            Event::End(TagEnd::TableHead) | Event::End(TagEnd::TableRow) => {
                table_rows.push(current_row.clone());
                current_row.clear();
            }
            Event::End(TagEnd::Table) => {
                let table_lines = render_table(
                    &table_rows,
                    &table_alignments,
                    theme,
                    available_width.map(|w| w as usize),
                );
                lines.extend(table_lines);
                in_table = false;
                table_rows.clear();
            }
            _ => {}
        }
    }

    if !current_line.is_empty() {
        flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(text.to_string(), base_style)));
    }

    let _ = (in_code_block, item_first_line, in_table);

    lines
}

// ============================================================================
// Table rendering helpers
// ============================================================================

const ELLIPSIS: &str = "…";

fn render_table<'a>(
    rows: &[Vec<String>],
    alignments: &[pulldown_cmark::Alignment],
    theme: &'a Theme,
    max_width: Option<usize>,
) -> Vec<Line<'a>> {
    if rows.is_empty() {
        return Vec::new();
    }
    let num_cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    if num_cols == 0 {
        return Vec::new();
    }

    // Normalize rows to uniform column count
    let normalized: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            let mut r = row.clone();
            r.resize(num_cols, String::new());
            r
        })
        .collect();

    // Calculate column widths
    let mut col_widths: Vec<usize> = vec![0; num_cols];
    for row in &normalized {
        for (i, cell) in row.iter().enumerate() {
            col_widths[i] = col_widths[i].max(cell.len());
        }
    }

    // Truncate columns if total width exceeds max_width
    if let Some(mw) = max_width {
        // total = sum(col_widths) + 2*padding per col + borders: left(1) + mid*(num_cols-1) + right(1)
        let overhead = 1 + (num_cols - 1) + 1 + num_cols * 2; // borders + " " padding
        let available = mw.saturating_sub(overhead);
        let total: usize = col_widths.iter().sum();
        if total > available {
            // Shrink widest columns until we fit
            let mut widths = col_widths.clone();
            while widths.iter().sum::<usize>() > available && widths.iter().any(|&w| w > 3) {
                let max_idx =
                    widths.iter().enumerate().max_by_key(|(_, w)| *w).map(|(i, _)| i).unwrap();
                widths[max_idx] -= 1;
            }
            // Apply truncation to cells and update col_widths
            col_widths = widths;
        }
    }

    let border_style = theme.subtle;
    let header_style = theme.text_bold;
    let cell_style = theme.text;

    let mut result = Vec::new();

    // Top border
    result.push(Line::from(Span::styled(
        format_table_border('╭', '─', '┬', '╮', &col_widths),
        border_style,
    )));

    // Header row (first row)
    result.push(format_table_row(
        &normalized[0],
        &col_widths,
        alignments,
        header_style,
        border_style,
    ));

    // Separator + data rows
    if normalized.len() > 1 {
        result.push(Line::from(Span::styled(
            format_table_border('├', '─', '┼', '┤', &col_widths),
            border_style,
        )));
        for row in &normalized[1..] {
            result.push(format_table_row(row, &col_widths, alignments, cell_style, border_style));
        }
    }

    // Bottom border
    result.push(Line::from(Span::styled(
        format_table_border('╰', '─', '┴', '╯', &col_widths),
        border_style,
    )));

    result
}

fn format_table_border(
    left: char,
    fill: char,
    mid: char,
    right: char,
    col_widths: &[usize],
) -> String {
    let mut s = String::new();
    s.push(left);
    for (i, &w) in col_widths.iter().enumerate() {
        if i > 0 {
            s.push(mid);
        }
        for _ in 0..(w + 2) {
            s.push(fill);
        }
    }
    s.push(right);
    s
}

fn format_table_row<'a>(
    row: &[String],
    col_widths: &[usize],
    alignments: &[pulldown_cmark::Alignment],
    cell_style: Style,
    border_style: Style,
) -> Line<'a> {
    let mut spans: Vec<Span<'a>> = Vec::new();
    spans.push(Span::styled("│ ", border_style));

    for (i, (cell, &width)) in row.iter().zip(col_widths.iter()).enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", border_style));
        }
        let alignment = alignments.get(i).copied().unwrap_or(pulldown_cmark::Alignment::None);
        let display = truncate_cell(cell, width);
        let padded = match alignment {
            pulldown_cmark::Alignment::Center => {
                let pad = width.saturating_sub(display.len());
                let left = pad / 2;
                let right = pad - left;
                format!("{}{}{}", " ".repeat(left), display, " ".repeat(right))
            }
            pulldown_cmark::Alignment::Right => format!("{:>width$}", display, width = width),
            _ => format!("{:<width$}", display, width = width),
        };
        spans.push(Span::styled(padded, cell_style));
    }

    spans.push(Span::styled(" │", border_style));
    Line::from(spans)
}

fn truncate_cell(cell: &str, max_width: usize) -> String {
    if cell.len() <= max_width {
        return cell.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width <= ELLIPSIS.len() {
        return ELLIPSIS.chars().take(max_width).collect();
    }
    let keep = max_width - ELLIPSIS.len();
    let truncated: String = cell.chars().take(keep).collect();
    format!("{truncated}{ELLIPSIS}")
}

// ============================================================================
// Timestamp helpers
// ============================================================================

fn append_timestamp_spans(
    spans: &mut Vec<Span<'_>>,
    msg: &crate::ui::tui::state::DisplayMessage,
    theme: &Theme,
    line_width: usize,
) {
    let Some(created) = msg.created_at else {
        return;
    };
    let elapsed = created.elapsed();
    let label = format_relative_time(elapsed);
    let current_width: usize = spans.iter().map(|s| s.content.len()).sum();
    let padding = line_width.saturating_sub(current_width).saturating_sub(label.len());
    if padding > 0 {
        spans.push(Span::raw(" ".repeat(padding)));
    }
    spans.push(Span::styled(label, theme.inactive));
}

fn format_relative_time(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    if secs < 5 {
        "now".to_string()
    } else if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else {
        format!("{}h ago", secs / 3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_theme() -> Theme {
        Theme::from_scheme(crate::ui::tui::theme::ColorScheme::Dark, false)
    }

    fn extract_text(lines: &[Line<'_>]) -> Vec<String> {
        lines
            .iter()
            .map(|line| line.spans.iter().map(|s| s.content.as_ref()).collect::<String>())
            .collect()
    }

    #[test]
    fn table_simple_with_headers() {
        let md = "| Name  | Age |\n|-------|-----|\n| Alice | 30  |\n| Bob   | 25  |";
        let theme = default_theme();
        let lines = render_markdown(md, theme.text, &theme, None);
        let text = extract_text(&lines);
        assert!(text[0].starts_with('╭'));
        assert!(text.iter().any(|t| t.contains("Alice")));
        assert!(text.iter().any(|t| t.contains("Bob")));
        assert!(text.last().unwrap().starts_with('╰'));
    }

    #[test]
    fn table_alignment() {
        let md = "| Left | Center | Right |\n|:-----|:------:|------:|\n| a    | b      | c     |";
        let theme = default_theme();
        let lines = render_markdown(md, theme.text, &theme, None);
        let text = extract_text(&lines);
        // Should have header and one data row
        assert!(text.iter().any(|t| t.contains("Left")));
        assert!(text.iter().any(|t| t.contains("a")));
    }

    #[test]
    fn table_width_truncation() {
        let md = "| Short | A very very long column that should be truncated |\n|-------|---------------------------------------------------|\n| x     | y                                                 |";
        let theme = default_theme();
        // Force narrow width
        let lines = render_markdown(md, theme.text, &theme, Some(30));
        let text = extract_text(&lines);
        // Should contain ellipsis due to truncation
        assert!(text.iter().any(|t| t.contains('…')));
    }

    #[test]
    fn table_no_rows_only_header() {
        let md = "| Col1 | Col2 |\n|------|------|";
        let theme = default_theme();
        let lines = render_markdown(md, theme.text, &theme, None);
        let text = extract_text(&lines);
        assert!(text.iter().any(|t| t.contains("Col1")));
        // Should have top border, header, bottom border (3 lines)
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn truncate_cell_basic() {
        assert_eq!(truncate_cell("hello", 10), "hello");
        assert_eq!(truncate_cell("hello world", 8), "hello…");
        assert_eq!(truncate_cell("ab", 0), "");
    }

    #[test]
    fn syntax_highlight_rust_code_block() {
        let md = "```rust\nfn main() {\n    println!(\"hello\");\n}\n```";
        let theme = default_theme();
        let lines = render_markdown(md, theme.text, &theme, None);
        let text = extract_text(&lines);
        assert!(text.iter().any(|t| t.contains("rust")));
        assert!(text.iter().any(|t| t.contains("fn main")));
    }

    #[test]
    fn syntax_highlight_unknown_language_fallback() {
        let md = "```brainfuck\n++>+<\n```";
        let theme = default_theme();
        let lines = render_markdown(md, theme.text, &theme, None);
        let text = extract_text(&lines);
        assert!(text.iter().any(|t| t.contains("brainfuck")));
        assert!(text.iter().any(|t| t.contains("++")));
    }

    #[test]
    fn syntax_highlight_no_language() {
        let md = "```\nplain text\n```";
        let theme = default_theme();
        let lines = render_markdown(md, theme.text, &theme, None);
        let text = extract_text(&lines);
        // No language label for untagged code blocks — only line numbers + content
        assert!(text.iter().any(|t| t.contains("plain text")));
        // Should have line number gutter (e.g. "1 ")
        assert!(text.iter().any(|t| t.contains("1")));
    }

    #[test]
    fn code_block_line_number_gutter() {
        let md = "```rust\nfn main() {\n    println!(\"hello\");\n}\n```";
        let theme = default_theme();
        let lines = render_markdown(md, theme.text, &theme, None);
        let text = extract_text(&lines);
        // Should have line numbers 1, 2, 3
        assert!(text.iter().any(|t| t.contains("1  ")));
        assert!(text.iter().any(|t| t.contains("2  ")));
        assert!(text.iter().any(|t| t.contains("3  ")));
        // First code line should contain fn main
        assert!(text.iter().any(|t| t.contains("fn main")));
    }

    #[test]
    fn code_block_gutter_width_multi_digit() {
        // 12 lines → gutter width = 2
        let mut md = "```text\n".to_string();
        for i in 1..=12 {
            md.push_str(&format!("line {i}\n"));
        }
        md.push_str("```");
        let theme = default_theme();
        let lines = render_markdown(&md, theme.text, &theme, None);
        let text = extract_text(&lines);
        // Line 10+ should have 2-digit gutter prefix (right-aligned)
        assert!(text.iter().any(|t| t.contains("10  ")));
        assert!(text.iter().any(|t| t.contains(" 1  ")));
    }
}
