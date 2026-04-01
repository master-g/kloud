//! TUI color schemes and semantic styles.

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

/// Built-in color schemes for the ratatui backend.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
	/// Default dark-oriented palette.
	#[default]
	Default,
	/// A lighter palette for bright terminal backgrounds.
	Light,
}

/// Semantic styles consumed by the TUI widgets.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct Theme {
	pub color_scheme: ColorScheme,
	pub no_color: bool,
	pub brand_badge: Style,
	pub text: Style,
	pub text_bold: Style,
	pub muted: Style,
	pub dim: Style,
	pub info: Style,
	pub info_bold: Style,
	pub warning: Style,
	pub warning_bold: Style,
	pub success: Style,
	pub success_bold: Style,
	pub error: Style,
	pub error_bold: Style,
	pub tool: Style,
	pub tool_bold: Style,
	pub status_model_badge: Style,
	pub status_effort_badge: Style,
	pub border_dim: Style,
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

	fn default_dark() -> Self {
		Self {
			color_scheme: ColorScheme::Default,
			no_color: false,
			brand_badge: Style::default()
				.fg(Color::Black)
				.bg(Color::Cyan)
				.add_modifier(Modifier::BOLD),
			text: Style::default().fg(Color::White),
			text_bold: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
			muted: Style::default().fg(Color::Gray),
			dim: Style::default().fg(Color::DarkGray),
			info: Style::default().fg(Color::Cyan),
			info_bold: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
			warning: Style::default().fg(Color::Yellow),
			warning_bold: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
			success: Style::default().fg(Color::Green),
			success_bold: Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
			error: Style::default().fg(Color::Red),
			error_bold: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
			tool: Style::default().fg(Color::Magenta),
			tool_bold: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
			status_model_badge: Style::default().fg(Color::White).bg(Color::DarkGray),
			status_effort_badge: Style::default().fg(Color::Black).bg(Color::Yellow),
			border_dim: Style::default().fg(Color::DarkGray),
		}
	}

	fn light() -> Self {
		Self {
			color_scheme: ColorScheme::Light,
			no_color: false,
			brand_badge: Style::default()
				.fg(Color::White)
				.bg(Color::Blue)
				.add_modifier(Modifier::BOLD),
			text: Style::default().fg(Color::Black),
			text_bold: Style::default().fg(Color::Black).add_modifier(Modifier::BOLD),
			muted: Style::default().fg(Color::DarkGray),
			dim: Style::default().fg(Color::Gray),
			info: Style::default().fg(Color::Blue),
			info_bold: Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
			warning: Style::default().fg(Color::Yellow),
			warning_bold: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
			success: Style::default().fg(Color::Green),
			success_bold: Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
			error: Style::default().fg(Color::Red),
			error_bold: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
			tool: Style::default().fg(Color::Magenta),
			tool_bold: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
			status_model_badge: Style::default().fg(Color::Black).bg(Color::Gray),
			status_effort_badge: Style::default().fg(Color::Black).bg(Color::Green),
			border_dim: Style::default().fg(Color::Gray),
		}
	}

	fn no_color(scheme: ColorScheme) -> Self {
		let plain = Style::default();
		let bold = Style::default().add_modifier(Modifier::BOLD);

		Self {
			color_scheme: scheme,
			no_color: true,
			brand_badge: bold,
			text: plain,
			text_bold: bold,
			muted: plain,
			dim: plain,
			info: plain,
			info_bold: bold,
			warning: plain,
			warning_bold: bold,
			success: plain,
			success_bold: bold,
			error: plain,
			error_bold: bold,
			tool: plain,
			tool_bold: bold,
			status_model_badge: plain,
			status_effort_badge: bold,
			border_dim: plain,
		}
	}
}
