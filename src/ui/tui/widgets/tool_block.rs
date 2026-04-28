//! Tool use and tool result block rendering.
//!
//! Renders tool blocks using CC-style left-indent indicators:
//! ```text
//! ⏺ ToolName(input_summary)
//!   content line 1
//!   content line 2
//! ```
//!
//! Tool results use `⎿` (U+23BF) left indent on first line, then
//! indented continuation:
//! ```text
//!   ⎿  result line 1 (summary)
//!       result line 2
//!       result line 3
//! ```

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use super::diff;
use crate::tools::ToolResultKind;
use crate::ui::tui::constants::{INDENT_MARKER, TOOL_CIRCLE};
use crate::ui::tui::state::ToolStatus;
use crate::ui::tui::theme::Theme;

/// Maximum lines shown before collapsing tool output.
const COLLAPSE_THRESHOLD: usize = 5;

/// Render a tool use block as a single-line header + indented content.
///
/// First line: `⏺ ToolName(input_summary)`
/// Remaining lines: 2-space indented content.
pub(super) fn render_tool_use_line<'a>(
    display_name: &'a str,
    _server_name: &'a Option<String>,
    rendered: &[Line<'static>],
    status: &ToolStatus,
    tick: u64,
    theme: &'a Theme,
    lines: &mut Vec<Line<'a>>,
) {
    let dot_style = match status {
        ToolStatus::Pending | ToolStatus::Running => {
            let blink_on = (tick / 6).is_multiple_of(2);
            if blink_on {
                theme.subtle
            } else {
                let summary = summary_from_rendered(rendered);
                lines.push(Line::from(vec![
                    Span::styled("  ", theme.subtle),
                    Span::styled(display_name.to_string(), theme.tool.add_modifier(Modifier::BOLD)),
                    Span::styled(summary, theme.inactive),
                ]));
                emit_indented_content(rendered, lines);
                return;
            }
        }
        ToolStatus::Done => theme.success,
        ToolStatus::Errored => theme.error,
    };

    let summary = summary_from_rendered(rendered);

    // Single line: ⏺ DisplayName(summary)
    lines.push(Line::from(vec![
        Span::styled(format!("{TOOL_CIRCLE} "), dot_style),
        Span::styled(display_name.to_string(), theme.tool.add_modifier(Modifier::BOLD)),
        Span::styled(summary, theme.inactive),
    ]));

    emit_indented_content(rendered, lines);
}

/// Build input summary from first rendered line, formatted as `(content)`.
fn summary_from_rendered(rendered: &[Line<'static>]) -> String {
    let text: String = rendered
        .first()
        .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect::<String>())
        .unwrap_or_default();
    if text.is_empty() {
        String::new()
    } else {
        format!("({text})")
    }
}

/// Emit remaining content lines (skip first) with 2-space indent.
fn emit_indented_content<'a>(rendered: &[Line<'static>], lines: &mut Vec<Line<'a>>) {
    for line in rendered.iter().skip(1) {
        let mut spans = vec![Span::raw("  ")];
        for span in &line.spans {
            spans.push(Span::styled(span.content.to_string(), span.style));
        }
        lines.push(Line::from(spans));
    }
}

/// Render a tool result block using `⎿` left-indent indicators.
pub(super) fn render_tool_result_block<'a>(
    rendered: &[Line<'static>],
    raw_output: &str,
    kind: ToolResultKind,
    collapsed: bool,
    theme: &'a Theme,
    lines: &mut Vec<Line<'a>>,
) {
    match kind {
        ToolResultKind::Canceled => {
            lines.push(first_line(
                &Line::from(Span::styled("Canceled", theme.inactive)),
                theme.subtle,
                theme.inactive,
            ));
            return;
        }
        ToolResultKind::Rejected => {
            lines.push(first_line(
                &Line::from(Span::styled("Rejected", theme.inactive)),
                theme.subtle,
                theme.inactive,
            ));
            return;
        }
        ToolResultKind::Error | ToolResultKind::Success => {}
    }

    if rendered.is_empty() && raw_output.is_empty() {
        return;
    }

    // If raw output looks like a diff, render with diff highlighting
    let diff_lines;
    let effective_rendered: &[Line<'static>] =
        if !raw_output.is_empty() && diff::looks_like_diff(raw_output) {
            diff_lines = diff::render_diff(raw_output, 100, theme);
            &diff_lines
        } else if !rendered.is_empty() {
            rendered
        } else {
            diff_lines = raw_output
                .lines()
                .map(|l| Line::from(Span::styled(l.to_string(), theme.inactive)))
                .collect();
            &diff_lines
        };

    let text_style = match kind {
        ToolResultKind::Error => theme.error,
        _ => theme.inactive,
    };

    let should_collapse = effective_rendered.len() > COLLAPSE_THRESHOLD;

    if should_collapse && collapsed {
        // First line with `⎿` marker
        if let Some(line) = effective_rendered.first() {
            lines.push(first_line(line, theme.subtle, text_style));
        }
        // Remaining visible lines: continuation indent (no `⎿`)
        for line in effective_rendered.iter().take(COLLAPSE_THRESHOLD).skip(1) {
            lines.push(continuation_line(line, text_style));
        }
        let hidden = effective_rendered.len() - COLLAPSE_THRESHOLD;
        lines.push(continuation_line(
            &Line::from(Span::styled(
                format!("... {hidden} more lines (Tab to expand)"),
                theme.suggestion,
            )),
            theme.suggestion,
        ));
    } else {
        // First line with `⎿` marker
        if let Some(line) = effective_rendered.first() {
            lines.push(first_line(line, theme.subtle, text_style));
        }
        // Remaining lines: continuation indent
        for line in effective_rendered.iter().skip(1) {
            lines.push(continuation_line(line, text_style));
        }
        if should_collapse {
            lines.push(continuation_line(
                &Line::from(Span::styled("(Tab to collapse)", theme.suggestion)),
                theme.suggestion,
            ));
        }
    }
}

/// First line of a result block: `  ⎿  content` (2-space indent + marker + 2 spaces).
fn first_line(line: &Line<'_>, marker_style: Style, text_style: Style) -> Line<'static> {
    let mut spans = vec![
        Span::styled("  ", marker_style),
        Span::styled(INDENT_MARKER, marker_style),
        Span::styled("  ", marker_style),
    ];
    for span in &line.spans {
        let style = if span.style == Style::default() {
            text_style
        } else {
            span.style
        };
        spans.push(Span::styled(span.content.to_string(), style));
    }
    Line::from(spans)
}

/// Continuation line: `     content` (5-space indent, aligned with first-line content).
fn continuation_line(line: &Line<'_>, text_style: Style) -> Line<'static> {
    let mut spans = vec![Span::raw("     ")];
    for span in &line.spans {
        let style = if span.style == Style::default() {
            text_style
        } else {
            span.style
        };
        spans.push(Span::styled(span.content.to_string(), style));
    }
    Line::from(spans)
}
