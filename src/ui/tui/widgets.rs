//! TUI render functions.
//!
//! Draws the three-panel layout: messages area, input bar, status bar.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use super::state::{AssistantStatus, DisplayBlock, TuiState};

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
		let role_style = if msg.role == "You" {
			Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
		} else {
			Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
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
			}
		}

		// Blank line between messages
		lines.push(Line::from(""));
	}

	// Streaming indicator
	if state.status == AssistantStatus::Streaming
		&& let Some(last) = lines.last_mut()
	{
		last.spans.push(Span::styled("█", Style::default().fg(Color::Yellow)));
	}

	// Auto-scroll: compute how much to scroll so the bottom is visible
	let content_height = lines.len() as u16;
	let visible_height = area.height.saturating_sub(2); // account for border
	let max_scroll = content_height.saturating_sub(visible_height);
	let scroll = if state.status == AssistantStatus::Streaming {
		max_scroll // always follow during streaming
	} else {
		state.scroll.min(max_scroll)
	};

	let paragraph = Paragraph::new(lines)
		.block(Block::default().borders(Borders::ALL).title(" kloud "))
		.wrap(Wrap {
			trim: false,
		})
		.scroll((scroll, 0));

	frame.render_widget(paragraph, area);
}

/// Render the input bar.
fn render_input(frame: &mut Frame, state: &TuiState, area: ratatui::layout::Rect) {
	let input_text = if state.status == AssistantStatus::Streaming {
		"(streaming...)".to_string()
	} else {
		state.input.clone()
	};

	let paragraph = Paragraph::new(input_text)
		.block(Block::default().borders(Borders::ALL).title(" > "))
		.wrap(Wrap {
			trim: false,
		});

	frame.render_widget(paragraph, area);

	// Place cursor
	if state.status == AssistantStatus::Idle {
		// Calculate cursor position using display width (not byte offset)
		// This ensures correct positioning for wide characters like CJK
		let text_before_cursor = &state.input[..state.cursor.min(state.input.len())];
		let display_width = text_before_cursor.width() as u16;
		let cursor_x = area.x + 1 + display_width;
		let cursor_y = area.y + 1;
		frame.set_cursor_position((cursor_x, cursor_y));
	}
}

/// Render the status bar.
fn render_status(frame: &mut Frame, state: &TuiState, area: ratatui::layout::Rect) {
	let status_text = match state.status {
		AssistantStatus::Idle => "ready",
		AssistantStatus::Streaming => "streaming...",
	};

	let line = Line::from(vec![
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
		Span::styled(status_text, Style::default().fg(Color::Yellow)),
	]);

	let paragraph = Paragraph::new(line);
	frame.render_widget(paragraph, area);
}
