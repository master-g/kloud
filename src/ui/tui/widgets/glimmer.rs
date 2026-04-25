//! Glimmer / shimmer effect for activity text (対標 `GlimmerMessage` + `ShimmerChar`).
//!
//! Uses a 3-segment approach (before | shimmer | after) instead of per-character
//! spans, matching Claude Code's `GlimmerMessage` optimization.

use ratatui::style::{Color, Modifier};
use ratatui::text::Span;
use unicode_width::UnicodeWidthChar;

use crate::ui::constants::{ERROR_RED, SHIMMER_SPEED_OTHER_MS, SHIMMER_SPEED_REQUESTING_MS};
use crate::ui::tui::state::SpinnerMode;

use super::helpers::{ActivityGradient, interpolate_color_to_rgb, rgb_style};

/// Compute the shimmer index for a given mode and tick.
///
/// Returns `i32` to support negative values (shimmer entering from offscreen).
/// Claude Code: requesting = left-to-right at 50ms, other modes = right-to-left at 200ms.
pub(super) fn compute_glimmer_index(mode: SpinnerMode, tick: u64, text_width: usize) -> i32 {
    let shimmer_speed = match mode {
        SpinnerMode::Requesting => SHIMMER_SPEED_REQUESTING_MS / 50,
        _ => SHIMMER_SPEED_OTHER_MS / 50,
    };

    let cycle_length = text_width as i32 + 20;
    let cycle_position = (tick / shimmer_speed) as i32 % cycle_length;

    match mode {
        SpinnerMode::Requesting => cycle_position - 10,
        _ => text_width as i32 + 10 - cycle_position,
    }
}

/// Render glimmer spans for activity verb text.
///
/// Priority (matching `GlimmerMessage.tsx`):
/// 1. Stalled (`stalled_intensity > 0`) — single span in stall-interpolated color
/// 2. Tool-use flash (`flash_opacity > 0`) — single span in flash-interpolated color
/// 3. Shimmer offscreen — single base-color span (zero overhead)
/// 4. Normal — 3-segment split: `before | shimmer | after`
pub(super) fn glimmer_spans(
    text: &str,
    glimmer_index: i32,
    gradient: &ActivityGradient,
    stalled_intensity: f32,
    flash_opacity: f32,
) -> Vec<Span<'static>> {
    if text.is_empty() {
        return vec![Span::styled(
            String::new(),
            rgb_style(Color::Rgb(gradient.fade.0, gradient.fade.1, gradient.fade.2)),
        )];
    }

    let bold = Modifier::BOLD;

    // 1. Stalled: whole text in stall-interpolated color (no shimmer)
    if stalled_intensity > 0.0 {
        let color = interpolate_color_to_rgb(gradient.base, ERROR_RED, stalled_intensity);
        return vec![Span::styled(text.to_string(), rgb_style(color).add_modifier(bold))];
    }

    // 2. Tool-use flash: whole text in flash-interpolated color
    if flash_opacity > 0.0 {
        let color = interpolate_color_to_rgb(gradient.base, gradient.hot, flash_opacity);
        return vec![Span::styled(text.to_string(), rgb_style(color).add_modifier(bold))];
    }

    // 3. Check if shimmer window is offscreen
    let text_width: i32 = text.chars().map(|ch| ch.width().unwrap_or(0) as i32).sum();
    let shimmer_start = glimmer_index - 1;
    let shimmer_end = glimmer_index + 1;

    let base_color = Color::Rgb(gradient.base.0, gradient.base.1, gradient.base.2);
    let base_style = rgb_style(base_color).add_modifier(bold);

    if shimmer_start >= text_width || shimmer_end < 0 {
        return vec![Span::styled(text.to_string(), base_style)];
    }

    // 4. 3-segment split by visual column position
    let hot_color = Color::Rgb(gradient.hot.0, gradient.hot.1, gradient.hot.2);
    let shimmer_style = rgb_style(hot_color).add_modifier(bold);

    let clamped_start = shimmer_start.max(0);
    let mut col_pos: i32 = 0;
    let mut before = String::new();
    let mut shim = String::new();
    let mut after = String::new();

    for ch in text.chars() {
        let w = ch.width().unwrap_or(0) as i32;
        if col_pos + w <= clamped_start {
            before.push(ch);
        } else if col_pos > shimmer_end {
            after.push(ch);
        } else {
            shim.push(ch);
        }
        col_pos += w;
    }

    let mut spans = Vec::with_capacity(3);
    if !before.is_empty() {
        spans.push(Span::styled(before, base_style));
    }
    if !shim.is_empty() {
        spans.push(Span::styled(shim, shimmer_style));
    }
    if !after.is_empty() {
        spans.push(Span::styled(after, base_style));
    }
    spans
}
