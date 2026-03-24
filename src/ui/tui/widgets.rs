//! TUI render functions.
//!
//! Draws the terminal workbench: title bar, main area, input bar, status bar.

use std::path::Path;
use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::llm::response::StopReason;

use super::state::{
	ActivityEntryKind, ActivityKind, AssistantStatus, DisplayBlock, ToolStatus, TuiState,
};

const ACTIVITY_TICK_DIVISOR: u64 = 4;

/// Render the full TUI layout into the given frame.
pub fn render(frame: &mut Frame, state: &TuiState) {
	let chunks = Layout::vertical([
		Constraint::Length(1), // title bar
		Constraint::Min(1),    // messages
		Constraint::Length(3), // input
		Constraint::Length(1), // status bar
	])
	.split(frame.area());

	render_title(frame, state, chunks[0]);
	if state.messages.is_empty() {
		render_dashboard(frame, state, chunks[1]);
	} else {
		render_messages(frame, state, chunks[1]);
	}
	render_input(frame, state, chunks[2]);
	render_status(frame, state, chunks[3]);
}

fn render_title(frame: &mut Frame, state: &TuiState, area: Rect) {
	let workspace_name = workspace_name(&state.workspace);
	let title = Line::from(vec![
		Span::styled(
			" Kloud ",
			Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
		),
		Span::raw(" "),
		Span::styled(
			format!("{} with {}", state.model, state.effort),
			Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
		),
		Span::raw(" │ "),
		Span::styled(format!("branch: {}", state.branch), Style::default().fg(Color::Yellow)),
		Span::raw(" │ "),
		Span::styled(workspace_name, Style::default().fg(Color::Green)),
		Span::raw(" "),
		Span::styled(state.workspace.as_str(), Style::default().fg(Color::DarkGray)),
	]);

	frame.render_widget(Paragraph::new(title), area);
}

fn render_dashboard(frame: &mut Frame, state: &TuiState, area: Rect) {
	let columns =
		Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)]).split(area);

	render_welcome_card(frame, state, columns[0]);
	render_side_panel(frame, state, columns[1]);
}

fn render_welcome_card(frame: &mut Frame, state: &TuiState, area: Rect) {
	let workspace_name = workspace_name(&state.workspace);
	let logo = [
		"                 ▗▄▄▄▖                 ",
		"               ▗███████▖               ",
		"              ▐███▛▀▜███▌              ",
		"              ▝███▄▄▄███▘              ",
		"                ▀▀   ▀▀                ",
	];

	let mut lines = vec![
		Line::from(""),
		Line::from(Span::styled(
			format!("Welcome to {}", workspace_name),
			Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
		)),
		Line::from(Span::styled(
			"Minimal agent workbench for reading, thinking, and using tools",
			Style::default().fg(Color::Gray),
		)),
		Line::from(""),
	];

	for row in logo {
		lines.push(Line::from(Span::styled(row, Style::default().fg(Color::Yellow))));
	}

	lines.extend([
		Line::from(""),
		Line::from(vec![
			Span::styled("model ", Style::default().fg(Color::DarkGray)),
			Span::styled(
				state.model.as_str(),
				Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
			),
			Span::raw("   "),
			Span::styled("effort ", Style::default().fg(Color::DarkGray)),
			Span::styled(
				state.effort.as_str(),
				Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
			),
		]),
		Line::from(Span::styled(state.workspace.as_str(), Style::default().fg(Color::Gray))),
		Line::from(""),
		metric_line("branch", state.branch.as_str(), Color::Yellow),
		metric_line("tools", &state.tool_count.to_string(), Color::Magenta),
		metric_line("guides", &state.instruction_files.len().to_string(), Color::Cyan),
		metric_line("hooks", &state.hook_count.to_string(), Color::Green),
		metric_line("messages", &state.messages.len().to_string(), Color::Gray),
		Line::from(""),
		Line::from(Span::styled(
			"Ask about project files, architecture, or tool behavior.",
			Style::default().fg(Color::White),
		)),
		Line::from(Span::styled(
			"Use /help for commands, or inspect the transcript once a session starts.",
			Style::default().fg(Color::Gray),
		)),
	]);

	let paragraph = Paragraph::new(lines)
		.alignment(Alignment::Center)
		.block(Block::default().title(" Welcome ").borders(Borders::ALL))
		.wrap(Wrap {
			trim: false,
		});

	frame.render_widget(paragraph, area);
}

