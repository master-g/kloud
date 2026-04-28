//! Keyboard event handling for the TUI.
//!
//! Maps crossterm key events to `TextArea` edits and [`UiAction`]s.
//! Supports multi-line editing, history navigation, and basic Vim mode.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::state::{AssistantStatus, Screen, TuiState};
use super::text_area::InputMode;
use crate::ui::events::UiAction;

/// Process a crossterm event against the current TUI state.
///
/// Returns `Some(UiAction)` when the event should be forwarded to the session
/// (e.g. message submission, exit). Returns `None` for events handled purely
/// within the UI (cursor movement, character insertion).
pub fn handle_event(event: &Event, state: &mut TuiState) -> Option<UiAction> {
    let Event::Key(KeyEvent {
        code,
        modifiers,
        ..
    }) = event
    else {
        return None;
    };

    // Permission prompt takes priority when pending and idle.
    if state.app.pending_permission.is_some() && matches!(state.status, AssistantStatus::Idle) {
        match (*code, *modifiers) {
            (KeyCode::Char('y'), KeyModifiers::NONE) => {
                return Some(UiAction::PermissionResponse {
                    allowed: true,
                });
            }
            (KeyCode::Char('n'), KeyModifiers::NONE) | (KeyCode::Esc, KeyModifiers::NONE) => {
                return Some(UiAction::PermissionResponse {
                    allowed: false,
                });
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                state.should_quit = true;
                return Some(UiAction::Exit);
            }
            _ => return None,
        }
    }

    if state.app.screen == Screen::Transcript {
        return handle_transcript_event(*code, *modifiers, state);
    }

    if state.app.screen == Screen::Search {
        return handle_search_event(*code, *modifiers, state);
    }

    match (*code, *modifiers) {
        (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
            Some(UiAction::SetScreen(Screen::Transcript))
        }
        // --- Search activation (Ctrl+S) ---
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => Some(UiAction::SearchActivate),

        // --- Autocomplete (Tab) ---
        (KeyCode::Tab, KeyModifiers::NONE) if state.input.autocomplete.visible => {
            if let Some(name) = state.input.autocomplete.completion() {
                state.input.text_area.set_text(&format!("/{name} "));
            }
            state.input.autocomplete.visible = false;
            None
        }

        // --- Toggle tool output collapse (Tab) ---
        (KeyCode::Tab, KeyModifiers::NONE) => {
            if state.app.collapsed_tools.is_empty() {
                // Auto-populate: find the nearest tool result with >5 lines and collapse it.
                const COLLAPSE_THRESHOLD: usize = 5;
                for (i, msg) in state.app.messages.iter().enumerate().rev() {
                    for block in &msg.blocks {
                        if let crate::ui::tui::state::DisplayBlock::ToolResult {
                            output,
                            ..
                        } = block
                            && output.lines().count() > COLLAPSE_THRESHOLD
                        {
                            state.app.collapsed_tools.insert(i, true);
                            return None;
                        }
                    }
                }
                return None;
            }
            // Toggle the most recent collapsed block
            if let Some(&last) = state.app.collapsed_tools.keys().last() {
                return Some(UiAction::ToggleToolCollapse(last));
            }
            None
        }

        // --- Exit / cancel ---
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            if matches!(state.status, AssistantStatus::Streaming | AssistantStatus::Cancelling) {
                state.begin_cancel();
                Some(UiAction::CancelTurn)
            } else {
                state.should_quit = true;
                Some(UiAction::Exit)
            }
        }
        (KeyCode::Char('d'), KeyModifiers::CONTROL) if state.input.text_area.is_empty() => {
            state.should_quit = true;
            Some(UiAction::Exit)
        }

        // --- Submit (Enter on single-line, empty) ---
        (KeyCode::Enter, KeyModifiers::NONE) => {
            if state.input.text_area.is_single_line() {
                if let Some(text) = state.take_input() {
                    if let Some(rest) = text.strip_prefix('/') {
                        let mut parts = rest.splitn(2, ' ');
                        let command = parts.next().unwrap_or("").to_string();
                        let args = parts.next().unwrap_or("").to_string();
                        Some(UiAction::SlashCommand {
                            command,
                            args,
                        })
                    } else {
                        Some(UiAction::SendMessage(text))
                    }
                } else {
                    None
                }
            } else {
                // Multi-line: Enter inserts newline
                state.input.text_area.insert_newline();
                None
            }
        }

        // --- Shift+Enter / Alt+Enter: always insert newline ---
        (KeyCode::Enter, KeyModifiers::SHIFT) | (KeyCode::Enter, KeyModifiers::ALT) => {
            state.input.text_area.insert_newline();
            None
        }

        // --- Vim mode toggle ---
        (KeyCode::Esc, KeyModifiers::NONE)
            if state.input.text_area.mode() == InputMode::Insert
                && matches!(state.status, AssistantStatus::Idle) =>
        {
            state.input.text_area.enter_normal_mode();
            None
        }

        // --- Normal mode keys (Vim) ---
        _ if state.input.text_area.mode() == InputMode::Normal => handle_normal_mode(*code, state),

        // --- Insert mode: text editing ---
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            state.input.text_area.insert_char(c);
            let text = state.input.text_area.text().to_string();
            if let Some(rest) = text.strip_prefix('/') {
                state.input.autocomplete.update_filter(rest);
                state.input.autocomplete.visible = !state.input.autocomplete.items.is_empty();
            } else {
                state.input.autocomplete.visible = false;
            }
            None
        }
        (KeyCode::Backspace, _) => {
            state.input.text_area.backspace();
            let text = state.input.text_area.text().to_string();
            if let Some(rest) = text.strip_prefix('/') {
                state.input.autocomplete.update_filter(rest);
                state.input.autocomplete.visible = !state.input.autocomplete.items.is_empty();
            } else {
                state.input.autocomplete.visible = false;
            }
            None
        }
        (KeyCode::Down, KeyModifiers::NONE) if state.input.autocomplete.visible => {
            state.input.autocomplete.next();
            None
        }
        (KeyCode::Up, KeyModifiers::NONE) if state.input.autocomplete.visible => {
            state.input.autocomplete.prev();
            None
        }
        (KeyCode::Esc, KeyModifiers::NONE) if state.input.autocomplete.visible => {
            state.input.autocomplete.visible = false;
            None
        }
        (KeyCode::Delete, _) => {
            state.input.text_area.delete();
            None
        }

        // --- Cursor movement ---
        (KeyCode::Left, KeyModifiers::NONE) => {
            state.input.text_area.move_left();
            None
        }
        (KeyCode::Right, KeyModifiers::NONE) => {
            state.input.text_area.move_right();
            None
        }
        (KeyCode::Up, KeyModifiers::NONE) => {
            if state.input.text_area.is_at_start()
                || state.input.text_area.history_index().is_some()
            {
                state.input.text_area.history_prev();
            } else {
                state.input.text_area.move_up();
            }
            None
        }
        (KeyCode::Down, KeyModifiers::NONE) => {
            if state.input.text_area.is_at_end() || state.input.text_area.history_index().is_some()
            {
                state.input.text_area.history_next();
            } else {
                state.input.text_area.move_down();
            }
            None
        }
        (KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
            state.input.text_area.move_home();
            None
        }
        (KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
            state.input.text_area.move_end();
            None
        }

        // --- Scroll ---
        (KeyCode::PageUp, _) => {
            state.scroll_messages_up(10);
            None
        }
        (KeyCode::PageDown, _) => {
            state.scroll_messages_down(10);
            None
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) if state.input.text_area.is_empty() => {
            state.scroll_messages_up(5);
            None
        }

        // --- Kill line ---
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            state.input.text_area.kill_to_beginning();
            None
        }
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
            state.input.text_area.kill_to_end();
            None
        }

        _ => None,
    }
}

