//! Multi-line text input widget.
//!
//! Supports multi-line editing, history navigation, and basic Vim mode.

#![allow(missing_docs)]

/// Input mode for the text area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Insert,
    Normal,
}

/// Multi-line text input with cursor tracking and history.
#[derive(Debug, Clone)]
pub struct TextArea {
    /// Lines of text (always >= 1).
    lines: Vec<String>,
    /// Cursor position: (row, column).
    cursor: (usize, usize),
    /// Input history (submitted texts).
    history: Vec<String>,
    /// Current position in history, None = not browsing.
    history_index: Option<usize>,
    /// Saved input before history navigation began.
    saved_input: Option<String>,
    /// Current input mode (Insert or Normal).
    mode: InputMode,
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: (0, 0),
            history: Vec::new(),
            history_index: None,
            saved_input: None,
            mode: InputMode::Insert,
        }
    }

    pub fn mode(&self) -> InputMode {
        self.mode
    }

    // ========================================================================
    // Read accessors
    // ========================================================================

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    pub fn is_single_line(&self) -> bool {
        self.lines.len() == 1
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    pub fn history_index(&self) -> Option<usize> {
        self.history_index
    }

    /// Return the full text with lines joined by newline.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Get a specific line.
    pub fn line(&self, row: usize) -> &str {
        self.lines.get(row).map_or("", |s| s.as_str())
    }

    /// Whether the cursor is on the first row and first column.
    pub fn is_at_start(&self) -> bool {
        self.cursor == (0, 0)
    }

    /// Whether the cursor is at the very end of all text.
    pub fn is_at_end(&self) -> bool {
        let (row, col) = self.cursor;
        row == self.lines.len() - 1 && col == self.lines[row].len()
    }

    // ========================================================================
    // Editing (Insert mode)
    // ========================================================================

    /// Insert a character at the current cursor position.
    pub fn insert_char(&mut self, c: char) {
        let (row, col) = self.cursor;
        self.lines[row].insert(col, c);
        self.cursor.1 += c.len_utf8();
        self.exit_history();
    }

    /// Insert a newline at the current cursor position.
    pub fn insert_newline(&mut self) {
        let (row, col) = self.cursor;
        let rest = self.lines[row].split_off(col);
        self.lines.insert(row + 1, rest);
        self.cursor = (row + 1, 0);
        self.exit_history();
    }

    /// Delete the character before the cursor (Backspace).
    pub fn backspace(&mut self) {
        let (row, col) = self.cursor;
        if col > 0 {
            let prev =
                self.lines[row][..col].char_indices().next_back().map(|(i, _)| i).unwrap_or(0);
            self.lines[row].drain(prev..col);
            self.cursor.1 = prev;
        } else if row > 0 {
            let prev_len = self.lines[row - 1].len();
            let current = self.lines.remove(row);
            self.lines[row - 1].push_str(&current);
            self.cursor = (row - 1, prev_len);
        }
        self.exit_history();
    }

    /// Delete the character at the cursor (Delete).
    pub fn delete(&mut self) {
        let (row, col) = self.cursor;
        if col < self.lines[row].len() {
            let next = self.lines[row][col..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| col + i)
                .unwrap_or(self.lines[row].len());
            self.lines[row].drain(col..next);
        } else if row + 1 < self.lines.len() {
            let next_line = self.lines.remove(row + 1);
            self.lines[row].push_str(&next_line);
        }
        self.exit_history();
    }

    /// Kill from cursor to beginning of line.
    pub fn kill_to_beginning(&mut self) {
        let (row, col) = self.cursor;
        self.lines[row].drain(..col);
        self.cursor.1 = 0;
        self.exit_history();
    }

    /// Kill from cursor to end of line.
    pub fn kill_to_end(&mut self) {
        let (_, col) = self.cursor;
        self.lines[self.cursor.0].truncate(col);
        self.exit_history();
    }

    // ========================================================================
    // Cursor movement
    // ========================================================================

    pub fn move_left(&mut self) {
        let (row, col) = self.cursor;
        if col > 0 {
            self.cursor.1 =
                self.lines[row][..col].char_indices().next_back().map(|(i, _)| i).unwrap_or(0);
        } else if row > 0 {
            self.cursor = (row - 1, self.lines[row - 1].len());
        }
    }

    pub fn move_right(&mut self) {
        let (row, col) = self.cursor;
        if col < self.lines[row].len() {
            self.cursor.1 = self.lines[row][col..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| col + i)
                .unwrap_or(self.lines[row].len());
        } else if row + 1 < self.lines.len() {
            self.cursor = (row + 1, 0);
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor.0 > 0 {
            let new_row = self.cursor.0 - 1;
            let col = self.cursor.1.min(self.lines[new_row].len());
            self.cursor = (new_row, col);
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor.0 + 1 < self.lines.len() {
            let new_row = self.cursor.0 + 1;
            let col = self.cursor.1.min(self.lines[new_row].len());
            self.cursor = (new_row, col);
        }
    }

    pub fn move_home(&mut self) {
        self.cursor.1 = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor.1 = self.lines[self.cursor.0].len();
    }

    // ========================================================================
    // History navigation
    // ========================================================================

    /// Navigate to the previous (older) history entry.
    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        if self.history_index.is_none() {
            self.saved_input = Some(self.text());
            self.history_index = Some(self.history.len() - 1);
        } else if let Some(i) = self.history_index
            && i > 0
        {
            self.history_index = Some(i - 1);
        }
        self.load_from_history();
    }

    /// Navigate to the next (newer) history entry.
    pub fn history_next(&mut self) {
        match self.history_index {
            None => {}
            Some(i) if i + 1 < self.history.len() => {
                self.history_index = Some(i + 1);
                self.load_from_history();
            }
            Some(_) => {
                self.history_index = None;
                if let Some(saved) = self.saved_input.take() {
                    self.set_text(&saved);
                }
            }
        }
    }

    fn load_from_history(&mut self) {
        if let Some(i) = self.history_index {
            let text = self.history[i].clone();
            self.set_text(&text);
        }
    }

    fn exit_history(&mut self) {
        self.history_index = None;
        self.saved_input = None;
    }

    // ========================================================================
    // Submit / clear
    // ========================================================================

    /// Take the current text for submission, pushing it to history.
    /// Returns None if empty.
    pub fn take(&mut self) -> Option<String> {
        let text = self.text();
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            return None;
        }
        self.history.push(trimmed);
        self.clear();
        Some(text)
    }

    /// Clear the text area completely.
    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor = (0, 0);
        self.history_index = None;
        self.saved_input = None;
    }

    /// Set the text area content from a string.
    pub fn set_text(&mut self, text: &str) {
        self.lines = text.lines().map(String::from).collect();
        if self.lines.is_empty() {
            self.lines = vec![String::new()];
        }
        // Move cursor to end
        let last_row = self.lines.len() - 1;
        self.cursor = (last_row, self.lines[last_row].len());
    }

    // ========================================================================
    // Vim mode
    // ========================================================================

    /// Switch to Normal mode.
    pub fn enter_normal_mode(&mut self) {
        self.mode = InputMode::Normal;
    }

    /// Switch to Insert mode.
    pub fn enter_insert_mode(&mut self) {
        self.mode = InputMode::Insert;
    }

    /// Vim: delete the current line (dd).
    pub fn vim_delete_line(&mut self) {
        if self.lines.len() == 1 {
            self.lines[0].clear();
            self.cursor = (0, 0);
        } else {
            self.lines.remove(self.cursor.0);
            if self.cursor.0 >= self.lines.len() {
                self.cursor.0 = self.lines.len() - 1;
            }
            self.cursor.1 = 0;
        }
        self.exit_history();
    }

    /// Vim: delete character under cursor (x).
    pub fn vim_delete_char(&mut self) {
        self.delete();
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line_insert_and_backspace() {
        let mut ta = TextArea::new();
        ta.insert_char('h');
        ta.insert_char('i');
        assert_eq!(ta.text(), "hi");
        ta.backspace();
        assert_eq!(ta.text(), "h");
        assert_eq!(ta.cursor(), (0, 1));
    }

    #[test]
    fn multi_line_insert() {
        let mut ta = TextArea::new();
        ta.insert_char('a');
        ta.insert_newline();
        ta.insert_char('b');
        assert_eq!(ta.text(), "a\nb");
        assert_eq!(ta.cursor(), (1, 1));
        assert!(!ta.is_single_line());
    }

    #[test]
    fn backspace_merges_lines() {
        let mut ta = TextArea::new();
        ta.insert_char('a');
        ta.insert_newline();
        // cursor at (1, 0)
        ta.backspace(); // delete newline
        assert_eq!(ta.text(), "a");
        assert_eq!(ta.cursor(), (0, 1));
    }

    #[test]
    fn cursor_movement() {
        let mut ta = TextArea::new();
        ta.set_text("hello\nworld");
        ta.cursor = (0, 5);
        ta.move_right(); // wrap to next line
        assert_eq!(ta.cursor(), (1, 0));
        ta.move_left(); // wrap back
        assert_eq!(ta.cursor(), (0, 5));
        ta.move_home();
        assert_eq!(ta.cursor(), (0, 0));
        ta.move_end();
        assert_eq!(ta.cursor(), (0, 5));
    }

    #[test]
    fn take_submits_and_clears() {
        let mut ta = TextArea::new();
        ta.set_text("hello");
        let text = ta.take();
        assert_eq!(text.as_deref(), Some("hello"));
        assert!(ta.is_empty());
        assert_eq!(ta.history().len(), 1);
    }

    #[test]
    fn take_rejects_empty() {
        let mut ta = TextArea::new();
        assert!(ta.take().is_none());
    }

    #[test]
    fn history_navigation() {
        let mut ta = TextArea::new();
        ta.set_text("first");
        ta.take();
        ta.set_text("second");
        ta.take();

        assert!(ta.is_empty());
        ta.history_prev();
        assert_eq!(ta.text(), "second");
        ta.history_prev();
        assert_eq!(ta.text(), "first");
        ta.history_next();
        assert_eq!(ta.text(), "second");
        ta.history_next();
        assert!(ta.is_empty()); // restored saved (empty)
    }

    #[test]
    fn kill_line() {
        let mut ta = TextArea::new();
        ta.set_text("hello");
        ta.cursor = (0, 3);
        ta.kill_to_end();
        assert_eq!(ta.text(), "hel");
        ta.cursor = (0, 2);
        ta.kill_to_beginning();
        assert_eq!(ta.text(), "l");
    }

    #[test]
    fn vim_mode_toggle() {
        let mut ta = TextArea::new();
        assert_eq!(ta.mode(), InputMode::Insert);
        ta.enter_normal_mode();
        assert_eq!(ta.mode(), InputMode::Normal);
        ta.enter_insert_mode();
        assert_eq!(ta.mode(), InputMode::Insert);
    }

    #[test]
    fn vim_delete_line() {
        let mut ta = TextArea::new();
        ta.set_text("line1\nline2\nline3");
        ta.cursor = (1, 0);
        ta.vim_delete_line();
        assert_eq!(ta.text(), "line1\nline3");
    }

    #[test]
    fn vim_delete_char() {
        let mut ta = TextArea::new();
        ta.set_text("abc");
        ta.cursor = (0, 1);
        ta.vim_delete_char();
        assert_eq!(ta.text(), "ac");
    }
}
