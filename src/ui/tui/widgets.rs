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
use crate::ui::constants::{
	ACTIVITY_FADE_WINDOW_MS, ACTIVITY_FRAMES, ACTIVITY_MIN_RETAIN, ACTIVITY_TICK_DIVISOR,
	CONTEXT_METER_WIDTH, DASHBOARD_LOGO, DEFAULT_ACTIVITY_VERB,
};

use super::state::{
	ActivityAccent, ActivityEntryKind, AssistantStatus, DisplayBlock, LiveActivity, ToolStatus,
	TuiState,
};
use super::theme::{ColorScheme, Theme};

/// Render the full TUI layout into the given frame.
pub fn render(frame: &mut Frame, state: &TuiState, theme: &Theme) {
	let chunks = Layout::vertical([
		Constraint::Length(1), // title bar
		Constraint::Min(1),    // messages
		Constraint::Length(3), // input
		Constraint::Length(1), // status bar
	])
	.split(frame.area());

	render_title(frame, state, theme, chunks[0]);
	if state.messages.is_empty() {
		render_dashboard(frame, state, theme, chunks[1]);
	} else {
		render_messages(frame, state, theme, chunks[1]);
	}
	render_input(frame, state, theme, chunks[2]);
	render_status(frame, state, theme, chunks[3]);
}

fn render_title(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
	let workspace_name = workspace_name(&state.workspace);
	let title = Line::from(vec![
		Span::styled(" Kloud ", theme.brand_badge),
		Span::raw(" "),
		Span::styled(format!("{} with {}", state.model, state.effort), theme.text_bold),
		Span::raw(" │ "),
		Span::styled(format!("branch: {}", state.branch), theme.warning),
		Span::raw(" │ "),
		Span::styled(workspace_name, theme.success),
		Span::raw(" "),
		Span::styled(state.workspace.as_str(), theme.dim),
	]);

	frame.render_widget(Paragraph::new(title), area);
}

fn render_dashboard(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
	let columns =
		Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)]).split(area);

	render_welcome_card(frame, state, theme, columns[0]);
	render_side_panel(frame, state, theme, columns[1]);
}

