//! TUI render functions.
//!
//! Draws the terminal workbench: title bar, main area, input bar, status bar.

mod activity_line;
mod diff;
mod glimmer;
mod helpers;
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
pub fn render(frame: &mut Frame, state: &mut TuiState, theme: &Theme, show_title_bar: bool) {
    if state.screen == Screen::Transcript {
        render_transcript(frame, state, theme, show_title_bar);
        return;
    }

    let chunks = if show_title_bar {
        Layout::vertical([
            Constraint::Length(1), // title bar
            Constraint::Min(1),    // messages (+activity line)
            Constraint::Length(3), // input
            Constraint::Length(1), // status bar
        ])
        .split(frame.area())
    } else {
        Layout::vertical([
            Constraint::Min(1),    // messages (+activity line)
            Constraint::Length(3), // input
            Constraint::Length(1), // status bar
        ])
        .split(frame.area())
    };

    let messages_idx = if show_title_bar {
        1
    } else {
        0
    };
    let input_idx = messages_idx + 1;
    let status_idx = input_idx + 1;

    if show_title_bar {
        layout::render_title(frame, state, theme, chunks[0]);
    }
    messages::render_messages(frame, state, theme, chunks[messages_idx]);
    layout::render_input(frame, state, theme, chunks[input_idx]);
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
