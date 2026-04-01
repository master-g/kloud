//! Tool use and tool result block rendering.
//!
//! Implements CC-style `ToolUseLoader` blinking indicator and
//! `MessageResponse` (`⎿`) prefix for tool results.

use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use crate::ui::constants::TOOL_CIRCLE;
use crate::ui::tui::state::ToolStatus;
use crate::ui::tui::theme::Theme;

/// Render a tool use header line with blinking dot for running state.
///
/// Matches CC's `ToolUseLoader` + `AssistantToolUseMessage` pattern:
/// - Running: dim circle blinks on/off (~300ms cycle at 50ms ticks)
/// - Done: solid green circle
/// - Errored: solid red circle
/// - Tool name is always bold
pub(super) fn render_tool_use_line<'a>(
	name: &'a str,
	server_name: &'a Option<String>,
	input_preview: &'a str,
	status: &ToolStatus,
	tick: u64,
	theme: &'a Theme,
	lines: &mut Vec<Line<'a>>,
) {
	let display_name =
		server_name.as_ref().map(|s| format!("{s} - {name}")).unwrap_or_else(|| name.to_string());

	let (dot_text, dot_style, name_style) = match status {
		ToolStatus::Running => {
			// ~300ms blink cycle: 6 ticks on, 6 ticks off (at 50ms/tick)
			let blink_on = (tick / 6).is_multiple_of(2);
			let dot = if blink_on {
				TOOL_CIRCLE
			} else {
				" "
			};
			(dot.to_string(), theme.dim, theme.tool.add_modifier(Modifier::BOLD))
		}
		ToolStatus::Done => {
			(TOOL_CIRCLE.to_string(), theme.success, theme.tool.add_modifier(Modifier::BOLD))
		}
		ToolStatus::Errored => {
			(TOOL_CIRCLE.to_string(), theme.error, theme.tool.add_modifier(Modifier::BOLD))
		}
	};

	lines.push(Line::from(vec![
		Span::styled(format!("{dot_text} "), dot_style),
		Span::styled(display_name, name_style),
	]));

	if !input_preview.is_empty() {
		for line in input_preview.lines() {
			lines
				.push(Line::from(vec![Span::raw("  "), Span::styled(line.to_string(), theme.dim)]));
		}
	}
}

/// Render a tool result block using CC's `MessageResponse` (`⎿`) prefix.
///
/// - Error results use `theme.error` for text
/// - Success results use `theme.dim` for text
/// - Empty output is silently skipped
pub(super) fn render_tool_result_block<'a>(
	output: &'a str,
	is_error: bool,
	theme: &'a Theme,
	lines: &mut Vec<Line<'a>>,
) {
	let text_style = if is_error {
		theme.error
	} else {
		theme.dim
	};
	let prefix_style = theme.dim;

	if output.is_empty() {
		return;
	}

	let mut first = true;
	for line in output.lines() {
		if first {
			lines.push(Line::from(vec![
				Span::styled("⎿ ", prefix_style),
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
