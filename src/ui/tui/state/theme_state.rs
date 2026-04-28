//! Theme state wrapper for [`TuiState`].

use crate::ui::tui::theme::{ColorScheme, Theme};

/// Theme state held by `TuiState`, wrapping the active `Theme`
/// and providing a convenience method for scheme switching.
pub struct ThemeState {
    pub inner: Theme,
}

impl ThemeState {
    /// Create a new `ThemeState` with the given theme.
    pub fn new(theme: Theme) -> Self {
        Self {
            inner: theme,
        }
    }

    /// Switch the active color scheme, preserving `no_color`.
    pub fn set_scheme(&mut self, scheme: ColorScheme) {
        self.inner = Theme::from_scheme(scheme, self.inner.no_color);
    }
}
