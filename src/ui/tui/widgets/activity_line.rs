//! Activity line: the live status row above the assistant message (対標 `SpinnerAnimationRow`).
//!
//! Implements progressive width gating and Byline separators matching Claude
//! Code's `SpinnerAnimationRow` layout algorithm.

use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::ui::constants::{ERROR_RED, SHOW_TOKENS_AFTER_SECS, THINKING_BARE_WIDTH};
use crate::ui::tui::state::{AssistantStatus, SpinnerMode, TuiState};
use crate::ui::tui::theme::Theme;

use super::glimmer::{compute_glimmer_index, glimmer_spans};
use super::helpers::{
    activity_gradient, activity_styles, format_elapsed, format_number, interpolate_color_to_rgb,
    rgb_style, thinking_shimmer_style,
};
use super::spinner_glyph::active_glyph;

pub(super) fn render_live_assistant_header(
    state: &TuiState,
    theme: &Theme,
    columns: u16,
) -> Line<'static> {
    if state.status == AssistantStatus::Cancelling {
        return Line::from(vec![
            Span::styled(
                active_glyph(SpinnerMode::ToolUse, state.activity_clock.tick),
                theme.warning_bold,
            ),
            Span::raw(" "),
            Span::styled("Cancelling", theme.warning_bold),
            Span::raw(" "),
            Span::styled("current turn", theme.text),
        ]);
    }

    if let Some(activity) = state.displayed_activity() {
        let tick = state.activity_clock.tick;
        let styles = activity_styles(activity, theme);
        let stalled_intensity = state.stalled_state.intensity();

        // --- Glyph ---
        let glyph_style = if stalled_intensity > 0.0 {
            let gradient = activity_gradient(theme, activity.accent);
            let interpolated =
                interpolate_color_to_rgb(gradient.glyph, ERROR_RED, stalled_intensity);
            rgb_style(interpolated).add_modifier(Modifier::BOLD)
        } else {
            styles.glyph
        };

        let mut spans = vec![
            Span::styled(active_glyph(activity.mode, state.activity_clock.tick), glyph_style),
            Span::raw(" "),
        ];

        // --- Verb message (glimmer) ---
        let msg = &activity.message;
        let gradient = activity_gradient(theme, activity.accent);
        let glimmer_index = compute_glimmer_index(activity.mode, tick, msg.width());
        let flash_opacity = if activity.mode == SpinnerMode::ToolUse {
            let time_ms = tick as f64 * 50.0;
            ((time_ms / 1000.0 * std::f64::consts::PI).sin() + 1.0) / 2.0
        } else {
            0.0
        } as f32;
        spans.extend(glimmer_spans(
            msg,
            glimmer_index,
            &gradient,
            stalled_intensity,
            flash_opacity,
        ));

        // --- Status area with progressive width gating ---
        let message_width = msg.width();
        let available = columns as i32 - message_width as i32 - 4; // 4 = glyph(2) + spaces(2)
        let sep_w = 3i32; // width of " · "

        // Thinking (highest priority in width budget)
        let effort_suffix = state.effort_suffix();
        let mut thinking_text = state.thinking_status.display_text(&effort_suffix);
        let wants_thinking = thinking_text.is_some();
        let mut thinking_w = thinking_text.as_ref().map_or(0, |t| t.width() as i32);
        let mut show_thinking = wants_thinking && available > thinking_w;

        // Width fallback: drop effort suffix and try bare "thinking"
        if !show_thinking
            && wants_thinking
            && state.thinking_status.is_shimmering()
            && !effort_suffix.is_empty()
            && available > THINKING_BARE_WIDTH as i32
        {
            thinking_text = Some("thinking".to_string());
            thinking_w = THINKING_BARE_WIDTH as i32;
            show_thinking = true;
        }

        let used_after_thinking = if show_thinking {
            thinking_w + sep_w
        } else {
            0
        };

        // CC gates both timer and tokens behind a 30-second threshold
        let elapsed_secs = activity.started_at.elapsed().as_secs();
        let wants_timer_and_tokens = elapsed_secs >= SHOW_TOKENS_AFTER_SECS;

        // Timer (plain elapsed, no arrow — arrow belongs with tokens)
        let timer_text = format_elapsed(activity.started_at.elapsed());
        let timer_w = timer_text.width() as i32;
        let show_timer = wants_timer_and_tokens && available > used_after_thinking + timer_w;
        let used_after_timer = used_after_thinking
            + if show_timer {
                timer_w + sep_w
            } else {
                0
            };

        // Tokens
        let token_value = state.token_counter.token_value();
        let tokens_text =
            format!("{} {} tokens", mode_arrow(activity.mode), format_number(token_value));
        let tokens_w = tokens_text.width() as i32;
        let show_tokens =
            wants_timer_and_tokens && token_value > 0 && available > used_after_timer + tokens_w;

        let thinking_only =
            show_thinking && state.thinking_status.is_shimmering() && !show_timer && !show_tokens;

        // Assemble status parts with Byline ` · ` separators
        let mut parts: Vec<Span<'static>> = Vec::new();
        let mut n = 0usize;

        if show_timer {
            parts.push(Span::styled(timer_text, theme.subtle));
            n += 1;
        }
        if show_tokens {
            if n > 0 {
                parts.push(Span::styled(" · ", theme.subtle));
            }
            parts.push(Span::styled(tokens_text, theme.subtle));
            n += 1;
        }
        if show_thinking && let Some(text) = thinking_text {
            if n > 0 {
                parts.push(Span::styled(" · ", theme.subtle));
            }
            if state.thinking_status.is_shimmering() {
                let style = thinking_shimmer_style(tick);
                if thinking_only {
                    parts.push(Span::styled(format!("({text})"), style));
                } else {
                    parts.push(Span::styled(text, style));
                }
            } else {
                parts.push(Span::styled(text, theme.subtle));
            }
        }

        if !parts.is_empty() {
            if thinking_only {
                spans.push(Span::raw(" "));
                spans.extend(parts);
            } else {
                spans.push(Span::styled(" (", theme.subtle));
                spans.extend(parts);
                spans.push(Span::styled(")", theme.subtle));
            }
        }

        return Line::from(spans);
    }

    Line::from("")
}

/// Render compact tool spinners: `\u{25cf} tool-name X.Xs` for each active tool.
#[allow(dead_code)]
pub(super) fn render_tool_spinners(state: &TuiState, theme: &Theme) -> Option<Line<'static>> {
    if state.active_tools.is_empty() {
        return None;
    }
    let mut spans: Vec<Span<'static>> = Vec::new();
    for (i, name) in state.active_tools.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        let elapsed =
            state.tool_start_times.get(name).map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);
        spans.push(Span::styled("\u{25cf}", theme.tool));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(name.clone(), theme.text));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(format!("{elapsed:.1}s"), theme.subtle));
    }
    Some(Line::from(spans))
}

/// Returns the arrow icon for the status area based on `SpinnerMode`.
fn mode_arrow(mode: SpinnerMode) -> &'static str {
    match mode {
        SpinnerMode::Requesting => "↑",
        SpinnerMode::Responding | SpinnerMode::ToolUse | SpinnerMode::Thinking => "↓",
    }
}
