//! Virtual scrolling for the message list.
//!
//! Only renders messages within the visible viewport plus a buffer,
//! avoiding O(n) rendering for large conversations.

#![allow(missing_docs)]

use crate::ui::tui::state::DisplayMessage;

/// Buffer of extra messages rendered above/below viewport.
const VIEWPORT_BUFFER: usize = 5;

/// Virtual scroll state.
#[derive(Debug, Clone)]
pub struct VirtualScroll {
    /// Cached line heights per message (invalidated on resize).
    line_heights: Vec<usize>,
    /// Whether the cache is valid.
    cache_valid: bool,
    /// Last known terminal width (for cache invalidation).
    last_width: u16,
    /// Whether auto-scroll is enabled (user at bottom).
    auto_scroll: bool,
    /// Accumulated scroll offset in lines.
    scroll_offset: usize,
}

impl VirtualScroll {
    pub fn new() -> Self {
        Self {
            line_heights: Vec::new(),
            cache_valid: false,
            last_width: 0,
            auto_scroll: true,
            scroll_offset: 0,
        }
    }

    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Invalidate the line height cache (call on resize).
    pub fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }

    /// Update terminal width and invalidate cache if changed.
    pub fn update_width(&mut self, width: u16) {
        if width != self.last_width {
            self.last_width = width;
            self.cache_valid = false;
        }
    }

    /// Scroll up by `n` lines. Disables auto-scroll.
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
        self.auto_scroll = false;
    }

    /// Scroll down by `n` lines. May re-enable auto-scroll.
    pub fn scroll_down(&mut self, total_lines: usize, visible_lines: usize, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(n);
        let max_scroll = total_lines.saturating_sub(visible_lines);
        if self.scroll_offset >= max_scroll {
            self.scroll_offset = max_scroll;
            self.auto_scroll = true;
        }
    }

    /// Jump to the bottom, re-enabling auto-scroll.
    pub fn scroll_to_bottom(&mut self, total_lines: usize, visible_lines: u16) {
        let max_scroll = total_lines.saturating_sub(visible_lines as usize);
        self.scroll_offset = max_scroll;
        self.auto_scroll = true;
    }

    /// Jump to the top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
        self.auto_scroll = false;
    }

    /// Scroll half a page up.
    pub fn half_page_up(&mut self, visible_lines: u16) {
        self.scroll_up((visible_lines / 2) as usize);
    }

    /// Scroll half a page down.
    pub fn half_page_down(&mut self, total_lines: usize, visible_lines: u16) {
        self.scroll_down(total_lines, visible_lines as usize, (visible_lines / 2) as usize);
    }

    /// Compute the visible range (start..end) of line indices.
    /// If `auto_scroll` is enabled, snaps to bottom.
    pub fn visible_line_range(&self, total_lines: usize, visible_lines: u16) -> (usize, usize) {
        let visible = visible_lines as usize;
        if total_lines <= visible {
            return (0, total_lines);
        }

        let offset = if self.auto_scroll {
            total_lines.saturating_sub(visible)
        } else {
            self.scroll_offset.min(total_lines.saturating_sub(visible))
        };

        let end = (offset + visible).min(total_lines);
        (offset, end)
    }

    /// Estimate line heights for messages (rough: 1 line per block + 1 per message).
    /// Real implementation would measure wrapped text, but this is a fast approximation.
    pub fn ensure_cache(&mut self, messages: &[DisplayMessage], width: u16) {
        self.update_width(width);
        if self.cache_valid && self.line_heights.len() == messages.len() {
            return;
        }
        self.line_heights =
            messages.iter().map(|msg| estimate_message_height(msg, width)).collect();
        self.cache_valid = true;
    }

    /// Get the total number of lines across all messages.
    pub fn total_lines(&self) -> usize {
        self.line_heights.iter().sum()
    }

    /// Compute the visible message range and line offset within the first visible message.
    /// Returns (`first_msg_idx`, `line_offset_in_first_msg`, `last_msg_idx`).
    pub fn visible_message_range(
        &self,
        visible_lines: u16,
        extra_lines: usize,
    ) -> (usize, usize, usize) {
        let total = self.total_lines();
        let (start_line, end_line) = self.visible_line_range(total + extra_lines, visible_lines);

        let mut acc = 0;
        let mut first_idx = 0;
        let mut offset_in_first = 0;
        for (i, &h) in self.line_heights.iter().enumerate() {
            if acc + h > start_line {
                first_idx = i;
                offset_in_first = start_line.saturating_sub(acc);
                break;
            }
            acc += h;
        }

        let mut last_idx = first_idx;
        let mut acc2 = acc;
        for (i, &h) in self.line_heights[first_idx..].iter().enumerate() {
            if acc2 + h >= end_line {
                last_idx = first_idx + i;
                break;
            }
            acc2 += h;
            last_idx = first_idx + i;
        }

        // Add buffer
        let buffered_first = first_idx.saturating_sub(VIEWPORT_BUFFER);
        let buffered_last =
            (last_idx + VIEWPORT_BUFFER).min(self.line_heights.len().saturating_sub(1));

        (buffered_first, offset_in_first, buffered_last)
    }
}

