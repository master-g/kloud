//! TUI render functions.
//!
//! Draws the terminal workbench: title bar, main area, input bar, status bar.

mod activity_line;
mod diff;
mod glimmer;
mod helpers;
mod highlight;
mod layout;
mod messages;
mod permission;
mod spinner_glyph;
mod tool_block;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

use crate::ui::tui::state::{Screen, TuiState};
use crate::ui::tui::theme::Theme;

/// Render the full TUI layout into the given frame.
pub fn render(frame: &mut Frame, state: &mut TuiState, show_title_bar: bool) {
    let theme = state.theme.inner;
    let theme = &theme;

    if state.app.screen == Screen::Transcript {
        render_transcript(frame, state, theme, show_title_bar);
        return;
    }

    // Build dynamic constraint list
    let mut constraints: Vec<Constraint> = Vec::new();
    let mut title_idx: Option<usize> = None;
    let mut notification_idx: Option<usize> = None;

    if show_title_bar {
        title_idx = Some(constraints.len());
        constraints.push(Constraint::Length(1)); // title bar
    }

    let messages_idx = constraints.len();
    constraints.push(Constraint::Min(1)); // messages (+activity line)

    let has_notifications = !state.app.notifications.is_empty();
    if has_notifications {
        notification_idx = Some(constraints.len());
        constraints.push(Constraint::Length(1)); // notification row
    }

    let input_idx = constraints.len();
    let input_height = state.input_height(frame.area().height);
    constraints.push(Constraint::Length(input_height)); // input

    let status_idx = constraints.len();
    constraints.push(Constraint::Length(1)); // status bar

    let chunks = Layout::vertical(constraints).split(frame.area());

    // Render
    if let Some(idx) = title_idx {
        layout::render_title(frame, state, theme, chunks[idx]);
    }
    messages::render_messages(frame, state, theme, chunks[messages_idx]);

    // Notification row (conditional)
    if let Some(idx) = notification_idx {
        layout::render_toasts(frame, state, theme, chunks[idx]);
    }

    layout::render_input(frame, state, theme, chunks[input_idx]);

    // Autocomplete overlay above input area (absolute positioning, 0 layout rows)
    if state.input.autocomplete.visible {
        layout::render_autocomplete(frame, state, theme, chunks[input_idx]);
    }

    layout::render_status(frame, state, theme, chunks[status_idx]);
}

fn render_transcript(frame: &mut Frame, state: &mut TuiState, theme: &Theme, show_title_bar: bool) {
    let chunks = if show_title_bar {
        Layout::vertical([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
            .split(frame.area())
    } else {
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(frame.area())
    };

    let messages_idx = if show_title_bar {
        1
    } else {
        0
    };
    let footer_idx = messages_idx + 1;

    if show_title_bar {
        layout::render_title(frame, state, theme, chunks[0]);
    }
    messages::render_messages(frame, state, theme, chunks[messages_idx]);
    layout::render_transcript_footer(frame, state, theme, chunks[footer_idx]);
}
