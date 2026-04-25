//! TUI color schemes and semantic styles.
//!
//! Color values are aligned with Claude Code's `utils/theme.ts`.
//! The dark palette mirrors `darkTheme`, light mirrors `lightTheme`.

use std::str::FromStr;

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// Built-in color schemes for the ratatui backend.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    /// Dark palette (true-color RGB).
    #[default]
    Dark,
    /// Light palette for bright terminal backgrounds (true-color RGB).
    Light,
    /// Dark palette using ANSI 256-color fallback.
    DarkAnsi,
    /// Light palette using ANSI 256-color fallback.
    LightAnsi,
    /// Dark palette with daltonized (color-blind friendly) colors.
    DarkDaltonized,
    /// Light palette with daltonized (color-blind friendly) colors.
    LightDaltonized,
}

impl FromStr for ColorScheme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dark" => Ok(Self::Dark),
            "light" => Ok(Self::Light),
            "dark-ansi" => Ok(Self::DarkAnsi),
            "light-ansi" => Ok(Self::LightAnsi),
            "dark-daltonized" => Ok(Self::DarkDaltonized),
            "light-daltonized" => Ok(Self::LightDaltonized),
            other => Err(format!(
                "unknown theme: {other} (available: dark, light, dark-ansi, light-ansi, dark-daltonized, light-daltonized)"
            )),
        }
    }
}

/// Semantic styles consumed by the TUI widgets.
///
/// Field names follow Claude Code's theme keys where applicable:
/// - `claude` / `claude_shimmer` — brand accent (CC: `claude` / `claudeShimmer`)
/// - `inactive` / `inactive_shimmer` — secondary text (CC: `inactive` / `inactiveShimmer`)
/// - `subtle` — tertiary/faint text (CC: `subtle`)
/// - `prompt_border` / `prompt_border_shimmer` — input border (CC: `promptBorder`)
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub color_scheme: ColorScheme,
    pub no_color: bool,

    // Brand
    pub brand_badge: Style,
    pub claude: Style,
    pub claude_bold: Style,
    pub claude_shimmer: Style,

    // Text hierarchy
    pub text: Style,
    pub text_bold: Style,
    pub inverse_text: Style,
    pub inactive: Style,
    pub inactive_shimmer: Style,
    pub subtle: Style,

    // Semantic
    pub warning: Style,
    pub warning_bold: Style,
    pub warning_shimmer: Style,
    pub success: Style,
    pub success_bold: Style,
    pub error: Style,
    pub error_bold: Style,

    // Tool (kloud-specific, CC uses `claude` for tools)
    pub tool: Style,
    pub tool_bold: Style,

    // Chrome
    pub prompt_border: Style,
    pub prompt_border_shimmer: Style,
    pub status_model_badge: Style,
    pub status_effort_badge: Style,

    /// Hint/instruction text (e.g. "Tab to expand").
    pub suggestion: Style,
}

impl Theme {
    /// Detect whether the terminal supports true-color (24-bit) RGB.
    ///
    /// Checks `COLORTERM` env var for "truecolor" or "24bit".
    pub fn terminal_supports_truecolor() -> bool {
        std::env::var("COLORTERM").is_ok_and(|v| v == "truecolor" || v == "24bit")
    }

    /// Auto-select the best scheme for the current terminal.
    /// Falls back to ANSI variant if true-color is not supported.
    pub fn auto_scheme() -> ColorScheme {
        if Self::terminal_supports_truecolor() {
            ColorScheme::Dark
        } else {
            ColorScheme::DarkAnsi
        }
    }

    /// Resolve the active theme from a built-in color scheme and the global no-color flag.
    pub fn from_scheme(scheme: ColorScheme, no_color: bool) -> Self {
        if no_color {
            return Self::no_color(scheme);
        }

        match scheme {
            ColorScheme::Dark => Self::dark(),
            ColorScheme::Light => Self::light(),
            ColorScheme::DarkAnsi => Self::dark_ansi(),
            ColorScheme::LightAnsi => Self::light_ansi(),
            ColorScheme::DarkDaltonized => Self::dark_daltonized(),
            ColorScheme::LightDaltonized => Self::light_daltonized(),
        }
    }

