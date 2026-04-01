//! TUI color schemes and semantic styles.
//!
//! Color values are aligned with Claude Code's `utils/theme.ts`.
//! The dark palette mirrors `darkTheme`, light mirrors `lightTheme`.

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// Built-in color schemes for the ratatui backend.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
	/// Default dark-oriented palette (true-color RGB).
	#[default]
	Default,
	/// A lighter palette for bright terminal backgrounds (true-color RGB).
	Light,
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
}

impl Theme {
	/// Resolve the active theme from a built-in color scheme and the global no-color flag.
	pub fn from_scheme(scheme: ColorScheme, no_color: bool) -> Self {
		if no_color {
			return Self::no_color(scheme);
		}

		match scheme {
			ColorScheme::Default => Self::default_dark(),
			ColorScheme::Light => Self::light(),
		}
	}

	/// CC `darkTheme` — true-color RGB values.
	fn default_dark() -> Self {
		let claude_rgb = Color::Rgb(215, 119, 87);

		Self {
			color_scheme: ColorScheme::Default,
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
		}
	}
}