fn render_welcome_card(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
	let workspace_name = workspace_name(&state.workspace);

	let mut lines = vec![
		Line::from(""),
		Line::from(Span::styled(format!("Welcome to {}", workspace_name), theme.info_bold)),
		Line::from(Span::styled(
			"Minimal agent workbench for reading, thinking, and using tools",
			theme.muted,
		)),
		Line::from(""),
	];

	for row in DASHBOARD_LOGO {
		lines.push(Line::from(Span::styled(*row, theme.warning)));
	}

	lines.extend([
		Line::from(""),
		Line::from(vec![
			Span::styled("model ", theme.dim),
			Span::styled(state.model.as_str(), theme.text_bold),
			Span::raw("   "),
			Span::styled("effort ", theme.dim),
			Span::styled(state.effort.as_str(), theme.warning_bold),
		]),
		Line::from(Span::styled(state.workspace.as_str(), theme.muted)),
		Line::from(""),
		metric_line("branch", state.branch.as_str(), theme.muted, theme.warning_bold),
		metric_line("tools", &state.tool_count.to_string(), theme.muted, theme.tool_bold),
		metric_line(
			"guides",
			&state.instruction_files.len().to_string(),
			theme.muted,
			theme.info_bold,
		),
		metric_line("hooks", &state.hook_count.to_string(), theme.muted, theme.success_bold),
		metric_line("messages", &state.messages.len().to_string(), theme.muted, theme.muted),
		Line::from(""),
		Line::from(Span::styled(
			"Ask about project files, architecture, or tool behavior.",
			theme.text,
		)),
		Line::from(Span::styled(
			"Use /help for commands, or inspect the transcript once a session starts.",
			theme.muted,
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

fn render_side_panel(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
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
			Line::from(vec![Span::styled("• ", theme.warning), Span::styled(tip, theme.text)])
		})
		.collect();

	let recent_activity = if state.recent_activity.is_empty() {
		vec![Line::from(Span::styled("No recent activity", theme.dim))]
	} else {
		state
			.recent_activity
			.iter()
			.rev()
			.take(4)
			.map(|item| {
				let (prefix, style) = activity_style(theme, item.kind);
				Line::from(vec![
					Span::styled(format!("{prefix} "), style),
					Span::styled(item.text.clone(), theme.text),
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
			Span::styled("Branch: ", theme.muted),
			Span::styled(state.branch.as_str(), theme.warning),
		]),
		Line::from(vec![
			Span::styled("Tools: ", theme.muted),
			Span::styled(state.tool_count.to_string(), theme.tool),
		]),
		Line::from(vec![
			Span::styled("Hooks: ", theme.muted),
			Span::styled(state.hook_count.to_string(), theme.success),
		]),
	];

	if state.instruction_files.is_empty() {
		project_lines.push(Line::from(Span::styled("No CLAUDE/AGENTS file found", theme.dim)));
	} else {
		project_lines.push(Line::from(""));
		for file in &state.instruction_files {
			project_lines.push(Line::from(vec![
				Span::styled("• ", theme.info),
				Span::styled(file.as_str(), theme.text),
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
			theme.muted,
		)),
		Line::from(Span::styled(
			format!("Active tools: {}", state.active_tools.len()),
			theme.muted,
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
fn render_messages(
	frame: &mut Frame,
	state: &TuiState,
	theme: &Theme,
	area: ratatui::layout::Rect,
) {
	let mut lines: Vec<Line<'_>> = Vec::new();

	for (index, msg) in state.messages.iter().enumerate() {
		let is_last_message = index + 1 == state.messages.len();
		let show_live_assistant_header = msg.role == "Assistant"
			&& is_last_message
			&& matches!(state.status, AssistantStatus::Streaming | AssistantStatus::Cancelling);
		let mut assistant_text_prefixed = false;

		if msg.role == "Error" {
			lines.push(Line::from(vec![Span::styled("error", theme.error_bold)]));
		} else if show_live_assistant_header {
			lines.push(render_live_assistant_header(state, theme));
		}

		for (block_index, block) in msg.blocks.iter().enumerate() {
			match block {
				DisplayBlock::Text(text) => {
					if msg.role == "You" {
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
					} else if msg.role == "Assistant" && !show_live_assistant_header {
						let mut text_lines = text.lines();
						if let Some(first_line) = text_lines.next() {
							if !assistant_text_prefixed {
								lines.push(Line::from(vec![
									Span::styled("⏺ ", theme.info),
									Span::styled(first_line.to_string(), theme.text),
								]));
								assistant_text_prefixed = true;
							} else {
								lines.push(Line::from(Span::styled(
									first_line.to_string(),
									theme.text,
								)));
							}
						}
						for line in text_lines {
							lines.push(Line::from(vec![
								Span::raw("  "),
								Span::styled(line.to_string(), theme.muted),
							]));
						}
					} else {
						for line in text.lines() {
							lines.push(Line::from(Span::styled(line.to_string(), theme.text)));
						}
					}
				}
				DisplayBlock::Thinking(text) => {
					for line in text.lines() {
						lines.push(Line::from(Span::styled(
							format!("[thinking] {line}"),
							theme.dim,
						)));
					}
				}
				DisplayBlock::RedactedThinking(text) => {
					if text.is_empty() {
						lines.push(Line::from(Span::styled("[thinking redacted]", theme.dim)));
					} else {
						for line in text.lines() {
							lines.push(Line::from(Span::styled(
								format!("[thinking redacted] {line}"),
								theme.dim,
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
					let status_style = match status {
						ToolStatus::Running => theme.warning,
						ToolStatus::Done => theme.success,
						ToolStatus::Errored => theme.error,
					};

					lines.push(Line::from(vec![
						Span::styled("[tool] ", theme.tool_bold),
						Span::styled(name.as_str(), theme.tool),
						Span::raw(" "),
						Span::styled(status_text, status_style),
					]));

					for line in input_preview.lines() {
						lines.push(Line::from(Span::styled(format!("args: {line}"), theme.dim)));
					}
				}
				DisplayBlock::ToolResult {
					name,
					output,
					is_error,
					..
				} => {
					let (prefix, prefix_style, name_style) = if *is_error {
						("[tool result] [error]", theme.error_bold, theme.error)
					} else {
						("[tool result]", theme.success_bold, theme.success)
					};

					lines.push(Line::from(vec![
						Span::styled(prefix, prefix_style),
						Span::raw(" "),
						Span::styled(name.as_str(), name_style),
					]));

					for line in output.lines() {
						lines.push(Line::from(Span::styled(line.to_string(), theme.text)));
					}
				}
			}

			if msg.role == "You" && block_index + 1 == msg.blocks.len() {
				continue;
			}
		}

		lines.push(Line::from(""));
	}

	if matches!(state.status, AssistantStatus::Streaming | AssistantStatus::Cancelling)
		&& let Some(last) = lines.last_mut()
	{
		last.spans.push(Span::styled("█", theme.warning));
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

fn render_live_assistant_header(state: &TuiState, theme: &Theme) -> Line<'static> {
	if state.status == AssistantStatus::Cancelling {
		return Line::from(vec![
			Span::styled(active_glyph(state.animation_tick), theme.warning_bold),
			Span::raw(" "),
			Span::styled("Cancelling", theme.warning_bold),
			Span::raw(" "),
			Span::styled("current turn", theme.text),
		]);
	}

	if let Some(activity) = &state.live_activity {
		let elapsed = format_elapsed(activity.started_at.elapsed());
		let token_suffix = if state.output_tokens > 0 {
			format!(" · ↑ {} tokens", state.output_tokens)
		} else {
			String::new()
		};
		let styles = activity_styles(activity, theme);

		let mut spans =
			vec![Span::styled(active_glyph(state.animation_tick), styles.glyph), Span::raw(" ")];
		spans.extend(animated_verb_spans(activity, state.animation_tick, theme));
		spans.push(Span::raw(" "));
		spans.push(Span::styled(activity.object.clone(), styles.object));
		spans.push(Span::styled(format!(" ({elapsed}{token_suffix})"), theme.dim));
		return Line::from(spans);
	}

	Line::from(vec![Span::styled("assistant", theme.info_bold)])
}

/// Render the input bar.
fn render_input(frame: &mut Frame, state: &TuiState, theme: &Theme, area: ratatui::layout::Rect) {
	let show_placeholder = matches!(state.status, AssistantStatus::Idle) && state.input.is_empty();
	let (input_text, show_cursor) = match state.status {
		AssistantStatus::Streaming => ("(streaming...)".to_string(), false),
		AssistantStatus::Cancelling => ("(cancelling...)".to_string(), false),
		AssistantStatus::Idle => (state.input.clone(), true),
	};

	let display = if show_placeholder {
		Line::from(vec![
			Span::raw("> "),
			Span::styled("Ask kloud to inspect files, trace state, or use tools...", theme.dim),
		])
	} else {
		Line::from(vec![Span::raw("> "), Span::styled(input_text, theme.text)])
	};

	let paragraph = Paragraph::new(display)
		.block(Block::default().borders(Borders::TOP | Borders::BOTTOM))
		.wrap(Wrap {
			trim: false,
		});

	frame.render_widget(paragraph, area);

	if show_cursor {
		let text_before_cursor = &state.input[..state.cursor.min(state.input.len())];
		let display_width = text_before_cursor.width() as u16;
		let cursor_x = area.x + 2 + display_width;
		let cursor_y = area.y + 1;
		frame.set_cursor_position((cursor_x, cursor_y));
	}
}

/// Render the status bar.
fn render_status(frame: &mut Frame, state: &TuiState, theme: &Theme, area: ratatui::layout::Rect) {
	let mut spans = vec![
		Span::styled(format!(" {} ", state.model), theme.status_model_badge),
		Span::raw(" │ "),
		Span::styled(format!(" {} ", state.effort), theme.status_effort_badge),
		Span::raw(" │ "),
		Span::styled(format!("branch: {}", state.branch), theme.warning),
		Span::raw(" │ "),
		Span::styled(
			format!("tokens: {}↓ {}↑", state.input_tokens, state.output_tokens),
			theme.muted,
		),
		Span::raw(" │ "),
		render_context_meter(state, theme),
	];

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
		spans.push(Span::styled(format!("stop: {label}"), theme.muted));
	}

	if !state.active_tools.is_empty() {
		spans.push(Span::raw(" │ "));
		spans.push(Span::styled(format!("tools: {}", state.active_tools.join(", ")), theme.tool));
	}

	spans.push(Span::raw(" │ "));
	spans.push(Span::styled(
		format!(
			"guides: {} hooks: {} history: {}",
			state.instruction_files.len(),
			state.hook_count,
			state.history.len()
		),
		theme.info,
	));

	if let Some(hint) = state.current_command_hint() {
		spans.push(Span::raw(" │ "));
		spans.push(Span::styled(format!("/{} — {}", hint.name, hint.summary), theme.info));
	}

	let line = Line::from(spans);
	let paragraph = Paragraph::new(line);
	frame.render_widget(paragraph, area);
}

fn workspace_name(path: &str) -> String {
	Path::new(path).file_name().and_then(|name| name.to_str()).unwrap_or(path).to_string()
}

fn metric_line(label: &str, value: &str, label_style: Style, value_style: Style) -> Line<'static> {
	Line::from(vec![
		Span::styled(format!("{label}: "), label_style),
		Span::styled(value.to_string(), value_style),
	])
}

fn render_context_meter(state: &TuiState, theme: &Theme) -> Span<'static> {
	let max_context = state.max_context_tokens.max(1);
	let used = state.input_tokens.min(max_context);
	let ratio = used as f64 / max_context as f64;
	let filled = ((ratio * CONTEXT_METER_WIDTH as f64).round() as usize).min(CONTEXT_METER_WIDTH);
	let bar = format!(
		"ctx: [{}{}] {:>3}%",
		"#".repeat(filled),
		".".repeat(CONTEXT_METER_WIDTH.saturating_sub(filled)),
		(ratio * 100.0).round() as u32
	);

	let style = if ratio >= 0.9 {
		theme.error
	} else if ratio >= 0.7 {
		theme.warning
	} else {
		theme.dim
	};

	Span::styled(bar, style)
}

fn activity_style(theme: &Theme, kind: ActivityEntryKind) -> (&'static str, Style) {
	match kind {
		ActivityEntryKind::User => ("❯", theme.dim),
		ActivityEntryKind::Assistant => ("◦", theme.info),
		ActivityEntryKind::Tool => ("✦", theme.tool),
		ActivityEntryKind::Success => ("✓", theme.success),
		ActivityEntryKind::Error => ("!", theme.error),
		ActivityEntryKind::Meta => ("·", theme.muted),
	}
}

fn active_glyph(tick: u64) -> &'static str {
	let phase = (tick / ACTIVITY_TICK_DIVISOR) as usize;
	ACTIVITY_FRAMES[phase % ACTIVITY_FRAMES.len()]
}

#[derive(Clone, Copy)]
struct ActivityStyles {
	glyph: Style,
	object: Style,
}

#[derive(Clone, Copy)]
struct ActivityGradient {
	glyph: (u8, u8, u8),
	hot: (u8, u8, u8),
	base: (u8, u8, u8),
	object: (u8, u8, u8),
	fade: (u8, u8, u8),
	object_fade: (u8, u8, u8),
}

fn activity_styles(activity: &LiveActivity, theme: &Theme) -> ActivityStyles {
	if theme.no_color {
		let plain = Style::default();
		let bold = plain.add_modifier(Modifier::BOLD);
		return ActivityStyles {
			glyph: bold,
			object: plain,
		};
	}

	let retain = activity_decay_mix(activity);
	let gradient = activity_gradient(theme, activity.accent);
	let glyph =
		rgb_style(blend_rgb(gradient.glyph, gradient.fade, retain)).add_modifier(Modifier::BOLD);
	let object =
		rgb_style(blend_rgb(gradient.object, gradient.object_fade, 0.28 + (retain * 0.72)));

	ActivityStyles {
		glyph,
		object,
	}
}

fn activity_gradient(theme: &Theme, accent: ActivityAccent) -> ActivityGradient {
	match (theme.color_scheme, accent) {
		(ColorScheme::Default, ActivityAccent::Info) => ActivityGradient {
			glyph: (118, 209, 255),
			hot: (170, 238, 255),
			base: (92, 173, 255),
			object: (205, 222, 255),
			fade: (82, 92, 108),
			object_fade: (104, 110, 122),
		},
		(ColorScheme::Default, ActivityAccent::Tool) => ActivityGradient {
			glyph: (224, 119, 255),
			hot: (255, 182, 247),
			base: (200, 102, 255),
			object: (235, 208, 255),
			fade: (94, 82, 112),
			object_fade: (110, 103, 122),
		},
		(ColorScheme::Light, ActivityAccent::Info) => ActivityGradient {
			glyph: (0, 120, 212),
			hot: (32, 148, 255),
			base: (30, 102, 196),
			object: (44, 60, 88),
			fade: (136, 144, 156),
			object_fade: (146, 152, 162),
		},
		(ColorScheme::Light, ActivityAccent::Tool) => ActivityGradient {
			glyph: (156, 62, 201),
			hot: (198, 98, 232),
			base: (132, 54, 184),
			object: (82, 54, 98),
			fade: (144, 138, 152),
			object_fade: (152, 148, 160),
		},
	}
}

fn activity_decay_mix(activity: &LiveActivity) -> f32 {
	let elapsed_ms = activity.last_signal_at.elapsed().as_millis().min(ACTIVITY_FADE_WINDOW_MS);
	let progress = elapsed_ms as f32 / ACTIVITY_FADE_WINDOW_MS as f32;
	1.0 - progress * (1.0 - ACTIVITY_MIN_RETAIN)
}

fn blend_rgb(from: (u8, u8, u8), to: (u8, u8, u8), retain: f32) -> Color {
	let retain = retain.clamp(0.0, 1.0);
	Color::Rgb(
		blend_channel(from.0, to.0, retain),
		blend_channel(from.1, to.1, retain),
		blend_channel(from.2, to.2, retain),
	)
}

fn blend_channel(from: u8, to: u8, retain: f32) -> u8 {
	let retained = from as f32 * retain;
	let faded = to as f32 * (1.0 - retain);
	(retained + faded).round() as u8
}

fn rgb_style(color: Color) -> Style {
	Style::default().fg(color)
}

fn animated_verb_spans(activity: &LiveActivity, tick: u64, theme: &Theme) -> Vec<Span<'static>> {
	let verb = current_activity_verb(activity);
	let gradient = activity_gradient(theme, activity.accent);
	shimmer_word_spans(verb, tick, gradient)
}

fn current_activity_verb(activity: &LiveActivity) -> &str {
	activity
		.verbs
		.get(activity.verb_index)
		.map(String::as_str)
		.filter(|verb| !verb.is_empty())
		.unwrap_or(DEFAULT_ACTIVITY_VERB)
}

fn shimmer_word_spans(text: &str, tick: u64, gradient: ActivityGradient) -> Vec<Span<'static>> {
	let chars: Vec<char> = text.chars().collect();
	if chars.is_empty() {
		return vec![Span::styled(String::new(), rgb_style(Color::Rgb(gradient.fade.0, gradient.fade.1, gradient.fade.2)))];
	}

	let hotspot = ((tick / ACTIVITY_TICK_DIVISOR) as usize) % chars.len();
	let shimmer_color = Color::Rgb(gradient.hot.0, gradient.hot.1, gradient.hot.2);
	let message_color = Color::Rgb(gradient.base.0, gradient.base.1, gradient.base.2);
	let mut spans = Vec::new();

	for (index, ch) in chars.into_iter().enumerate() {
		let distance = index.abs_diff(hotspot);
		// Claude Code 方式: 离散颜色切换，只有 shimmer 或 message 两种状态
		let is_near = distance <= 1;
		let color = if is_near { shimmer_color } else { message_color };
		let style = rgb_style(color).add_modifier(Modifier::BOLD);
		spans.push(Span::styled(ch.to_string(), style));
	}

	spans
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
