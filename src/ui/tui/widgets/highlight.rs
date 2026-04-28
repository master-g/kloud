//! Syntect-based syntax highlighting for code blocks.

use std::sync::LazyLock;

use ratatui::style::{Color, Modifier, Style};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

static SYNTECT_THEME: LazyLock<syntect::highlighting::Theme> = LazyLock::new(|| {
    let ts = ThemeSet::load_defaults();
    ts.themes["base16-eighties.dark"].clone()
});

/// Highlight a line of code in the given language, returning styled spans.
pub fn highlight_line<'a>(
    lang: &str,
    line: &str,
    highlight_lines: &mut Option<syntect::easy::HighlightLines<'static>>,
) -> Vec<ratatui::text::Span<'a>> {
    if highlight_lines.is_none() {
        let syntax = SYNTAX_SET
            .find_syntax_by_token(lang)
            .or_else(|| SYNTAX_SET.find_syntax_by_extension(lang));

        let Some(syntax) = syntax else {
            return vec![ratatui::text::Span::raw(line.to_string())];
        };

        *highlight_lines = Some(syntect::easy::HighlightLines::new(syntax, &SYNTECT_THEME));
    }

    let Some(hl) = highlight_lines else {
        return vec![ratatui::text::Span::raw(line.to_string())];
    };

    match hl.highlight_line(line, &SYNTAX_SET) {
        Ok(ranges) => {
            let mut spans = Vec::new();
            for (style, text) in ranges {
                spans.push(ratatui::text::Span::styled(
                    text.to_string(),
                    syntect_style_to_ratatui(style),
                ));
            }
            if spans.is_empty() {
                spans.push(ratatui::text::Span::raw(line.to_string()));
            }
            spans
        }
        Err(_) => vec![ratatui::text::Span::raw(line.to_string())],
    }
}

/// Check if a language token is recognized by syntect.
pub fn is_language_supported(lang: &str) -> bool {
    SYNTAX_SET
        .find_syntax_by_token(lang)
        .or_else(|| SYNTAX_SET.find_syntax_by_extension(lang))
        .is_some()
}

fn syntect_style_to_ratatui(style: syntect::highlighting::Style) -> Style {
    let fg = syntect_color_to_ratatui(style.foreground);
    let mut s = Style::default().fg(fg);

    if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
        s = s.add_modifier(Modifier::BOLD);
    }
    if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
        s = s.add_modifier(Modifier::ITALIC);
    }
    if style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
        s = s.add_modifier(Modifier::UNDERLINED);
    }

    s
}

fn syntect_color_to_ratatui(color: syntect::highlighting::Color) -> Color {
    if color.a == 0 {
        return Color::Reset;
    }
    Color::Rgb(color.r, color.g, color.b)
}
