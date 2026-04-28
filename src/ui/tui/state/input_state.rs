//! Input state: text area, history, autocomplete, search.
#![allow(missing_docs)]

use crate::agent::message::{DisplayBlock, DisplayMessage};
use crate::ui::tui::text_area::TextArea;

#[derive(Debug, Clone)]
pub struct CommandHint {
    pub name: &'static str,
    pub summary: &'static str,
}

pub(crate) const SLASH_COMMAND_HINTS: &[CommandHint] = &[
    CommandHint {
        name: "help",
        summary: "Show available commands",
    },
    CommandHint {
        name: "exit",
        summary: "Quit kloud",
    },
    CommandHint {
        name: "quit",
        summary: "Alias for /exit",
    },
];

/// Search state for the message area.
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Current search query.
    pub query: String,
    /// Indices of messages that match the query.
    pub matches: Vec<usize>,
    /// Index into `matches` for the current highlighted match.
    pub current_match: usize,
    /// Whether search has wrapped around.
    pub wrapped: bool,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            matches: Vec::new(),
            current_match: 0,
            wrapped: false,
        }
    }

    /// Execute case-insensitive substring search across messages.
    pub fn execute(&mut self, query: &str, messages: &[DisplayMessage]) {
        self.query = query.to_string();
        if query.is_empty() {
            self.matches.clear();
            self.current_match = 0;
            self.wrapped = false;
            return;
        }
        let lower = query.to_lowercase();
        self.matches = messages
            .iter()
            .enumerate()
            .filter(|(_, msg)| {
                msg.blocks.iter().any(|b| match b {
                    DisplayBlock::Text(t)
                    | DisplayBlock::Thinking(t)
                    | DisplayBlock::RedactedThinking(t) => t.to_lowercase().contains(&lower),
                    DisplayBlock::ToolUse {
                        input_preview,
                        ..
                    } => input_preview.to_lowercase().contains(&lower),
                    DisplayBlock::ToolResult {
                        output,
                        ..
                    } => output.to_lowercase().contains(&lower),
                })
            })
            .map(|(i, _)| i)
            .collect();
        self.current_match = 0;
        self.wrapped = false;
    }

    /// Move to the next match. Sets `wrapped` if wrapping around.
    pub fn next(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.current_match += 1;
        if self.current_match >= self.matches.len() {
            self.current_match = 0;
            self.wrapped = true;
        } else {
            self.wrapped = false;
        }
    }

    /// Move to the previous match. Sets `wrapped` if wrapping around.
    pub fn prev(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        if self.current_match == 0 {
            self.current_match = self.matches.len() - 1;
            self.wrapped = true;
        } else {
            self.current_match -= 1;
            self.wrapped = false;
        }
    }

    /// Return the message index of the current match, if any.
    pub fn current_message_index(&self) -> Option<usize> {
        self.matches.get(self.current_match).copied()
    }

    /// Return match count display string like "[3/12]".
    pub fn match_display(&self) -> String {
        if self.matches.is_empty() {
            "[0/0]".to_string()
        } else {
            format!("[{}/{}]", self.current_match + 1, self.matches.len())
        }
    }
}

/// Autocomplete state for slash commands.
#[derive(Debug, Clone)]
pub struct AutocompleteState {
    pub visible: bool,
    pub items: Vec<(String, String)>,
    pub selected: usize,
    pub filter: String,
}

impl Default for AutocompleteState {
    fn default() -> Self {
        Self {
            visible: false,
            items: SLASH_COMMAND_HINTS
                .iter()
                .map(|h| (h.name.to_string(), h.summary.to_string()))
                .collect(),
            selected: 0,
            filter: String::new(),
        }
    }
}

impl AutocompleteState {
    pub fn update_filter(&mut self, filter: &str) {
        self.filter = filter.to_string();
        self.items = SLASH_COMMAND_HINTS
            .iter()
            .filter(|h| h.name.contains(filter))
            .map(|h| (h.name.to_string(), h.summary.to_string()))
            .collect();
        if self.selected >= self.items.len() {
            self.selected = 0;
        }
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn prev(&mut self) {
        if !self.items.is_empty() {
            self.selected = if self.selected == 0 {
                self.items.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn completion(&self) -> Option<&str> {
        self.items.get(self.selected).map(|(name, _)| name.as_str())
    }
}

/// Grouped input state: text area, history, search, autocomplete.
#[derive(Default)]
pub struct InputState {
    pub text_area: TextArea,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
    pub search: SearchState,
    pub autocomplete: AutocompleteState,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            text_area: TextArea::new(),
            history: Vec::new(),
            history_index: None,
            search: SearchState::new(),
            autocomplete: AutocompleteState::default(),
        }
    }
}
