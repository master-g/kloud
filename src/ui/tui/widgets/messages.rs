//! Message rendering: conversation messages and markdown formatting.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::activity_line::render_live_assistant_header;
use super::tool_block::{render_tool_result_block, render_tool_use_line};
use crate::ui::constants::TOOL_CIRCLE;
use crate::ui::tui::state::{AssistantStatus, DisplayBlock, MessageLevel, MessageType, TuiState};
use crate::ui::tui::theme::Theme;

/// Render the scrollable messages area.
pub(super) fn render_messages(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
	let mut lines: Vec<Line<'_>> = Vec::new();

	for msg in &state.messages {
		match &msg.message_type {
			MessageType::User => {
				render_user_message(msg, theme, &mut lines);
			}
			MessageType::Assistant => {
				render_assistant_message(msg, state, theme, &mut lines);
			}
			MessageType::System {
				level,
			} => {
				render_system_message(msg, level, theme, &mut lines);
			}
		}
		lines.push(Line::from(""));
	}

	// Keep activity line inside transcript flow so it scrolls with messages.
	if state.displayed_activity().is_some() || state.status == AssistantStatus::Cancelling {
		lines.push(render_live_assistant_header(state, theme, area.width));
	}

	let content_height = lines.len() as u16;
	let visible_height = area.height;
	let max_scroll = content_height.saturating_sub(visible_height);
	let scroll = if matches!(state.status, AssistantStatus::Streaming | AssistantStatus::Cancelling)
	{
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
					Span::styled("❯ ", theme.dim),
					Span::styled(first_line.to_string(), theme.text),
				]));
			}
			for line in text_lines {
				lines.push(Line::from(vec![
					Span::raw("  "),
					Span::styled(line.to_string(), theme.muted),
				]));
			}
		}
	}
}

fn render_assistant_message<'a>(
	msg: &'a crate::ui::tui::state::DisplayMessage,
	state: &'a TuiState,
	theme: &'a Theme,
	lines: &mut Vec<Line<'a>>,
) {
	let mut dot_placed = false;
	let tick = state.activity_clock.tick;

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
				render_tool_use_line(name, server_name, input_preview, status, tick, theme, lines);
			}
			DisplayBlock::ToolResult {
				output,
				is_error,
				..
			} => {
				render_tool_result_block(output, *is_error, theme, lines);
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
		MessageLevel::Info => (theme.dim, theme.dim),
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
	let style = theme.dim.add_modifier(Modifier::ITALIC);

	if text.is_empty() {
		lines.push(Line::from(Span::styled(label, style)));
		return;
	}
	lines.push(Line::from(Span::styled(label, style)));
	for line in text.lines() {
		lines.push(Line::from(vec![Span::raw("  "), Span::styled(line.to_string(), theme.dim)]));
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

	let dim_style = theme.dim;

	fn flush_line<'a>(
		current_line: &mut Vec<Span<'a>>,
		lines: &mut Vec<Line<'a>>,
		blockquote_depth: usize,
		dim_style: Style,
	) {
		let mut result: Vec<Span<'a>> = Vec::new();
		for _ in 0..blockquote_depth {
			result.push(Span::styled("│ ", dim_style));
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
					flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
					current_line.clear();
				}
				let prefix = "#".repeat(level as usize);
				current_line.push(Span::styled(
					format!("{prefix} "),
					theme.info.add_modifier(Modifier::BOLD),
				));
				let s = theme.info.add_modifier(Modifier::BOLD);
				style_stack.push(s);
			}
			Event::End(TagEnd::Heading(_)) => {
				flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
				current_line.clear();
				style_stack.pop();
			}
			Event::Start(Tag::BlockQuote(_)) => {
				if !current_line.is_empty() {
					flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
					current_line.clear();
				}
				blockquote_depth += 1;
			}
			Event::End(TagEnd::BlockQuote(_)) => {
				if !current_line.is_empty() {
					flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
					current_line.clear();
				}
				blockquote_depth = blockquote_depth.saturating_sub(1);
			}
			Event::Start(Tag::List(first_number)) => {
				if !current_line.is_empty() {
					flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
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
					flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
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
					flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
					current_line.clear();
				}
				item_first_line = false;
			}
			Event::Start(Tag::Link {
				dest_url,
				..
			}) => {
				style_stack.push(active_style(&style_stack).add_modifier(Modifier::UNDERLINED));
				// Store URL for end tag — we just render inline with underline
				let _ = dest_url;
			}
			Event::Start(Tag::Paragraph) | Event::End(TagEnd::Paragraph) => {
				if !current_line.is_empty() {
					flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
					current_line.clear();
				}
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
					flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
					current_line.clear();
				}
				in_code_block = true;
				if lang.is_empty() {
					lines.push(Line::from(vec![Span::styled("┌─ code", theme.dim)]));
				} else {
					lines.push(Line::from(vec![
						Span::styled("┌─ ", theme.dim),
						Span::styled(lang.to_string(), theme.info),
					]));
				}
			}
			Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
				if !current_line.is_empty() {
					flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
					current_line.clear();
				}
				in_code_block = true;
				lines.push(Line::from(vec![Span::styled("┌─ code", theme.dim)]));
			}
			Event::End(TagEnd::CodeBlock) => {
				if !current_line.is_empty() {
					flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
					current_line.clear();
				}
				in_code_block = false;
				lines.push(Line::from(vec![Span::styled("└─", theme.dim)]));
			}
			Event::Rule => {
				if !current_line.is_empty() {
					flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
					current_line.clear();
				}
				lines.push(Line::from(Span::styled("────────────", theme.dim)));
			}
			_ => {}
		}
	}

	if !current_line.is_empty() {
		flush_line(&mut current_line, &mut lines, blockquote_depth, dim_style);
	}

	if lines.is_empty() {
		lines.push(Line::from(Span::styled(text.to_string(), base_style)));
	}

	let _ = (in_code_block, item_first_line);

	lines
}
