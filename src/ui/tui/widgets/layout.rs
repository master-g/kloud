//! Layout widgets: title bar, input bar, status bar.

use std::path::Path;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::llm::response::StopReason;
use crate::ui::constants::CONTEXT_METER_WIDTH;
use crate::ui::tui::state::{AssistantStatus, TuiState};
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
		Span::styled(state.workspace.as_str(), theme.subtle),
	]);

	frame.render_widget(Paragraph::new(title), area);
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
			Span::styled(
				"Ask kloud to inspect files, trace state, or use tools...",
				theme.inactive,
			),
		])
	} else {
		Line::from(vec![Span::raw("❯ "), Span::styled(input_text, theme.text)])
	};

	let paragraph = Paragraph::new(display)
		.block(
			Block::default()
				.borders(Borders::TOP | Borders::BOTTOM)
				.border_style(theme.prompt_border),
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
			theme.inactive,
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
		spans.push(Span::styled(format!("stop: {label}"), theme.inactive));
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
		theme.claude,
	));

	if let Some(hint) = state.current_command_hint() {
		spans.push(Span::raw(" │ "));
		spans.push(Span::styled(format!("/{} — {}", hint.name, hint.summary), theme.claude));
	}

	let line = Line::from(spans);
	let paragraph = Paragraph::new(line);
	frame.render_widget(paragraph, area);
}

pub(super) fn workspace_name(path: &str) -> String {
	Path::new(path).file_name().and_then(|name| name.to_str()).unwrap_or(path).to_string()
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
		theme.subtle
	};

	Span::styled(bar, style)
}

/// Truncate a string to fit within `max_width` columns, appending `…` if needed.
pub(super) fn truncate_to_width(text: &str, max_width: usize) -> String {
	if text.width() <= max_width {
		return text.to_string();
	}
	let mut out = String::new();
	let mut w = 0;
	for ch in text.chars() {
		let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
		if w + cw + 1 > max_width {
			break;
		}
		out.push(ch);
		w += cw;
	}
	out.push('\u{2026}');
	out
}

/// Middle-truncate a filesystem path (matching CC's `truncatePath`).
pub(super) fn truncate_path(path: &str, max_width: usize) -> String {
	if path.width() <= max_width {
		return path.to_string();
	}
	let parts: Vec<&str> = path.split('/').collect();
	if parts.len() <= 2 {
		return truncate_to_width(path, max_width);
	}
	let first = parts.first().copied().unwrap_or("");
	let last = parts.last().copied().unwrap_or("");

	let result = format!("{first}/\u{2026}/{last}");
	if result.width() <= max_width {
		return result;
	}
	truncate_to_width(path, max_width)
}
