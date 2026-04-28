//! Tool use and tool result block rendering.
//!
//! Implements CC-style `ToolUseLoader` blinking indicator and
//! `MessageResponse` (`⎿`) prefix for tool results.
//!
//! This module only handles **layout** (status dot, collapse/expand, `⎿` prefix).
//! Content rendering is done by Tool trait methods at the store layer.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::tools::ToolResultKind;
use crate::ui::tui::constants::TOOL_CIRCLE;
use crate::ui::tui::state::ToolStatus;
use crate::ui::tui::theme::Theme;

/// Render a tool use header line with blinking dot for running state.
///
/// Merges the status dot into the first rendered line (CC pattern):
/// - Non-empty rendered: `⏺ <rendered first line>`, subsequent lines indented
/// - Empty rendered (fallback/MCP): `⏺ <name>` or `⏺ <server> - <name>`
pub(super) fn render_tool_use_line<'a>(
    name: &'a str,
    server_name: &'a Option<String>,
    rendered: &[Line<'static>],
    status: &ToolStatus,
    tick: u64,
    theme: &'a Theme,
    lines: &mut Vec<Line<'a>>,
) {
    let (dot_text, dot_style) = match status {
        ToolStatus::Pending | ToolStatus::Running => {
            let blink_on = (tick / 6).is_multiple_of(2);
            let dot = if blink_on {
                TOOL_CIRCLE
            } else {
                " "
            };
            (dot.to_string(), theme.subtle)
        }
        ToolStatus::Done => (TOOL_CIRCLE.to_string(), theme.success),
        ToolStatus::Errored => (TOOL_CIRCLE.to_string(), theme.error),
    };

    // Fallback: no rendered content — use raw tool name (unregistered/MCP tools
    // where render_tool_use_message() produced nothing).
    if rendered.is_empty() {
        let display_name = server_name
            .as_ref()
            .map(|s| format!("{s} - {name}"))
            .unwrap_or_else(|| name.to_string());
        lines.push(Line::from(vec![
            Span::styled(format!("{dot_text} "), dot_style),
            Span::styled(display_name, theme.tool.add_modifier(Modifier::BOLD)),
        ]));
        return;
    }

    // Merge dot into first rendered line
    let mut iter = rendered.iter();
    if let Some(first) = iter.next() {
        let mut spans: Vec<Span<'a>> = vec![Span::styled(format!("{dot_text} "), dot_style)];
        if let Some(srv) = server_name {
            spans.push(Span::styled(format!("{srv} - "), theme.inactive));
        }
        for span in &first.spans {
            spans.push(Span::styled(span.content.to_string(), span.style));
        }
        lines.push(Line::from(spans));

        for line in iter {
            lines.push(prepend_prefix_to_line(line, "  ", theme.inactive));
        }
    }
}

/// Maximum lines shown before collapsing tool output.
const COLLAPSE_THRESHOLD: usize = 5;

/// Render a tool result block using CC's `MessageResponse` (`⎿`) prefix.
pub(super) fn render_tool_result_block<'a>(
    rendered: &[Line<'static>],
    kind: ToolResultKind,
    collapsed: bool,
    theme: &'a Theme,
    lines: &mut Vec<Line<'a>>,
) {
    match kind {
        ToolResultKind::Canceled => {
            lines.push(Line::from(vec![Span::raw("  "), Span::styled("Canceled", theme.inactive)]));
            return;
        }
        ToolResultKind::Rejected => {
            lines.push(Line::from(vec![Span::raw("  "), Span::styled("Rejected", theme.inactive)]));
            return;
        }
        ToolResultKind::Error | ToolResultKind::Success => {}
    }

    if rendered.is_empty() {
        return;
    }

    let text_style = match kind {
        ToolResultKind::Error => theme.error,
        _ => theme.inactive,
    };
    let prefix_style = theme.inactive;

    let should_collapse = rendered.len() > COLLAPSE_THRESHOLD;

    if should_collapse && collapsed {
        for (i, line) in rendered.iter().take(COLLAPSE_THRESHOLD).enumerate() {
            if i == 0 {
                lines.push(prepend_prefix_to_line(line, "⎿ ", prefix_style));
            } else {
                lines.push(prepend_prefix_to_line(line, "  ", text_style));
            }
        }
        let hidden = rendered.len() - COLLAPSE_THRESHOLD;
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("... {hidden} more lines (Tab to expand)"), theme.suggestion),
        ]));
    } else {
        let mut first = true;
        for line in rendered {
            if first {
                lines.push(prepend_prefix_to_line(line, "⎿ ", prefix_style));
                first = false;
            } else {
                lines.push(prepend_prefix_to_line(line, "  ", text_style));
            }
        }
        if should_collapse {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("(Tab to collapse)", theme.suggestion),
            ]));
        }
    }
}

/// Prepend a prefix string (styled) to an existing Line by rebuilding its spans.
fn prepend_prefix_to_line(line: &Line<'_>, prefix: &str, prefix_style: Style) -> Line<'static> {
    let mut spans = vec![Span::styled(prefix.to_string(), prefix_style)];
    for span in &line.spans {
        spans.push(Span::styled(span.content.to_string(), span.style));
    }
    Line::from(spans)
}