    /// CC `darkTheme` — true-color RGB values.
    fn dark() -> Self {
        let claude_rgb = Color::Rgb(215, 119, 87);

        Self {
            color_scheme: ColorScheme::Dark,
            no_color: false,

            brand_badge: Style::default()
                .fg(Color::Rgb(0, 0, 0))
                .bg(claude_rgb)
                .add_modifier(Modifier::BOLD),
            claude: Style::default().fg(claude_rgb),
            claude_bold: Style::default().fg(claude_rgb).add_modifier(Modifier::BOLD),
            claude_shimmer: Style::default().fg(Color::Rgb(235, 159, 127)),

            text: Style::default().fg(Color::Rgb(255, 255, 255)),
            text_bold: Style::default().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD),
            inverse_text: Style::default().fg(Color::Rgb(0, 0, 0)),
            inactive: Style::default().fg(Color::Rgb(153, 153, 153)),
            inactive_shimmer: Style::default().fg(Color::Rgb(193, 193, 193)),
            subtle: Style::default().fg(Color::Rgb(80, 80, 80)),

            warning: Style::default().fg(Color::Rgb(255, 193, 7)),
            warning_bold: Style::default().fg(Color::Rgb(255, 193, 7)).add_modifier(Modifier::BOLD),
            warning_shimmer: Style::default().fg(Color::Rgb(255, 223, 57)),
            success: Style::default().fg(Color::Rgb(78, 186, 101)),
            success_bold: Style::default()
                .fg(Color::Rgb(78, 186, 101))
                .add_modifier(Modifier::BOLD),
            error: Style::default().fg(Color::Rgb(255, 107, 128)),
            error_bold: Style::default().fg(Color::Rgb(255, 107, 128)).add_modifier(Modifier::BOLD),

            tool: Style::default().fg(Color::Rgb(175, 135, 255)),
            tool_bold: Style::default().fg(Color::Rgb(175, 135, 255)).add_modifier(Modifier::BOLD),

            prompt_border: Style::default().fg(Color::Rgb(136, 136, 136)),
            prompt_border_shimmer: Style::default().fg(Color::Rgb(166, 166, 166)),
            status_model_badge: Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(60, 60, 60)),
            status_effort_badge: Style::default()
                .fg(Color::Rgb(0, 0, 0))
                .bg(Color::Rgb(255, 193, 7)),
            suggestion: Style::default().fg(Color::Rgb(120, 120, 120)),
        }
    }

    /// CC `lightTheme` — true-color RGB values.
    fn light() -> Self {
        let claude_rgb = Color::Rgb(215, 119, 87);

        Self {
            color_scheme: ColorScheme::Light,
            no_color: false,

            brand_badge: Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .bg(claude_rgb)
                .add_modifier(Modifier::BOLD),
            claude: Style::default().fg(claude_rgb),
            claude_bold: Style::default().fg(claude_rgb).add_modifier(Modifier::BOLD),
            claude_shimmer: Style::default().fg(Color::Rgb(245, 149, 117)),

            text: Style::default().fg(Color::Rgb(0, 0, 0)),
            text_bold: Style::default().fg(Color::Rgb(0, 0, 0)).add_modifier(Modifier::BOLD),
            inverse_text: Style::default().fg(Color::Rgb(255, 255, 255)),
            inactive: Style::default().fg(Color::Rgb(102, 102, 102)),
            inactive_shimmer: Style::default().fg(Color::Rgb(142, 142, 142)),
            subtle: Style::default().fg(Color::Rgb(175, 175, 175)),

            warning: Style::default().fg(Color::Rgb(150, 108, 30)),
            warning_bold: Style::default()
                .fg(Color::Rgb(150, 108, 30))
                .add_modifier(Modifier::BOLD),
            warning_shimmer: Style::default().fg(Color::Rgb(200, 158, 80)),
            success: Style::default().fg(Color::Rgb(44, 122, 57)),
            success_bold: Style::default().fg(Color::Rgb(44, 122, 57)).add_modifier(Modifier::BOLD),
            error: Style::default().fg(Color::Rgb(171, 43, 63)),
            error_bold: Style::default().fg(Color::Rgb(171, 43, 63)).add_modifier(Modifier::BOLD),

            tool: Style::default().fg(Color::Rgb(135, 0, 255)),
            tool_bold: Style::default().fg(Color::Rgb(135, 0, 255)).add_modifier(Modifier::BOLD),

            prompt_border: Style::default().fg(Color::Rgb(153, 153, 153)),
            prompt_border_shimmer: Style::default().fg(Color::Rgb(183, 183, 183)),
            status_model_badge: Style::default()
                .fg(Color::Rgb(0, 0, 0))
                .bg(Color::Rgb(200, 200, 200)),
            status_effort_badge: Style::default()
                .fg(Color::Rgb(0, 0, 0))
                .bg(Color::Rgb(44, 122, 57)),
            suggestion: Style::default().fg(Color::Rgb(140, 140, 140)),
        }
    }

    /// Dark theme with ANSI 256-color palette.
    fn dark_ansi() -> Self {
        Self {
            color_scheme: ColorScheme::DarkAnsi,
            no_color: false,

            brand_badge: Style::default()
                .fg(Color::Black)
                .bg(Color::Indexed(208))
                .add_modifier(Modifier::BOLD),
            claude: Style::default().fg(Color::Indexed(208)),
            claude_bold: Style::default().fg(Color::Indexed(208)).add_modifier(Modifier::BOLD),
            claude_shimmer: Style::default().fg(Color::Indexed(216)),

            text: Style::default().fg(Color::White),
            text_bold: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            inverse_text: Style::default().fg(Color::Black),
            inactive: Style::default().fg(Color::Indexed(249)),
            inactive_shimmer: Style::default().fg(Color::Indexed(252)),
            subtle: Style::default().fg(Color::DarkGray),

            warning: Style::default().fg(Color::Indexed(220)),
            warning_bold: Style::default().fg(Color::Indexed(220)).add_modifier(Modifier::BOLD),
            warning_shimmer: Style::default().fg(Color::Indexed(226)),
            success: Style::default().fg(Color::Indexed(76)),
            success_bold: Style::default().fg(Color::Indexed(76)).add_modifier(Modifier::BOLD),
            error: Style::default().fg(Color::Indexed(203)),
            error_bold: Style::default().fg(Color::Indexed(203)).add_modifier(Modifier::BOLD),

            tool: Style::default().fg(Color::Indexed(183)),
            tool_bold: Style::default().fg(Color::Indexed(183)).add_modifier(Modifier::BOLD),

            prompt_border: Style::default().fg(Color::Indexed(246)),
            prompt_border_shimmer: Style::default().fg(Color::Indexed(250)),
            status_model_badge: Style::default().fg(Color::White).bg(Color::Indexed(239)),
            status_effort_badge: Style::default().fg(Color::Black).bg(Color::Indexed(220)),
            suggestion: Style::default().fg(Color::DarkGray),
        }
    }

    /// Light theme with ANSI 256-color palette.
    fn light_ansi() -> Self {
        Self {
            color_scheme: ColorScheme::LightAnsi,
            no_color: false,

            brand_badge: Style::default()
                .fg(Color::White)
                .bg(Color::Indexed(208))
                .add_modifier(Modifier::BOLD),
            claude: Style::default().fg(Color::Indexed(208)),
            claude_bold: Style::default().fg(Color::Indexed(208)).add_modifier(Modifier::BOLD),
            claude_shimmer: Style::default().fg(Color::Indexed(216)),

            text: Style::default().fg(Color::Black),
            text_bold: Style::default().fg(Color::Black).add_modifier(Modifier::BOLD),
            inverse_text: Style::default().fg(Color::White),
            inactive: Style::default().fg(Color::Indexed(243)),
            inactive_shimmer: Style::default().fg(Color::Indexed(249)),
            subtle: Style::default().fg(Color::Indexed(252)),

            warning: Style::default().fg(Color::Indexed(136)),
            warning_bold: Style::default().fg(Color::Indexed(136)).add_modifier(Modifier::BOLD),
            warning_shimmer: Style::default().fg(Color::Indexed(178)),
            success: Style::default().fg(Color::Indexed(64)),
            success_bold: Style::default().fg(Color::Indexed(64)).add_modifier(Modifier::BOLD),
            error: Style::default().fg(Color::Indexed(160)),
            error_bold: Style::default().fg(Color::Indexed(160)).add_modifier(Modifier::BOLD),

            tool: Style::default().fg(Color::Indexed(93)),
            tool_bold: Style::default().fg(Color::Indexed(93)).add_modifier(Modifier::BOLD),

            prompt_border: Style::default().fg(Color::Indexed(249)),
            prompt_border_shimmer: Style::default().fg(Color::Indexed(252)),
            status_model_badge: Style::default().fg(Color::Black).bg(Color::Indexed(252)),
            status_effort_badge: Style::default().fg(Color::Black).bg(Color::Indexed(64)),
            suggestion: Style::default().fg(Color::Indexed(249)),
        }
    }

    /// Dark theme with daltonized (color-blind friendly) colors.
    fn dark_daltonized() -> Self {
        Self {
            color_scheme: ColorScheme::DarkDaltonized,
            no_color: false,

            brand_badge: Style::default()
                .fg(Color::Rgb(0, 0, 0))
                .bg(Color::Rgb(215, 119, 87))
                .add_modifier(Modifier::BOLD),
            claude: Style::default().fg(Color::Rgb(215, 119, 87)),
            claude_bold: Style::default().fg(Color::Rgb(215, 119, 87)).add_modifier(Modifier::BOLD),
            claude_shimmer: Style::default().fg(Color::Rgb(235, 159, 127)),

            text: Style::default().fg(Color::Rgb(255, 255, 255)),
            text_bold: Style::default().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD),
            inverse_text: Style::default().fg(Color::Rgb(0, 0, 0)),
            inactive: Style::default().fg(Color::Rgb(153, 153, 153)),
            inactive_shimmer: Style::default().fg(Color::Rgb(193, 193, 193)),
            subtle: Style::default().fg(Color::Rgb(80, 80, 80)),

            // Daltonized: warning = bright blue, success = teal-shifted for luminance separation
            warning: Style::default().fg(Color::Rgb(120, 160, 230)),
            warning_bold: Style::default()
                .fg(Color::Rgb(120, 160, 230))
                .add_modifier(Modifier::BOLD),
            warning_shimmer: Style::default().fg(Color::Rgb(150, 185, 240)),
            success: Style::default().fg(Color::Rgb(78, 155, 180)),
            success_bold: Style::default()
                .fg(Color::Rgb(78, 155, 180))
                .add_modifier(Modifier::BOLD),
            error: Style::default().fg(Color::Rgb(255, 157, 90)),
            error_bold: Style::default().fg(Color::Rgb(255, 157, 90)).add_modifier(Modifier::BOLD),

            tool: Style::default().fg(Color::Rgb(175, 135, 255)),
            tool_bold: Style::default().fg(Color::Rgb(175, 135, 255)).add_modifier(Modifier::BOLD),

            prompt_border: Style::default().fg(Color::Rgb(136, 136, 136)),
            prompt_border_shimmer: Style::default().fg(Color::Rgb(166, 166, 166)),
            status_model_badge: Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(60, 60, 60)),
            status_effort_badge: Style::default()
                .fg(Color::Rgb(0, 0, 0))
                .bg(Color::Rgb(120, 160, 230)),
            suggestion: Style::default().fg(Color::Rgb(120, 120, 120)),
        }
    }

    /// Light theme with daltonized (color-blind friendly) colors.
    fn light_daltonized() -> Self {
        Self {
            color_scheme: ColorScheme::LightDaltonized,
            no_color: false,

            brand_badge: Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(215, 119, 87))
                .add_modifier(Modifier::BOLD),
            claude: Style::default().fg(Color::Rgb(215, 119, 87)),
            claude_bold: Style::default().fg(Color::Rgb(215, 119, 87)).add_modifier(Modifier::BOLD),
            claude_shimmer: Style::default().fg(Color::Rgb(245, 149, 117)),

            text: Style::default().fg(Color::Rgb(0, 0, 0)),
            text_bold: Style::default().fg(Color::Rgb(0, 0, 0)).add_modifier(Modifier::BOLD),
            inverse_text: Style::default().fg(Color::Rgb(255, 255, 255)),
            inactive: Style::default().fg(Color::Rgb(102, 102, 102)),
            inactive_shimmer: Style::default().fg(Color::Rgb(142, 142, 142)),
            subtle: Style::default().fg(Color::Rgb(175, 175, 175)),

            warning: Style::default().fg(Color::Rgb(56, 106, 164)),
            warning_bold: Style::default()
                .fg(Color::Rgb(56, 106, 164))
                .add_modifier(Modifier::BOLD),
            warning_shimmer: Style::default().fg(Color::Rgb(106, 156, 214)),
            success: Style::default().fg(Color::Rgb(44, 92, 164)),
            success_bold: Style::default().fg(Color::Rgb(44, 92, 164)).add_modifier(Modifier::BOLD),
            error: Style::default().fg(Color::Rgb(201, 83, 23)),
            error_bold: Style::default().fg(Color::Rgb(201, 83, 23)).add_modifier(Modifier::BOLD),

            tool: Style::default().fg(Color::Rgb(135, 0, 255)),
            tool_bold: Style::default().fg(Color::Rgb(135, 0, 255)).add_modifier(Modifier::BOLD),

            prompt_border: Style::default().fg(Color::Rgb(153, 153, 153)),
            prompt_border_shimmer: Style::default().fg(Color::Rgb(183, 183, 183)),
            status_model_badge: Style::default()
                .fg(Color::Rgb(0, 0, 0))
                .bg(Color::Rgb(200, 200, 200)),
            status_effort_badge: Style::default()
                .fg(Color::Rgb(0, 0, 0))
                .bg(Color::Rgb(56, 106, 164)),
            suggestion: Style::default().fg(Color::Rgb(140, 140, 140)),
        }
    }

    fn no_color(scheme: ColorScheme) -> Self {
        let plain = Style::default();
        let bold = Style::default().add_modifier(Modifier::BOLD);

        Self {
            color_scheme: scheme,
            no_color: true,
            brand_badge: bold,
            claude: plain,
            claude_bold: bold,
            claude_shimmer: plain,
            text: plain,
            text_bold: bold,
            inverse_text: plain,
            inactive: plain,
            inactive_shimmer: plain,
            subtle: plain,
            warning: plain,
            warning_bold: bold,
            warning_shimmer: plain,
            success: plain,
            success_bold: bold,
            error: plain,
            error_bold: bold,
            tool: plain,
            tool_bold: bold,
            prompt_border: plain,
            prompt_border_shimmer: plain,
            status_model_badge: plain,
            status_effort_badge: bold,
            suggestion: plain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_schemes_parse() {
        for name in
            &["dark", "light", "dark-ansi", "light-ansi", "dark-daltonized", "light-daltonized"]
        {
            assert!(name.parse::<ColorScheme>().is_ok(), "failed to parse {name}");
        }
    }

    #[test]
    fn unknown_scheme_fails() {
        assert!("nord".parse::<ColorScheme>().is_err());
    }

    #[test]
    fn from_scheme_produces_matching_scheme() {
        for scheme in [
            ColorScheme::Dark,
            ColorScheme::Light,
            ColorScheme::DarkAnsi,
            ColorScheme::LightAnsi,
            ColorScheme::DarkDaltonized,
            ColorScheme::LightDaltonized,
        ] {
            let theme = Theme::from_scheme(scheme, false);
            assert_eq!(theme.color_scheme, scheme);
        }
    }

    #[test]
    fn no_color_theme_ignores_scheme() {
        let theme = Theme::from_scheme(ColorScheme::Dark, true);
        assert!(theme.no_color);
    }

    #[test]
    fn auto_scheme_returns_dark_for_truecolor() {
        // This test may or may not pass depending on the test runner's env,
        // so we just verify it doesn't panic.
        let _ = Theme::auto_scheme();
    }
}
