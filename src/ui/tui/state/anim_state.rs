//! Animation state: clock, stall detection, token counter, thinking status.
#![allow(missing_docs)]

use std::time::Instant;

pub use crate::agent::message::{
    ActivityAccent, ActivityEntry, ActivityEntryKind, LiveActivity, SpinnerMode,
};

#[derive(Debug, Clone)]
pub struct ActivityClock {
    pub tick: u64,
    last_update: Instant,
}

impl ActivityClock {
    pub fn new() -> Self {
        Self {
            tick: 0,
            last_update: Instant::now(),
        }
    }

    pub fn try_tick(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_update).as_millis() >= 50 {
            self.tick = self.tick.wrapping_add(1);
            self.last_update = now;
            true
        } else {
            false
        }
    }
}

impl Default for ActivityClock {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct StalledState {
    last_response_length: usize,
    last_token_at: Instant,
    intensity: f32,
    is_stalled: bool,
}

impl StalledState {
    pub fn new() -> Self {
        Self {
            last_response_length: 0,
            last_token_at: Instant::now(),
            intensity: 0.0,
            is_stalled: false,
        }
    }

    pub fn update(&mut self, response_length: usize, has_active_tools: bool) {
        let now = Instant::now();

        if response_length > self.last_response_length {
            self.last_response_length = response_length;
            self.last_token_at = now;
            self.is_stalled = false;
        } else if has_active_tools {
            self.last_token_at = now;
            self.is_stalled = false;
        }

        let elapsed_ms = now.duration_since(self.last_token_at).as_millis();
        let target = if elapsed_ms > 3000 {
            self.is_stalled = true;
            ((elapsed_ms as f32 - 3000.0) / 2000.0).min(1.0)
        } else {
            0.0
        };

        if target > 0.0 || self.intensity > 0.0 {
            self.intensity += (target - self.intensity) * 0.1;
            if self.intensity < 0.001 {
                self.intensity = 0.0;
            }
        }
    }

    pub fn intensity(&self) -> f32 {
        self.intensity
    }

    pub fn is_stalled(&self) -> bool {
        self.is_stalled
    }

    pub fn reset(&mut self) {
        self.last_response_length = 0;
        self.last_token_at = Instant::now();
        self.intensity = 0.0;
        self.is_stalled = false;
    }
}

impl Default for StalledState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TokenCounter {
    displayed_chars: usize,
    target_chars: usize,
}

impl TokenCounter {
    pub fn new() -> Self {
        Self {
            displayed_chars: 0,
            target_chars: 0,
        }
    }

    pub fn set_target(&mut self, chars: usize) {
        self.target_chars = chars;
    }

    pub fn advance(&mut self) {
        if self.displayed_chars >= self.target_chars {
            return;
        }
        let gap = self.target_chars - self.displayed_chars;
        let increment = if gap < 70 {
            3
        } else if gap < 200 {
            ((gap as f32 * 0.15).ceil() as usize).max(8)
        } else {
            50
        };
        self.displayed_chars = (self.displayed_chars + increment).min(self.target_chars);
    }

    pub fn token_value(&self) -> u32 {
        ((self.displayed_chars as f64 / 4.0).round()) as u32
    }

    pub fn reset(&mut self) {
        self.displayed_chars = 0;
        self.target_chars = 0;
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum ThinkingStatus {
    None,
    Active {
        started_at: Instant,
    },
    PendingDuration {
        duration_ms: u64,
        min_display_until: Instant,
    },
    ShowDuration {
        duration_ms: u64,
        clear_at: Instant,
    },
}

impl ThinkingStatus {
    pub fn display_text(&self, effort_suffix: &str) -> Option<String> {
        match self {
            Self::None => None,
            Self::Active {
                ..
            }
            | Self::PendingDuration {
                ..
            } => Some(format!("thinking{effort_suffix}")),
            Self::ShowDuration {
                duration_ms,
                ..
            } => {
                let secs = (*duration_ms as f64 / 1000.0).round().max(1.0) as u64;
                Some(format!("thought for {secs}s"))
            }
        }
    }

    pub fn is_shimmering(&self) -> bool {
        matches!(self, Self::Active { .. } | Self::PendingDuration { .. })
    }

    pub fn is_visible(&self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone)]
pub struct ActivitySnapshot {
    pub activity: LiveActivity,
    pub expires_at: Instant,
}
