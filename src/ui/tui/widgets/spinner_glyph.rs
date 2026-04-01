//! Spinner glyph animation (対標 `SpinnerGlyph`).

use crate::ui::constants::{BOUNCE_FRAMES, GLYPH_FRAME_INTERVAL_MS};
use crate::ui::tui::state::SpinnerMode;

/// Select the current spinner glyph from the 12-frame bounce array.
///
/// Claude Code: `frame = Math.floor(time / 120)` where time is in ms.
/// Here `tick` is in 50ms units, so `time = tick * 50`.
pub(super) fn active_glyph(_mode: SpinnerMode, tick: u64) -> &'static str {
	let frame = (tick * 50 / GLYPH_FRAME_INTERVAL_MS) as usize;
	BOUNCE_FRAMES[frame % BOUNCE_FRAMES.len()]
}
