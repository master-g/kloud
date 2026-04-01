//! Layout widgets: title bar, dashboard, input bar, status bar.

use std::path::Path;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::llm::response::StopReason;
use crate::ui::constants::{CONTEXT_METER_WIDTH, DASHBOARD_LOGO};
use crate::ui::tui::state::{ActivityEntryKind, AssistantStatus, TuiState};
use crate::ui::tui::theme::Theme;

pub(super) fn render_title(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
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

pub(super) fn render_dashboard(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
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

/// Render the input bar.
pub(super) fn render_input(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
	let show_placeholder = matches!(state.status, AssistantStatus::Idle) && state.input.is_empty();
	let (input_text, show_cursor) = match state.status {
		AssistantStatus::Streaming => ("(streaming...)".to_string(), false),
		AssistantStatus::Cancelling => ("(cancelling...)".to_string(), false),
		AssistantStatus::Idle => (state.input.clone(), true),
	};

	let display = if show_placeholder {
		Line::from(vec![
			Span::raw("❯ "),
			Span::styled("Ask kloud to inspect files, trace state, or use tools...", theme.dim),
		])
	} else {
		Line::from(vec![Span::raw("❯ "), Span::styled(input_text, theme.text)])
	};

	let paragraph = Paragraph::new(display)
		.block(
			Block::default().borders(Borders::TOP | Borders::BOTTOM).border_style(theme.border_dim),
		)
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
pub(super) fn render_status(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
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
