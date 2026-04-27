//! Diff renderer: unified diff with colored lines and line-number gutter.

use ratatui::text::{Line, Span};

use crate::ui::tui::theme::Theme;

/// Detect unified diff content by looking for `@@ -\d+(,\d+)? \+\d+(,\d+)? @@` hunk headers.
pub fn looks_like_diff(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    for line in text.lines() {
        if line.starts_with("@@ ") && line.contains("@@") && line.len() > 4 {
            // Check for the pattern @@ -\d+ \+\d+ @@
            let content = &line[3..];
            if let Some(end) = content.find("@@") {
                let inner = &content[..end].trim();
                // Must have - and + parts separated by space
                let parts: Vec<&str> = inner.split_whitespace().collect();
                if parts.len() >= 2
                    && parts[0].starts_with('-')
                    && parts[0][1..].chars().next().is_some_and(|c| c.is_ascii_digit())
                    && parts[1].starts_with('+')
                    && parts[1][1..].chars().next().is_some_and(|c| c.is_ascii_digit())
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Render a unified diff string into styled lines.
/// Returns at most `max_lines` content lines plus a truncation indicator if needed.
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
