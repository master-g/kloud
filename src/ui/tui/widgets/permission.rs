//! Permission prompt renderer.

use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::ui::tui::state::PendingPermissionView;
use crate::ui::tui::theme::Theme;

/// Render a permission prompt with label, description, and action buttons.
#[allow(dead_code)]
pub fn render_permission_prompt(
    frame: &mut ratatui::Frame,
    theme: &Theme,
    area: Rect,
    perm: &PendingPermissionView,
) {
    let lines = vec![
        Line::from(vec![Span::styled(
            " ⚠ Permission Required",
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
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_style(theme.warning))
        .wrap(Wrap {
            trim: false,
        });

    frame.render_widget(paragraph, area);
}
