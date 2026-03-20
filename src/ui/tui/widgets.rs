//! TUI render functions.
//!
//! Draws the three-panel layout: messages area, input bar, status bar.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::llm::response::StopReason;

use super::state::{AssistantStatus, DisplayBlock, ToolStatus, TuiState};

/// Render the full TUI layout into the given frame.
pub fn render(frame: &mut Frame, state: &TuiState) {
	let chunks = Layout::vertical([
		Constraint::Min(1),    // messages
		Constraint::Length(3), // input
		Constraint::Length(1), // status bar
	])
	.split(frame.area());

	render_messages(frame, state, chunks[0]);
	render_input(frame, state, chunks[1]);
	render_status(frame, state, chunks[2]);
}

/// Render the scrollable messages area.
fn render_messages(frame: &mut Frame, state: &TuiState, area: ratatui::layout::Rect) {
	let mut lines: Vec<Line<'_>> = Vec::new();

	for msg in &state.messages {
		// Role header
		let role_style = match msg.role.as_str() {
			"You" => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
			"Error" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
			_ => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
		};

		lines.push(Line::from(vec![Span::styled(format!("{}: ", msg.role), role_style)]));

		// Content blocks
		for block in &msg.blocks {
			match block {
				DisplayBlock::Text(text) => {
					for line in text.lines() {
						lines.push(Line::from(line.to_string()));
					}
				}
				DisplayBlock::Thinking(text) => {
					for line in text.lines() {
						lines.push(Line::from(Span::styled(
							format!("[thinking] {line}"),
							Style::default().fg(Color::DarkGray),
						)));
					}
				}
				DisplayBlock::ToolUse {
					name,
					input_preview,
					status,
					..
				} => {
					let status_text = match status {
						ToolStatus::Running => "(running)",
						ToolStatus::Done => "(done)",
						ToolStatus::Errored => "(error)",
					};
					let status_color = match status {
						ToolStatus::Running => Color::Yellow,
						ToolStatus::Done => Color::Green,
						ToolStatus::Errored => Color::Red,
					};

					lines.push(Line::from(vec![
						Span::styled(
							"[tool] ",
							Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
						),
						Span::styled(name.as_str(), Style::default().fg(Color::Magenta)),
						Span::raw(" "),
						Span::styled(status_text, Style::default().fg(status_color)),
					]));

					for line in input_preview.lines() {
						lines.push(Line::from(Span::styled(
							format!("args: {line}"),
							Style::default().fg(Color::DarkGray),
						)));
					}
				}
				DisplayBlock::ToolResult {
					name,
					output,
					is_error,
					..
				} => {
					let (prefix, color) = if *is_error {
						("[tool result] [error]", Color::Red)
					} else {
						("[tool result]", Color::Green)
					};

					lines.push(Line::from(vec![
						Span::styled(
							prefix,
							Style::default().fg(color).add_modifier(Modifier::BOLD),
						),
						Span::raw(" "),
						Span::styled(name.as_str(), Style::default().fg(color)),
					]));

					for line in output.lines() {
						lines.push(Line::from(line.to_string()));
					}
				}
			}
		}

		// Blank line between messages
		lines.push(Line::from(""));
	}

	// Streaming indicator
	if matches!(state.status, AssistantStatus::Streaming | AssistantStatus::Cancelling)
		&& let Some(last) = lines.last_mut()
	{
		last.spans.push(Span::styled("█", Style::default().fg(Color::Yellow)));
	}

	// Auto-scroll: compute how much to scroll so the bottom is visible
	let content_height = lines.len() as u16;
	let visible_height = area.height;
	let max_scroll = content_height.saturating_sub(visible_height);
	let scroll = if matches!(state.status, AssistantStatus::Streaming | AssistantStatus::Cancelling)
	{
		max_scroll // always follow during streaming / cancelling
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

/// Render the input bar.
fn render_input(frame: &mut Frame, state: &TuiState, area: ratatui::layout::Rect) {
	let (input_text, show_cursor) = match state.status {
		AssistantStatus::Streaming => ("(streaming...)".to_string(), false),
		AssistantStatus::Cancelling => ("(cancelling...)".to_string(), false),
		AssistantStatus::Idle => (state.input.clone(), true),
	};

	let display = format!("> {input_text}");

	let paragraph = Paragraph::new(display)
		.block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
		.wrap(Wrap {
			trim: false,
		});

	frame.render_widget(paragraph, area);

	// Place cursor when idle.
	if show_cursor {
		let text_before_cursor = &state.input[..state.cursor.min(state.input.len())];
		let display_width = text_before_cursor.width() as u16;
		let cursor_x = area.x + 2 + display_width; // "> " prefix
		let cursor_y = area.y + 1; // below top border
		frame.set_cursor_position((cursor_x, cursor_y));
	}
}

/// Render the status bar.
fn render_status(frame: &mut Frame, state: &TuiState, area: ratatui::layout::Rect) {
	let status_label = match state.status {
		AssistantStatus::Idle => "ready",
		AssistantStatus::Streaming => "streaming...",
		AssistantStatus::Cancelling => "cancelling...",
	};

	let spinner = match state.status {
		AssistantStatus::Streaming | AssistantStatus::Cancelling => {
			match state.animation_tick % 4 {
				0 => "-",
				1 => "\\",
				2 => "|",
				_ => "/",
			}
		}
		AssistantStatus::Idle => " ",
	};

	let mut spans = vec![
		Span::styled(
			format!(" {} ", state.model),
			Style::default().fg(Color::White).bg(Color::DarkGray),
		),
		Span::raw(" │ "),
		Span::styled(
			format!("tokens: {}↓ {}↑", state.input_tokens, state.output_tokens),
			Style::default().fg(Color::Gray),
		),
		Span::raw(" │ "),
		Span::styled(format!("{status_label} {spinner}"), Style::default().fg(Color::Yellow)),
	];

	// Last stop reason
	if let Some(reason) = &state.last_stop_reason {
		let label = match reason {
			StopReason::EndTurn => "end_turn",
			StopReason::MaxTokens => "max_tokens",
			StopReason::StopSequence => "stop_sequence",
			StopReason::ToolUse => "tool_use",
			StopReason::PauseTurn => "pause_turn",
			StopReason::Refusal => "refusal",
			StopReason::ModelContextWindowExceeded => "ctx_exceeded",
		};
		spans.push(Span::raw(" │ "));
		spans.push(Span::styled(format!("stop: {label}"), Style::default().fg(Color::Gray)));
	}

	// Active tools
	if !state.active_tools.is_empty() {
		spans.push(Span::raw(" │ "));
		spans.push(Span::styled(
			format!("tools: {}", state.active_tools.join(", ")),
			Style::default().fg(Color::Magenta),
		));
	}

	// Slash command inline help
	if let Some(hint) = state.current_command_hint() {
		spans.push(Span::raw(" │ "));
		spans.push(Span::styled(
			format!("/{} — {}", hint.name, hint.summary),
			Style::default().fg(Color::Cyan),
		));
	}

	let line = Line::from(spans);
	let paragraph = Paragraph::new(line);
	frame.render_widget(paragraph, area);
}
