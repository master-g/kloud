//! Permission prompt renderer.

use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use crate::ui::tui::state::PendingPermissionView;
use crate::ui::tui::theme::Theme;

/// Build permission prompt lines for inline use in the message scroll area.
pub fn render_permission_prompt(theme: &Theme, perm: &PendingPermissionView) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " \u{26a0} Permission Required",
            theme.warning.add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(format!(" {}", perm.label), theme.text_bold)]),
        Line::from(vec![Span::styled(format!(" {}", perm.description), theme.subtle)]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [y] ", theme.success.add_modifier(Modifier::BOLD)),
            Span::styled("Allow", theme.success),
            Span::raw("   "),
            Span::styled(" [n] ", theme.error.add_modifier(Modifier::BOLD)),
            Span::styled("Deny", theme.error),
        ]),
    ]
}