impl Default for VirtualScroll {
    fn default() -> Self {
        Self::new()
    }
}

/// Rough line height estimate for a message.
fn estimate_message_height(msg: &DisplayMessage, width: u16) -> usize {
    let width = width.max(1) as usize;
    let mut lines = 1; // spacing line
    for block in &msg.blocks {
        lines += match block {
            crate::ui::tui::state::DisplayBlock::Text(t)
            | crate::ui::tui::state::DisplayBlock::Thinking(t)
            | crate::ui::tui::state::DisplayBlock::RedactedThinking(t) => {
                let content_lines = t.lines().count().max(1);

                t.len().div_ceil(width).max(content_lines)
            }
            crate::ui::tui::state::DisplayBlock::ToolUse {
                input_preview,
                ..
            } => {
                let header = 1;
                let preview = if input_preview.is_empty() {
                    0
                } else {
                    input_preview.lines().count()
                };
                header + preview
            }
            crate::ui::tui::state::DisplayBlock::ToolResult {
                output,
                ..
            } => {
                if output.is_empty() {
                    0
                } else {
                    output.lines().count().max(1)
                }
            }
        };
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_range_small_content() {
        let vs = VirtualScroll::new();
        let (start, end) = vs.visible_line_range(5, 10);
        assert_eq!(start, 0);
        assert_eq!(end, 5);
    }

    #[test]
    fn auto_scroll_snaps_to_bottom() {
        let vs = VirtualScroll::new();
        assert!(vs.auto_scroll());
        let (start, end) = vs.visible_line_range(100, 10);
        assert_eq!(start, 90);
        assert_eq!(end, 100);
    }

    #[test]
    fn scroll_up_disables_auto() {
        let mut vs = VirtualScroll::new();
        vs.scroll_up(5);
        assert!(!vs.auto_scroll());
        assert_eq!(vs.scroll_offset(), 0);
    }

    #[test]
    fn scroll_to_bottom_reenables_auto() {
        let mut vs = VirtualScroll::new();
        vs.scroll_up(5);
        assert!(!vs.auto_scroll());
        vs.scroll_to_bottom(100, 10);
        assert!(vs.auto_scroll());
    }
}

#[cfg(test)]
mod perf_tests {
    use super::*;
    use crate::ui::tui::state::{DisplayBlock, DisplayMessage, MessageType};

    fn make_messages(n: usize) -> Vec<DisplayMessage> {
        (0..n)
			.map(|i| DisplayMessage {
				id: i as u64,
				message_type: if i % 2 == 0 {
					MessageType::User
				} else {
					MessageType::Assistant
				},
				blocks: vec![DisplayBlock::Text(format!(
					"Message {i}: this is a moderately long message with some text content that wraps."
				))],
			})
			.collect()
    }

    #[test]
    fn render_1000_messages_under_16ms() {
        let messages = make_messages(1000);
        let mut vs = VirtualScroll::new();
        vs.ensure_cache(&messages, 80);

        let start = std::time::Instant::now();
        for _ in 0..100 {
            let (start_idx, _offset, end_idx) = vs.visible_message_range(50, 0);
            let _visible: Vec<_> = messages
                .iter()
                .skip(start_idx)
                .take(end_idx.saturating_sub(start_idx) + 1)
                .collect();
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 16,
            "100 iterations of visible_message_range took {:?} (>16ms)",
            elapsed,
        );
    }
}
