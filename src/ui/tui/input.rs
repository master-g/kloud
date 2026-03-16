//! Keyboard event handling for the TUI.
//!
//! Maps crossterm key events to input buffer edits and [`UiAction`]s.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use super::state::TuiState;
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

	match (*code, *modifiers) {
		// --- Exit ---
		(KeyCode::Char('c'), KeyModifiers::CONTROL) => {
			state.should_quit = true;
			Some(UiAction::Exit)
		}
		(KeyCode::Char('d'), KeyModifiers::CONTROL) if state.input.is_empty() => {
			state.should_quit = true;
			Some(UiAction::Exit)
		}

		// --- Submit ---
		(KeyCode::Enter, KeyModifiers::NONE) => {
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
		}

		// --- Text editing ---
		(KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
			state.input.insert(state.cursor, c);
			state.cursor += c.len_utf8();
			None
		}
		(KeyCode::Backspace, _) => {
			if state.cursor > 0 {
				// Find the previous char boundary
				let prev = state.input[..state.cursor]
					.char_indices()
					.next_back()
					.map(|(i, _)| i)
					.unwrap_or(0);
				state.input.drain(prev..state.cursor);
				state.cursor = prev;
			}
			None
		}
		(KeyCode::Delete, _) => {
			if state.cursor < state.input.len() {
				let next = state.input[state.cursor..]
					.char_indices()
					.nth(1)
					.map(|(i, _)| state.cursor + i)
					.unwrap_or(state.input.len());
				state.input.drain(state.cursor..next);
			}
			None
		}

		// --- Cursor movement ---
		(KeyCode::Left, KeyModifiers::NONE) => {
			if state.cursor > 0 {
				state.cursor = state.input[..state.cursor]
					.char_indices()
					.next_back()
					.map(|(i, _)| i)
					.unwrap_or(0);
			}
			None
		}
		(KeyCode::Right, KeyModifiers::NONE) => {
			if state.cursor < state.input.len() {
				state.cursor = state.input[state.cursor..]
					.char_indices()
					.nth(1)
					.map(|(i, _)| state.cursor + i)
					.unwrap_or(state.input.len());
			}
			None
		}
		(KeyCode::Home, _) | (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
			state.cursor = 0;
			None
		}
		(KeyCode::End, _) | (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
			state.cursor = state.input.len();
			None
		}

		// --- Scroll ---
		(KeyCode::Up, KeyModifiers::NONE) => {
			state.scroll = state.scroll.saturating_sub(1);
			None
		}
		(KeyCode::Down, KeyModifiers::NONE) => {
			state.scroll = state.scroll.saturating_add(1);
			None
		}
		(KeyCode::PageUp, _) => {
			state.scroll = state.scroll.saturating_sub(10);
			None
		}
		(KeyCode::PageDown, _) => {
			state.scroll = state.scroll.saturating_add(10);
			None
		}

		// --- Kill line ---
		(KeyCode::Char('u'), KeyModifiers::CONTROL) => {
			state.input.drain(..state.cursor);
			state.cursor = 0;
			None
		}
		(KeyCode::Char('k'), KeyModifiers::CONTROL) => {
			state.input.truncate(state.cursor);
			None
		}

		_ => None,
	}
}