fn render_side_panel(frame: &mut Frame, state: &TuiState, area: Rect) {
	let chunks = Layout::vertical([
		Constraint::Percentage(38),
		Constraint::Percentage(28),
		Constraint::Percentage(34),
	])
	.split(area);

	let tips = [
		"Start with a concrete file path or function name.",
		"Ask kloud to read a file before answering code questions.",
		"Tool calls and tool results appear inline in the transcript.",
		"Thinking, redacted thinking, and tool activity are all distinct blocks.",
	];

	let tip_lines: Vec<Line<'_>> = tips
		.into_iter()
		.map(|tip| {
			Line::from(vec![Span::styled("• ", Style::default().fg(Color::Yellow)), Span::raw(tip)])
		})
		.collect();

	let recent_activity = if state.recent_activity.is_empty() {
		vec![Line::from(Span::styled("No recent activity", Style::default().fg(Color::DarkGray)))]
	} else {
		state
			.recent_activity
			.iter()
			.rev()
			.take(4)
			.map(|item| {
				let (prefix, color) = activity_style(item.kind);
				Line::from(vec![
					Span::styled(format!("{prefix} "), Style::default().fg(color)),
					Span::styled(item.text.clone(), Style::default().fg(Color::White)),
				])
			})
			.collect()
	};

	let tips_widget = Paragraph::new(tip_lines)
		.block(Block::default().title(" Tips ").borders(Borders::ALL))
		.wrap(Wrap {
			trim: false,
		});
	frame.render_widget(tips_widget, chunks[0]);

	let mut project_lines = vec![
		Line::from(vec![
			Span::styled("Branch: ", Style::default().fg(Color::Gray)),
			Span::styled(state.branch.as_str(), Style::default().fg(Color::Yellow)),
		]),
		Line::from(vec![
			Span::styled("Tools: ", Style::default().fg(Color::Gray)),
			Span::styled(state.tool_count.to_string(), Style::default().fg(Color::Magenta)),
		]),
		Line::from(vec![
			Span::styled("Hooks: ", Style::default().fg(Color::Gray)),
			Span::styled(state.hook_count.to_string(), Style::default().fg(Color::Green)),
		]),
	];

	if state.instruction_files.is_empty() {
		project_lines.push(Line::from(Span::styled(
			"No CLAUDE/AGENTS file found",
			Style::default().fg(Color::DarkGray),
		)));
	} else {
		project_lines.push(Line::from(""));
		for file in &state.instruction_files {
			project_lines.push(Line::from(vec![
				Span::styled("• ", Style::default().fg(Color::Cyan)),
				Span::raw(file.as_str()),
			]));
		}
	}

	let project_widget = Paragraph::new(project_lines)
		.block(Block::default().title(" Project Signals ").borders(Borders::ALL))
		.wrap(Wrap {
			trim: false,
		});
	frame.render_widget(project_widget, chunks[1]);

	let mut activity_lines = vec![
		Line::from(Span::styled(
			format!("Feed entries: {}", state.recent_activity.len()),
			Style::default().fg(Color::Gray),
		)),
		Line::from(Span::styled(
			format!("Active tools: {}", state.active_tools.len()),
			Style::default().fg(Color::Gray),
		)),
		Line::from(""),
	];
	activity_lines.extend(recent_activity);

	let activity_widget = Paragraph::new(activity_lines)
		.block(Block::default().title(" Recent Activity ").borders(Borders::ALL))
		.wrap(Wrap {
			trim: false,
		});
	frame.render_widget(activity_widget, chunks[2]);
}

