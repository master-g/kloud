//! Layout widgets: title bar, input bar, status bar.

use std::path::Path;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::llm::response::StopReason;
use crate::ui::tui::constants::CONTEXT_METER_WIDTH;
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

/// Render the input bar with multi-line support.
pub(super) fn render_input(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
    let ta = &state.input.text_area;
    let show_placeholder = matches!(state.status, AssistantStatus::Idle) && ta.is_empty();
    let show_cursor = matches!(state.status, AssistantStatus::Idle);

    let lines: Vec<Line<'_>> = if show_placeholder {
        vec![Line::from(vec![
            Span::raw("❯ "),
            Span::styled(
                "Ask kloud to inspect files, trace state, or use tools...",
                theme.inactive,
            ),
        ])]
    } else if matches!(state.status, AssistantStatus::Streaming) {
        vec![Line::from(Span::styled("(streaming...)", theme.inactive))]
    } else if matches!(state.status, AssistantStatus::Cancelling) {
        vec![Line::from(Span::styled("(cancelling...)", theme.warning))]
    } else {
        let mut result = Vec::with_capacity(ta.line_count());
        for i in 0..ta.line_count() {
            let line_text = ta.line(i).to_string();
            if i == 0 {
                result.push(Line::from(vec![Span::raw("❯ "), Span::styled(line_text, theme.text)]));
            } else {
                result.push(Line::from(vec![Span::raw("  "), Span::styled(line_text, theme.text)]));
            }
        }
        result
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_style(theme.prompt_border),
        )
        .wrap(Wrap {
            trim: false,
        });

    frame.render_widget(paragraph, area);

    if show_cursor && !show_placeholder {
        let (row, col) = ta.cursor();
        let line_text = ta.line(row);
        let col_width = line_text[..col.min(line_text.len())].width() as u16;
        let cursor_x = area.x + 2 + col_width;
        let cursor_y = area.y + 1 + row as u16;
        if cursor_y < area.y + area.height {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

/// Render the status bar.
pub(super) fn render_status(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
    let wide = area.width >= 80;
    let mut spans = vec![
        Span::styled(format!(" {} ", state.model), theme.status_model_badge),
        Span::raw(" │ "),
        Span::styled(format!(" {} ", state.effort), theme.status_effort_badge),
        Span::raw(" │ "),
        Span::styled(format!("branch: {}", state.branch), theme.warning),
        Span::raw(" │ "),
        Span::styled(format!("worktree: {}", state.workspace_name()), theme.subtle),
        Span::raw(" │ "),
        Span::styled(
            format!("tokens: {}↓ {}↑", state.input_tokens, state.output_tokens),
            theme.inactive,
        ),
        Span::raw(" │ "),
        render_context_meter(state, theme),
    ];

    if wide {
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(format!("${:.2}", state.session_cost), theme.success));
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(format_duration(state.session_duration), theme.inactive));
    }

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

    if !state.app.active_tools.is_empty() {
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(
            format!("tools: {}", state.app.active_tools.join(", ")),
            theme.tool,
        ));
    }

    spans.push(Span::raw(" │ "));
    spans.push(Span::styled(
        format!(
            "guides: {} hooks: {} history: {}",
            state.instruction_files.len(),
            state.hook_count,
            state.input.text_area.history().len()
        ),
        theme.claude,
    ));

    if let Some(hint) = state.current_command_hint() {
        spans.push(Span::raw(" │ "));
        spans.push(Span::styled(format!("/{} — {}", hint.name, hint.summary), theme.claude));
    }

    // Context-sensitive hints (width-gated)
    let used_width: usize = spans.iter().map(|s| s.content.as_ref().width()).sum();
    if area.width as usize > used_width
        && let Some(hint_span) =
            hints_span(state, theme, area.width.saturating_sub(used_width as u16))
    {
        spans.push(hint_span);
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

/// Render autocomplete popup below the input area.
pub(super) fn render_autocomplete(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
    if !state.input.autocomplete.visible || state.input.autocomplete.items.is_empty() {
        return;
    }
    let max_visible = 4usize;
    let items = &state.input.autocomplete.items;
    let visible = items.len().min(max_visible);
    let lines: Vec<Line> = items
        .iter()
        .take(max_visible)
        .enumerate()
        .map(|(i, (name, summary))| {
            let is_selected = i == state.input.autocomplete.selected;
            let name_style = if is_selected {
                theme.claude.add_modifier(Modifier::BOLD)
            } else {
                theme.text
            };
            let summary_style = if is_selected {
                theme.subtle
            } else {
                theme.inactive
            };
            Line::from(vec![
                Span::styled(format!("  /{name}"), name_style),
                Span::styled(format!(" \u{2014} {summary}"), summary_style),
            ])
        })
        .collect();

    let popup_height = visible as u16 + 2; // +2 for border
    let popup_area = Rect {
        x: area.x + 2,
        y: area.y.saturating_sub(popup_height),
        width: (area.width.min(40)).min(area.width),
        height: popup_height.min(area.y),
    };

    let paragraph = ratatui::widgets::Paragraph::new(lines)
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(theme.prompt_border),
        )
        .style(ratatui::style::Style::default());
    frame.render_widget(paragraph, popup_area);
}

/// Render notification toasts above the input area.
/// Shows only the first notification (single-queue, matching CC's Notifications.tsx).
pub(super) fn render_toasts(frame: &mut Frame, state: &TuiState, theme: &Theme, area: Rect) {
    if state.app.notifications.is_empty() {
        return;
    }
    let notif = &state.app.notifications[0];
    let style = match notif.level {
        crate::ui::tui::state::MessageLevel::Error => theme.error,
        crate::ui::tui::state::MessageLevel::Warning => theme.warning,
        crate::ui::tui::state::MessageLevel::Info => theme.inactive,
    };
    let line = Line::from(Span::styled(format!(" {} ", notif.text), style));
    frame.render_widget(Paragraph::new(line), area);
}

/// Build a hints Span for embedding in the status bar. Returns None when too narrow.
pub(super) fn hints_span(
    state: &TuiState,
    theme: &Theme,
    available_width: u16,
) -> Option<Span<'static>> {
    if available_width < 40 {
        return None;
    }
    let hint = match state.app.screen {
        crate::agent::view::Screen::Search => "Esc back · / search · n next · N prev",
        _ => match state.status {
            AssistantStatus::Streaming | AssistantStatus::Cancelling => "Ctrl+C cancel",
            AssistantStatus::Idle => {
                if state.input.text_area.mode() == crate::ui::tui::text_area::InputMode::Normal {
                    "NORMAL h/j/k/l move · i insert · x delete · dd delete line"
                } else {
                    "Enter send · Shift+Enter newline · Esc normal · Ctrl+C exit"
                }
            }
        },
    };
    Some(Span::styled(format!(" │ {hint}"), theme.inactive))
}

pub(super) fn render_transcript_footer(
    frame: &mut Frame,
    state: &TuiState,
    theme: &Theme,
    area: Rect,
) {
    let line = Line::from(vec![Span::styled(
        truncate_to_width(&state.transcript_status_text(), area.width as usize),
        theme.inactive,
    )]);
    let paragraph = Paragraph::new(line)
        .block(Block::default().borders(Borders::TOP).border_style(theme.prompt_border));
    frame.render_widget(paragraph, area);
}

pub(super) fn workspace_name(path: &str) -> String {
    Path::new(path).file_name().and_then(|name| name.to_str()).unwrap_or(path).to_string()
}

fn format_duration(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
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
