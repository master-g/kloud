//! Scroll state: virtual scroll, auto-scroll, offset tracking.
#![allow(missing_docs)]

use crate::ui::tui::virtual_scroll::VirtualScroll;

/// Grouped scroll state.
pub struct ScrollState {
    /// Line-level scroll offset for the Paragraph widget.
    pub offset: usize,
    /// Whether auto-scroll is paused (user scrolled up).
    pub auto_scroll_paused: bool,
    /// Flag set by `jump_to_bottom()`, consumed by the render loop.
    jump_to_bottom: bool,
    /// Virtual scroll state for efficient rendering of large message lists.
    pub virtual_scroll: VirtualScroll,
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            offset: 0,
            auto_scroll_paused: false,
            jump_to_bottom: false,
            virtual_scroll: VirtualScroll::new(),
        }
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.offset = self.offset.saturating_sub(n);
        self.auto_scroll_paused = true;
    }

    pub fn scroll_down(&mut self, n: usize, content_height: usize, visible_height: usize) {
        self.offset = self.offset.saturating_add(n);
        let max_scroll = content_height.saturating_sub(visible_height);
        if self.offset >= max_scroll {
            self.offset = max_scroll;
            self.auto_scroll_paused = false;
        }
    }

    pub fn scroll_down_by(&mut self, n: usize) {
        self.offset = self.offset.saturating_add(n);
    }

    pub fn jump_to_bottom(&mut self) {
        self.jump_to_bottom = true;
    }

    pub fn reset(&mut self) {
        self.offset = 0;
        self.auto_scroll_paused = false;
        self.jump_to_bottom = false;
    }

    pub fn offset_u16(&self) -> u16 {
        self.offset.min(u16::MAX as usize) as u16
    }

    pub fn should_jump_to_bottom(&self) -> bool {
        self.jump_to_bottom
    }

    pub fn clear_jump_flag(&mut self) {
        self.jump_to_bottom = false;
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}
