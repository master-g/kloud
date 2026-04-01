//! Shared helpers: color blending, gradients, styles, and formatting.

use std::time::Duration;

use ratatui::style::{Color, Modifier, Style};

use crate::ui::constants::{
	ACTIVITY_FADE_WINDOW_MS, ACTIVITY_MIN_RETAIN, THINKING_DELAY_MS, THINKING_GLOW_PERIOD_S,
	THINKING_INACTIVE, THINKING_SHIMMER,
};
use crate::ui::tui::state::{ActivityAccent, LiveActivity};
use crate::ui::tui::theme::{ColorScheme, Theme};

#[derive(Clone, Copy)]
pub(super) struct ActivityStyles {
	pub(super) glyph: Style,
	pub(super) object: Style,
}

#[derive(Clone, Copy)]
pub(super) struct ActivityGradient {
	pub(super) glyph: (u8, u8, u8),
	pub(super) hot: (u8, u8, u8),
	pub(super) base: (u8, u8, u8),
	pub(super) object: (u8, u8, u8),
	pub(super) fade: (u8, u8, u8),
	pub(super) object_fade: (u8, u8, u8),
}

pub(super) fn activity_styles(activity: &LiveActivity, theme: &Theme) -> ActivityStyles {
	if theme.no_color {
		let plain = Style::default();
		let bold = plain.add_modifier(Modifier::BOLD);
		return ActivityStyles {
			glyph: bold,
			object: plain,
		};
	}

	let retain = activity_decay_mix(activity);
	let gradient = activity_gradient(theme, activity.accent);
	let glyph =
		rgb_style(blend_rgb(gradient.glyph, gradient.fade, retain)).add_modifier(Modifier::BOLD);
	let object =
		rgb_style(blend_rgb(gradient.object, gradient.object_fade, 0.28 + (retain * 0.72)));

	ActivityStyles {
		glyph,
		object,
	}
}

pub(super) fn activity_gradient(theme: &Theme, accent: ActivityAccent) -> ActivityGradient {
	match (theme.color_scheme, accent) {
		(ColorScheme::Default, ActivityAccent::Info) => ActivityGradient {
			glyph: (118, 209, 255),
			hot: (170, 238, 255),
			base: (92, 173, 255),
			object: (205, 222, 255),
			fade: (82, 92, 108),
			object_fade: (104, 110, 122),
		},
		(ColorScheme::Default, ActivityAccent::Tool) => ActivityGradient {
			glyph: (224, 119, 255),
			hot: (255, 182, 247),
			base: (200, 102, 255),
			object: (235, 208, 255),
			fade: (94, 82, 112),
			object_fade: (110, 103, 122),
		},
		(ColorScheme::Light, ActivityAccent::Info) => ActivityGradient {
			glyph: (0, 120, 212),
			hot: (32, 148, 255),
			base: (30, 102, 196),
			object: (44, 60, 88),
			fade: (136, 144, 156),
			object_fade: (146, 152, 162),
		},
		(ColorScheme::Light, ActivityAccent::Tool) => ActivityGradient {
			glyph: (156, 62, 201),
			hot: (198, 98, 232),
			base: (132, 54, 184),
			object: (82, 54, 98),
			fade: (144, 138, 152),
			object_fade: (152, 148, 160),
		},
	}
}

fn activity_decay_mix(activity: &LiveActivity) -> f32 {
	let elapsed_ms = activity.last_signal_at.elapsed().as_millis().min(ACTIVITY_FADE_WINDOW_MS);
	let progress = elapsed_ms as f32 / ACTIVITY_FADE_WINDOW_MS as f32;
	1.0 - progress * (1.0 - ACTIVITY_MIN_RETAIN)
}

fn blend_rgb(from: (u8, u8, u8), to: (u8, u8, u8), retain: f32) -> Color {
	let retain = retain.clamp(0.0, 1.0);
	Color::Rgb(
		blend_channel(from.0, to.0, retain),
		blend_channel(from.1, to.1, retain),
		blend_channel(from.2, to.2, retain),
	)
}

fn blend_channel(from: u8, to: u8, retain: f32) -> u8 {
	let retained = from as f32 * retain;
	let faded = to as f32 * (1.0 - retain);
	(retained + faded).round() as u8
}

/// Linear interpolation between two RGB colors.
/// At `t = 0.0` returns `from`, at `t = 1.0` returns `to`.
pub(super) fn interpolate_color_to_rgb(from: (u8, u8, u8), to: (u8, u8, u8), t: f32) -> Color {
	let t = t.clamp(0.0, 1.0);
	let inv = 1.0 - t;
	Color::Rgb(
		(from.0 as f32 * inv + to.0 as f32 * t).round() as u8,
		(from.1 as f32 * inv + to.1 as f32 * t).round() as u8,
		(from.2 as f32 * inv + to.2 as f32 * t).round() as u8,
	)
}

pub(super) fn rgb_style(color: Color) -> Style {
	Style::default().fg(color)
}

/// Thinking shimmer breathing effect.
///
/// Claude Code: 3 second delay, then 2 second sine wave period.
/// Colors interpolate between (153,153,153) and (185,185,185).
pub(super) fn thinking_shimmer_style(tick: u64) -> Style {
	let elapsed_ms = tick * 50;
	if elapsed_ms < THINKING_DELAY_MS * 1000 {
		return rgb_style(Color::Rgb(
			THINKING_INACTIVE.0,
			THINKING_INACTIVE.1,
			THINKING_INACTIVE.2,
		));
	}

	let elapsed_since_start = (elapsed_ms - THINKING_DELAY_MS * 1000) as f32 / 1000.0;
	let phase = elapsed_since_start * std::f32::consts::PI * 2.0 / THINKING_GLOW_PERIOD_S;
	let opacity = (phase.sin() + 1.0) / 2.0;

	let r = (THINKING_INACTIVE.0 as f32
		+ (THINKING_SHIMMER.0 as f32 - THINKING_INACTIVE.0 as f32) * opacity) as u8;
	let g = (THINKING_INACTIVE.1 as f32
		+ (THINKING_SHIMMER.1 as f32 - THINKING_INACTIVE.1 as f32) * opacity) as u8;
	let b = (THINKING_INACTIVE.2 as f32
		+ (THINKING_SHIMMER.2 as f32 - THINKING_INACTIVE.2 as f32) * opacity) as u8;

	rgb_style(Color::Rgb(r, g, b))
}

pub(super) fn format_elapsed(duration: Duration) -> String {
	let seconds = duration.as_secs();
	if seconds < 60 {
		format!("{seconds}s")
	} else {
		let minutes = seconds / 60;
		let remainder = seconds % 60;
		format!("{minutes}m {remainder:02}s")
	}
}

/// Format a number with thousand separators (e.g. 1234 → "1,234").
/// Matches CC's `formatNumber()` used in the activity line token display.
pub(super) fn format_number(n: u32) -> String {
	if n < 1_000 {
		return n.to_string();
	}
	let s = n.to_string();
	let mut result = String::with_capacity(s.len() + s.len() / 3);
	for (i, ch) in s.chars().enumerate() {
		if i > 0 && (s.len() - i).is_multiple_of(3) {
			result.push(',');
		}
		result.push(ch);
	}
	result
}
