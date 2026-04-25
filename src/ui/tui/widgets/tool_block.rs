//! Tool use and tool result block rendering.
//!
//! Implements CC-style `ToolUseLoader` blinking indicator and
//! `MessageResponse` (`⎿`) prefix for tool results.

use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use crate::ui::constants::TOOL_CIRCLE;
use crate::ui::tui::state::ToolStatus;
use crate::ui::tui::theme::Theme;

/// Tool type categories for differentiated rendering.
enum ToolKind {
    /// Shell command tools (bash, sh).
    Shell,
    /// File operation tools (`read_file`, `write_file`, `edit_file`, etc.).
    File,
    /// Web/search tools (`web_search`, `web_fetch`, etc.).
    Web,
    /// Default rendering.
    Other,
}

fn classify_tool(name: &str) -> ToolKind {
    match name {
        "bash" | "sh" | "shell" | "execute" | "run" => ToolKind::Shell,
        "read_file" | "write_file" | "edit_file" | "create_file" | "delete_file"
        | "list_directory" | "read" | "write" | "edit" | "create" | "delete" | "ls" | "cat"
        | "mkdir" | "mv" | "cp" | "touch" => ToolKind::File,
        "web_search" | "web_fetch" | "web_reader" | "fetch" | "curl" | "search" => ToolKind::Web,
        _ => ToolKind::Other,
    }
}

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
        ToolStatus::Pending | ToolStatus::Running => {
            let blink_on = (tick / 6).is_multiple_of(2);
            let dot = if blink_on {
                TOOL_CIRCLE
            } else {
                " "
            };
            (dot.to_string(), theme.subtle, theme.tool.add_modifier(Modifier::BOLD))
        }
        ToolStatus::Done => {
            (TOOL_CIRCLE.to_string(), theme.success, theme.tool.add_modifier(Modifier::BOLD))
        }
        ToolStatus::Errored => {
            (TOOL_CIRCLE.to_string(), theme.error, theme.tool.add_modifier(Modifier::BOLD))
        }
    };

    let kind = classify_tool(name);
    let prefix = match kind {
        ToolKind::Shell => "$ ",
        _ => "",
    };

    let preview_style = match kind {
        ToolKind::Shell => theme.inactive.add_modifier(Modifier::ITALIC),
        ToolKind::File => theme.claude,
        ToolKind::Web => theme.inactive.add_modifier(Modifier::UNDERLINED),
        ToolKind::Other => theme.inactive,
    };

    lines.push(Line::from(vec![
        Span::styled(format!("{dot_text} "), dot_style),
        Span::styled(display_name, name_style),
        if !prefix.is_empty() {
            Span::styled(prefix.to_string(), theme.subtle)
        } else {
            Span::raw("")
        },
    ]));

    if !input_preview.is_empty() {
        for line in input_preview.lines() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(line.to_string(), preview_style),
            ]));
        }
    }
}

/// Maximum lines shown before collapsing tool output.
const COLLAPSE_THRESHOLD: usize = 5;

/// Render a tool result block using CC's `MessageResponse` (`⎿`) prefix.
///
/// - Error results use `theme.error` for text
/// - Success results use `theme.inactive` for text
/// - Empty output is silently skipped
pub(super) fn render_tool_result_block<'a>(
    output: &'a str,
    is_error: bool,
    collapsed: bool,
    theme: &'a Theme,
    lines: &mut Vec<Line<'a>>,
) {
    let text_style = if is_error {
        theme.error
    } else {
        theme.inactive
    };
    let prefix_style = theme.inactive;

    if output.is_empty() {
        return;
    }

    let all_lines: Vec<&str> = output.lines().collect();
    let line_count = all_lines.len();
    let should_collapse = line_count > COLLAPSE_THRESHOLD;

    if should_collapse && collapsed {
        for (i, line) in all_lines.iter().enumerate() {
            if i >= COLLAPSE_THRESHOLD {
                break;
            }
            if i == 0 {
                lines.push(Line::from(vec![
                    Span::styled("⎿ ", prefix_style),
                    Span::styled(line.to_string(), text_style),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(line.to_string(), text_style),
                ]));
            }
        }
        let hidden = line_count - COLLAPSE_THRESHOLD;
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("... {hidden} more lines (Tab to expand)"), theme.suggestion),
        ]));
    } else {
        let mut first = true;
        for line in &all_lines {
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
        if should_collapse {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("(Tab to collapse)", theme.suggestion),
            ]));
        }
    }
}