/// Render the scrollable messages area.
fn render_messages(frame: &mut Frame, state: &TuiState, area: ratatui::layout::Rect) {
	let mut lines: Vec<Line<'_>> = Vec::new();

	for (index, msg) in state.messages.iter().enumerate() {
		let is_last_message = index + 1 == state.messages.len();
		let show_live_assistant_header = msg.role == "Assistant"
			&& is_last_message
			&& matches!(state.status, AssistantStatus::Streaming | AssistantStatus::Cancelling);
		let mut assistant_text_prefixed = false;

		if msg.role == "Error" {
			lines.push(Line::from(vec![Span::styled(
				"error",
				Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
			)]));
		} else if show_live_assistant_header {
			lines.push(render_live_assistant_header(state));
		}

		// Content blocks
		for (block_index, block) in msg.blocks.iter().enumerate() {
			match block {
				DisplayBlock::Text(text) => {
					if msg.role == "You" {
						let mut text_lines = text.lines();
						if let Some(first_line) = text_lines.next() {
							lines.push(Line::from(vec![
								Span::styled("❯ ", Style::default().fg(Color::DarkGray)),
								Span::styled(
									first_line.to_string(),
									Style::default().fg(Color::White),
								),
							]));
						}
						for line in text_lines {
							lines.push(Line::from(vec![
								Span::raw("  "),
								Span::styled(line.to_string(), Style::default().fg(Color::Gray)),
							]));
						}
					} else if msg.role == "Assistant" && !show_live_assistant_header {
						let mut text_lines = text.lines();
						if let Some(first_line) = text_lines.next() {
							if !assistant_text_prefixed {
								lines.push(Line::from(vec![
									Span::styled("⏺ ", Style::default().fg(Color::Cyan)),
									Span::styled(
										first_line.to_string(),
										Style::default().fg(Color::White),
									),
								]));
								assistant_text_prefixed = true;
							} else {
								lines.push(Line::from(first_line.to_string()));
							}
						}
						for line in text_lines {
							lines.push(Line::from(vec![
								Span::raw("  "),
								Span::styled(line.to_string(), Style::default().fg(Color::Gray)),
							]));
						}
					} else {
						for line in text.lines() {
							lines.push(Line::from(line.to_string()));
						}
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
				DisplayBlock::RedactedThinking(text) => {
					if text.is_empty() {
						lines.push(Line::from(Span::styled(
							"[thinking redacted]",
							Style::default().fg(Color::DarkGray),
						)));
					} else {
						for line in text.lines() {
							lines.push(Line::from(Span::styled(
								format!("[thinking redacted] {line}"),
								Style::default().fg(Color::DarkGray),
							)));
						}
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

			if msg.role == "You" && block_index + 1 == msg.blocks.len() {
				continue;
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

fn render_live_assistant_header(state: &TuiState) -> Line<'static> {
	if state.status == AssistantStatus::Cancelling {
		return Line::from(vec![
			Span::styled(
				active_glyph(state.animation_tick),
				Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
			),
			Span::raw(" "),
			Span::styled(
				"Cancelling",
				Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
			),
			Span::raw(" current turn"),
		]);
	}

	if let Some(activity) = &state.live_activity {
		let (verb, object) = activity_copy(activity);
		let elapsed = format_elapsed(activity.started_at.elapsed());
		let token_suffix = if state.output_tokens > 0 {
			format!(" · ↑ {} tokens", state.output_tokens)
		} else {
			String::new()
		};

		let mut spans = vec![
			Span::styled(
				active_glyph(state.animation_tick),
				Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
			),
			Span::raw(" "),
		];
		spans.extend(animated_verb_spans(&verb, state.animation_tick));
		spans.push(Span::raw(" "));
		spans.push(Span::styled(object, Style::default().fg(Color::White)));
		spans.push(Span::styled(
			format!(" ({elapsed}{token_suffix})"),
			Style::default().fg(Color::DarkGray),
		));
		return Line::from(spans);
	}

	Line::from(vec![Span::styled(
		"assistant",
		Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
	)])
}

/// Render the input bar.
fn render_input(frame: &mut Frame, state: &TuiState, area: ratatui::layout::Rect) {
	let show_placeholder = matches!(state.status, AssistantStatus::Idle) && state.input.is_empty();
	let (input_text, show_cursor) = match state.status {
		AssistantStatus::Streaming => ("(streaming...)".to_string(), false),
		AssistantStatus::Cancelling => ("(cancelling...)".to_string(), false),
		AssistantStatus::Idle => (state.input.clone(), true),
	};

	let display = if show_placeholder {
		Line::from(vec![
			Span::raw("> "),
			Span::styled(
				"Ask kloud to inspect files, trace state, or use tools...",
				Style::default().fg(Color::DarkGray),
			),
		])
	} else {
		Line::from(format!("> {input_text}"))
	};

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
	let mut spans = vec![
		Span::styled(
			format!(" {} ", state.model),
			Style::default().fg(Color::White).bg(Color::DarkGray),
		),
		Span::raw(" │ "),
		Span::styled(
			format!(" {} ", state.effort),
			Style::default().fg(Color::Black).bg(Color::Yellow),
		),
		Span::raw(" │ "),
		Span::styled(format!("branch: {}", state.branch), Style::default().fg(Color::Yellow)),
		Span::raw(" │ "),
		Span::styled(
			format!("tokens: {}↓ {}↑", state.input_tokens, state.output_tokens),
			Style::default().fg(Color::Gray),
		),
		Span::raw(" │ "),
		render_context_meter(state),
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

	spans.push(Span::raw(" │ "));
	spans.push(Span::styled(
		format!(
			"guides: {} hooks: {} history: {}",
			state.instruction_files.len(),
			state.hook_count,
			state.history.len()
		),
		Style::default().fg(Color::Cyan),
	));

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

fn workspace_name(path: &str) -> String {
	Path::new(path).file_name().and_then(|name| name.to_str()).unwrap_or(path).to_string()
}

fn metric_line(label: &str, value: &str, color: Color) -> Line<'static> {
	Line::from(vec![
		Span::styled(format!("{label}: "), Style::default().fg(Color::DarkGray)),
		Span::styled(value.to_string(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
	])
}

fn render_context_meter(state: &TuiState) -> Span<'static> {
	const WIDTH: usize = 10;

	let max_context = state.max_context_tokens.max(1);
	let used = state.input_tokens.min(max_context);
	let ratio = used as f64 / max_context as f64;
	let filled = ((ratio * WIDTH as f64).round() as usize).min(WIDTH);
	let bar = format!(
		"ctx: [{}{}] {:>3}%",
		"#".repeat(filled),
		".".repeat(WIDTH.saturating_sub(filled)),
		(ratio * 100.0).round() as u32
	);

	let color = if ratio >= 0.9 {
		Color::Red
	} else if ratio >= 0.7 {
		Color::Yellow
	} else {
		Color::DarkGray
	};

	Span::styled(bar, Style::default().fg(color))
}

fn activity_style(kind: ActivityEntryKind) -> (&'static str, Color) {
	match kind {
		ActivityEntryKind::User => ("❯", Color::DarkGray),
		ActivityEntryKind::Assistant => ("◦", Color::Cyan),
		ActivityEntryKind::Tool => ("✦", Color::Magenta),
		ActivityEntryKind::Success => ("✓", Color::Green),
		ActivityEntryKind::Error => ("!", Color::Red),
		ActivityEntryKind::Meta => ("·", Color::Gray),
	}
}

fn active_glyph(tick: u64) -> &'static str {
	const FRAMES: [&str; 6] = ["·", "✻", "✽", "✶", "✳", "✢"];
	let phase = (tick / ACTIVITY_TICK_DIVISOR) as usize;
	FRAMES[phase % FRAMES.len()]
}

fn activity_copy(activity: &super::state::LiveActivity) -> (String, String) {
	match &activity.kind {
		ActivityKind::Thinking => ("Thinking".to_string(), "through the request".to_string()),
		ActivityKind::Tool {
			name,
		} => match name.as_str() {
			"read" => ("Reading".to_string(), "project files".to_string()),
			"echo" => ("Echoing".to_string(), "tool output".to_string()),
			other => ("Using".to_string(), format!("tool `{other}`")),
		},
	}
}

fn animated_verb_spans(word: &str, tick: u64) -> Vec<Span<'static>> {
	let chars: Vec<char> = word.chars().collect();
	if chars.is_empty() {
		return vec![];
	}

	let phase = (tick / ACTIVITY_TICK_DIVISOR) as usize;
	let hotspot = phase % (chars.len() + 2);

	chars
		.into_iter()
		.enumerate()
		.map(|(index, ch)| {
			let style = if index == hotspot {
				Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
			} else if index + 1 == hotspot || hotspot + 1 == index {
				Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
			} else {
				Style::default().fg(Color::DarkGray)
			};

			Span::styled(ch.to_string(), style)
		})
		.collect()
}

fn format_elapsed(duration: Duration) -> String {
	let seconds = duration.as_secs();
	if seconds < 60 {
		format!("{seconds}s")
	} else {
		let minutes = seconds / 60;
		let remainder = seconds % 60;
		format!("{minutes}m {remainder:02}s")
	}
}
