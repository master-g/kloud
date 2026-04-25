//! Diff renderer: unified diff with colored lines and line-number gutter.

use ratatui::text::{Line, Span};

use crate::ui::tui::theme::Theme;

/// Render a unified diff string into styled lines.
/// Returns at most `max_lines` content lines plus a truncation indicator if needed.
#[allow(dead_code)]
pub fn render_diff(diff: &str, max_lines: usize, theme: &Theme) -> Vec<Line<'static>> {
    let all_lines: Vec<&str> = diff.lines().collect();
    let total = all_lines.len();
    let show = all_lines.into_iter().take(max_lines);
    let mut lines: Vec<Line<'static>> = show
        .enumerate()
        .map(|(i, line)| {
            let gutter = format!("{:>4} ", i + 1);
            if line.starts_with('+') && !line.starts_with("+++") {
                Line::from(vec![
                    Span::styled(gutter, theme.subtle),
                    Span::styled(line.to_string(), theme.success),
                ])
            } else if line.starts_with('-') && !line.starts_with("---") {
                Line::from(vec![
                    Span::styled(gutter, theme.subtle),
                    Span::styled(line.to_string(), theme.error),
                ])
            } else if line.starts_with('@') || line.starts_with("diff ") {
                Line::from(vec![
                    Span::styled(gutter, theme.subtle),
                    Span::styled(line.to_string(), theme.claude),
                ])
            } else {
                Line::from(vec![
                    Span::styled(gutter, theme.subtle),
                    Span::styled(line.to_string(), theme.text),
                ])
            }
        })
        .collect();

    if total > max_lines {
        let remaining = total - max_lines;
        lines.push(Line::from(Span::styled(
            format!("  ... {remaining} more lines"),
            theme.inactive,
        )));
    }

    lines
}