/// Handle keys in Vim Normal mode.
fn handle_normal_mode(code: KeyCode, state: &mut TuiState) -> Option<UiAction> {
    match code {
        KeyCode::Char('i') => {
            state.input.text_area.enter_insert_mode();
            None
        }
        KeyCode::Char('h') => {
            state.input.text_area.move_left();
            None
        }
        KeyCode::Char('j') => {
            state.input.text_area.move_down();
            None
        }
        KeyCode::Char('k') => {
            state.input.text_area.move_up();
            None
        }
        KeyCode::Char('l') => {
            state.input.text_area.move_right();
            None
        }
        KeyCode::Char('x') => {
            state.input.text_area.vim_delete_char();
            None
        }
        KeyCode::Char('d') => {
            // dd = delete line (simplified: single 'd' press deletes line)
            state.input.text_area.vim_delete_line();
            None
        }
        _ => None,
    }
}

fn handle_search_event(
    code: KeyCode,
    modifiers: KeyModifiers,
    state: &mut TuiState,
) -> Option<UiAction> {
    match (code, modifiers) {
        (KeyCode::Esc, _) => Some(UiAction::SearchExit),
        (KeyCode::Enter, _) => {
            let q = state.input.search.query.clone();
            state.input.search.execute(&q, &state.app.messages);
            Some(UiAction::SearchSubmit(q))
        }
        (KeyCode::Char('n'), KeyModifiers::NONE) => {
            state.input.search.next();
            Some(UiAction::SearchNext)
        }
        (KeyCode::Char('N'), KeyModifiers::SHIFT) => {
            state.input.search.prev();
            Some(UiAction::SearchPrev)
        }
        (KeyCode::Backspace, _) if state.input.search.query.is_empty() => {
            Some(UiAction::SearchExit)
        }
        (KeyCode::Backspace, _) => {
            state.input.search.query.pop();
            let q = state.input.search.query.clone();
            state.input.search.execute(&q, &state.app.messages);
            None
        }
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            state.input.search.query.push(c);
            let q = state.input.search.query.clone();
            state.input.search.execute(&q, &state.app.messages);
            None
        }
        _ => None,
    }
}

fn handle_transcript_event(
    code: KeyCode,
    modifiers: KeyModifiers,
    state: &mut TuiState,
) -> Option<UiAction> {
    match (code, modifiers) {
        (KeyCode::Char('o'), KeyModifiers::CONTROL)
        | (KeyCode::Esc, _)
        | (KeyCode::Char('q'), KeyModifiers::NONE)
        | (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(UiAction::SetScreen(Screen::Prompt)),
        (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
            Some(UiAction::SetTranscriptShowAll(!state.transcript_show_all))
        }
        (KeyCode::Up, KeyModifiers::NONE) => {
            state.scroll.scroll_up(1);
            None
        }
        (KeyCode::Down, KeyModifiers::NONE) => {
            state.scroll.scroll_down_by(1);
            None
        }
        (KeyCode::PageUp, _) => {
            state.scroll.scroll_up(10);
            None
        }
        (KeyCode::PageDown, _) => {
            state.scroll.scroll_down_by(10);
            None
        }
        (KeyCode::Home, _) => {
            state.scroll.reset();
            None
        }
        (KeyCode::End, _) => {
            state.scroll.jump_to_bottom();
            None
        }
        _ => None,
    }
}
