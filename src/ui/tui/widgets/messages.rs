//! Message rendering: conversation messages and markdown formatting.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::activity_line::render_live_assistant_header;
use super::layout::{truncate_path, truncate_to_width, workspace_name};
use super::tool_block::{render_tool_result_block, render_tool_use_line};
use crate::ui::constants::{DASHBOARD_LOGO, TOOL_CIRCLE, VERSION};
use crate::ui::tui::state::{
    AssistantStatus, DisplayBlock, MessageLevel, MessageType, ToolStatus, TuiState,
};
use crate::ui::tui::theme::Theme;

/// Render the scrollable messages area with a logo header at the top.
pub(super) fn render_messages(frame: &mut Frame, state: &mut TuiState, theme: &Theme, area: Rect) {
    let mut lines: Vec<Line<'_>> = Vec::new();

    // Populate virtual scroll cache before any immutable borrows of state.
    let msg_count = state.messages.len();
    let show_range = if msg_count > 100 {
        state.virtual_scroll.ensure_cache(&state.messages, area.width);
        let (s, _, e) = state.virtual_scroll.visible_message_range(area.height, 0);
        Some((s, e))
    } else {
        None
    };

    // Search bar at top of messages area
    if state.screen == crate::ui::tui::state::Screen::Search {
        let match_info = state.search.match_display();
        let wrapped = if state.search.wrapped {
            " (wrapped)"
        } else {
            ""
        };
        lines.push(Line::from(vec![
            Span::styled(" /", theme.claude_bold),
            Span::styled(state.search.query.clone(), theme.text),
            Span::styled(format!(" {match_info}{wrapped}"), theme.suggestion),
        ]));
    }

    if state.screen != crate::ui::tui::state::Screen::Transcript {
        render_logo_header(state, theme, area.width, &mut lines);
    }

    let messages: Vec<&crate::ui::tui::state::DisplayMessage> =
        if let Some((start, end)) = show_range {
            state.messages.iter().skip(start).take(end.saturating_sub(start) + 1).collect()
        } else {
            state.messages.iter().collect()
        };

    let msg_offset = show_range.map(|(s, _)| s).unwrap_or(0);
    for (local_idx, msg) in messages.iter().enumerate() {
        let msg_idx = local_idx + msg_offset;
        match &msg.message_type {
            MessageType::User => {
                render_user_message(msg, theme, &mut lines);
            }
            MessageType::Assistant => {
                render_assistant_message(msg, msg_idx, state, theme, &mut lines);
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
    if let Some(perm) = &state.pending_permission {
        lines.extend(super::permission::render_permission_prompt(theme, perm));
    }

    let content_height = lines.len() as u16;
    let visible_height = area.height;
    let max_scroll = content_height.saturating_sub(visible_height);

    let is_streaming = state.screen != crate::ui::tui::state::Screen::Transcript
        && matches!(state.status, AssistantStatus::Streaming | AssistantStatus::Cancelling);

    let should_auto_scroll = !state.auto_scroll_paused;
    let scroll = if should_auto_scroll {
        max_scroll
    } else {
        state.scroll.min(max_scroll)
    };

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap {
            trim: false,
        })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);

    if should_auto_scroll {
        state.scroll = max_scroll;
    }

    // Auto-scroll paused indicator
    if state.auto_scroll_paused && is_streaming && visible_height > 2 {
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
    lines: &mut Vec<Line<'a>>,
) {
    for block in &msg.blocks {
        if let DisplayBlock::Text(text) = block {
            let mut text_lines = text.lines();
            if let Some(first_line) = text_lines.next() {
                lines.push(Line::from(vec![
                    Span::styled("❯ ", theme.inactive),
                    Span::styled(first_line.to_string(), theme.text),
                ]));
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
    lines: &mut Vec<Line<'a>>,
) {
    let mut dot_placed = false;
    let tick = state.activity_clock.tick;
    let is_collapsed = state.collapsed_tools.contains_key(&msg_index);

    // Render batch tool header once if this message has multiple consecutive tool blocks.
    let mut batch_header_printed = false;
    for block in &msg.blocks {
        match block {
            DisplayBlock::Text(text) => {
                let md_lines = render_markdown(text, theme.text, theme);
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
                render_thinking_block(text, false, theme, lines);
            }
            DisplayBlock::RedactedThinking(text) => {
                render_thinking_block(text, true, theme, lines);
            }
            DisplayBlock::ToolUse {
                name,
                server_name,
                input_preview,
                status,
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
                render_tool_use_line(name, server_name, input_preview, status, tick, theme, lines);
            }
            DisplayBlock::ToolResult {
                output,
                is_error,
                ..
            } => {
                render_tool_result_block(output, *is_error, is_collapsed, theme, lines);
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

fn render_thinking_block<'a>(
    text: &'a str,
    is_redacted: bool,
    theme: &'a Theme,
    lines: &mut Vec<Line<'a>>,
) {
    let label = if is_redacted {
        format!("{THEREFORE_SIGN} Thinking (redacted)")
    } else {
        format!("{THEREFORE_SIGN} Thinking")
    };
    let style = theme.inactive.add_modifier(Modifier::ITALIC);

    if text.is_empty() {
        lines.push(Line::from(Span::styled(label, style)));
        return;
    }
    lines.push(Line::from(Span::styled(label, style)));
    for line in text.lines() {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(line.to_string(), theme.inactive),
        ]));
    }
}

/// Render markdown text to a vector of Lines with styling.
///
/// Supports: emphasis (italic), strong (bold), headings, inline code,
/// fenced/indented code blocks, lists (ordered + unordered), blockquotes,
/// horizontal rules, and links.
fn render_markdown<'a>(text: &'a str, base_style: Style, theme: &'a Theme) -> Vec<Line<'a>> {
    use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

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

    let parser = Parser::new(text);

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
            Event::Start(Tag::Emphasis) => {
                let s = active_style(&style_stack).add_modifier(Modifier::ITALIC);
                style_stack.push(s);
            }
            Event::End(TagEnd::Emphasis)
            | Event::End(TagEnd::Strong)
            | Event::End(TagEnd::Link) => {
                style_stack.pop();
            }
            Event::Start(Tag::Strong) => {
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
                current_line.push(Span::styled(s.to_string(), active_style(&style_stack)));
            }
            Event::Code(s) => {
                current_line.push(Span::styled(
                    format!("`{s}`"),
                    active_style(&style_stack).add_modifier(Modifier::DIM),
                ));
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                in_code_block = true;
                if lang.is_empty() {
                    lines.push(Line::from(vec![Span::styled("┌─ code", theme.subtle)]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled("┌─ ", theme.subtle),
                        Span::styled(lang.to_string(), theme.claude),
                    ]));
                }
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                in_code_block = true;
                lines.push(Line::from(vec![Span::styled("┌─ code", theme.subtle)]));
            }
            Event::End(TagEnd::CodeBlock) => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                in_code_block = false;
                lines.push(Line::from(vec![Span::styled("└─", theme.subtle)]));
            }
            Event::Rule => {
                if !current_line.is_empty() {
                    flush_line(&mut current_line, &mut lines, blockquote_depth, subtle_style);
                    current_line.clear();
                }
                lines.push(Line::from(Span::styled("────────────", theme.subtle)));
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

    let _ = (in_code_block, item_first_line);

    lines
}
