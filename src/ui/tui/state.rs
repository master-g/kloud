//! TUI-local state and animation helpers.
#![allow(missing_docs)]

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

pub use crate::agent::message::{
    ActivityAccent, ActivityEntry, ActivityEntryKind, DisplayBlock, DisplayMessage, LiveActivity,
    MessageLevel, MessageType, SpinnerMode, ToolStatus,
};
pub use crate::agent::view::{AssistantStatus, PendingPermissionView, PendingToast, Screen, SessionView};
use crate::llm::response::StopReason;
use crate::ui::constants::{
    ACTIVITY_SNAPSHOT_MS, MAX_ACTIVITY_ITEMS, THINKING_DURATION_SHOW_MS, THINKING_MIN_DISPLAY_MS,
};
use crate::ui::tui::text_area::TextArea;
use crate::ui::tui::virtual_scroll::VirtualScroll;

const MAX_MESSAGES_TO_SHOW_IN_TRANSCRIPT_MODE: usize = 30;

// ============================================================================
// Animation Clock & Activity State
// ============================================================================

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

// ============================================================================
// Stalled State
// ============================================================================

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
}

impl Default for StalledState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Token Counter
// ============================================================================

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

// ============================================================================
// Thinking Status
// ============================================================================

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

// ============================================================================
// TUI State
// ============================================================================

#[derive(Debug, Clone)]
pub struct CommandHint {
    pub name: &'static str,
    pub summary: &'static str,
}

const SLASH_COMMAND_HINTS: &[CommandHint] = &[
    CommandHint {
        name: "help",
        summary: "Show available commands",
    },
    CommandHint {
        name: "exit",
        summary: "Quit kloud",
    },
    CommandHint {
        name: "quit",
        summary: "Alias for /exit",
    },
];

/// Autocomplete state for slash commands.
#[derive(Debug, Clone)]
pub struct AutocompleteState {
    pub visible: bool,
    pub items: Vec<(String, String)>,
    pub selected: usize,
    pub filter: String,
}

impl Default for AutocompleteState {
    fn default() -> Self {
        Self {
            visible: false,
            items: SLASH_COMMAND_HINTS
                .iter()
                .map(|h| (h.name.to_string(), h.summary.to_string()))
                .collect(),
            selected: 0,
            filter: String::new(),
        }
    }
}

impl AutocompleteState {
    pub fn update_filter(&mut self, filter: &str) {
        self.filter = filter.to_string();
        self.items = SLASH_COMMAND_HINTS
            .iter()
            .filter(|h| h.name.contains(filter))
            .map(|h| (h.name.to_string(), h.summary.to_string()))
            .collect();
        if self.selected >= self.items.len() {
            self.selected = 0;
        }
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn prev(&mut self) {
        if !self.items.is_empty() {
            self.selected = if self.selected == 0 {
                self.items.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn completion(&self) -> Option<&str> {
        self.items.get(self.selected).map(|(name, _)| name.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct ActivitySnapshot {
    pub activity: LiveActivity,
    pub expires_at: Instant,
}

#[derive(Debug, Clone)]
pub struct TranscriptSnapshot {
    pub view: SessionView,
}

/// Search state for the message area.
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Current search query.
    pub query: String,
    /// Indices of messages that match the query.
    pub matches: Vec<usize>,
    /// Index into `matches` for the current highlighted match.
    pub current_match: usize,
    /// Whether search has wrapped around.
    pub wrapped: bool,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            matches: Vec::new(),
            current_match: 0,
            wrapped: false,
        }
    }

    /// Execute case-insensitive substring search across messages.
    pub fn execute(&mut self, query: &str, messages: &[DisplayMessage]) {
        self.query = query.to_string();
        if query.is_empty() {
            self.matches.clear();
            self.current_match = 0;
            self.wrapped = false;
            return;
        }
        let lower = query.to_lowercase();
        self.matches = messages
            .iter()
            .enumerate()
            .filter(|(_, msg)| {
                msg.blocks.iter().any(|b| match b {
                    DisplayBlock::Text(t)
                    | DisplayBlock::Thinking(t)
                    | DisplayBlock::RedactedThinking(t) => t.to_lowercase().contains(&lower),
                    DisplayBlock::ToolUse {
                        input_preview,
                        ..
                    } => input_preview.to_lowercase().contains(&lower),
                    DisplayBlock::ToolResult {
                        output,
                        ..
                    } => output.to_lowercase().contains(&lower),
                })
            })
            .map(|(i, _)| i)
            .collect();
        self.current_match = 0;
        self.wrapped = false;
    }

    /// Move to the next match. Sets `wrapped` if wrapping around.
    pub fn next(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.current_match += 1;
        if self.current_match >= self.matches.len() {
            self.current_match = 0;
            self.wrapped = true;
        } else {
            self.wrapped = false;
        }
    }

    /// Move to the previous match. Sets `wrapped` if wrapping around.
    pub fn prev(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        if self.current_match == 0 {
            self.current_match = self.matches.len() - 1;
            self.wrapped = true;
        } else {
            self.current_match -= 1;
            self.wrapped = false;
        }
    }

    /// Return the message index of the current match, if any.
    pub fn current_message_index(&self) -> Option<usize> {
        self.matches.get(self.current_match).copied()
    }

    /// Return match count display string like "[3/12]".
    pub fn match_display(&self) -> String {
        if self.matches.is_empty() {
            "[0/0]".to_string()
        } else {
            format!("[{}/{}]", self.current_match + 1, self.matches.len())
        }
    }
}

/// A notification toast.
#[derive(Debug, Clone)]
pub struct Notification {
    pub text: String,
    pub level: MessageLevel,
    pub created_at: Instant,
    pub dismiss_after: Duration,
}

/// All mutable state the TUI needs for rendering.
pub struct TuiState {
    pub latest_view: SessionView,
    pub transcript_snapshot: Option<TranscriptSnapshot>,
    pub messages: Vec<DisplayMessage>,
    pub screen: Screen,
    pub transcript_show_all: bool,
    pub transcript_hidden_message_count: usize,
    pub pending_permission: Option<PendingPermissionView>,
    pub workspace: String,
    pub branch: String,
    pub effort: String,
    pub tool_count: usize,
    pub instruction_files: Vec<String>,
    pub hook_count: usize,
    pub text_area: TextArea,
    pub scroll: u16,
    pub status: AssistantStatus,
    pub model: String,
    pub max_context_tokens: u32,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub should_quit: bool,
    pub active_tools: Vec<String>,
    pub last_stop_reason: Option<StopReason>,
    pub history: Vec<String>,
    pub recent_activity: Vec<ActivityEntry>,
    pub history_index: Option<usize>,
    pub live_activity: Option<LiveActivity>,
    pub activity_snapshot: Option<ActivitySnapshot>,
    pub activity_clock: ActivityClock,
    pub stalled_state: StalledState,
    pub token_counter: TokenCounter,
    pub thinking_status: ThinkingStatus,
    pub response_char_count: usize,
    pub session_cost: f64,
    pub session_duration: Duration,
    /// Whether the user has scrolled up during streaming, pausing auto-scroll.
    pub auto_scroll_paused: bool,
    /// Virtual scroll state for efficient rendering of large message lists.
    pub virtual_scroll: VirtualScroll,
    /// Collapsed tool output blocks: message index -> true (collapsed).
    pub collapsed_tools: BTreeMap<usize, bool>,
    /// Search state (active when screen is Search).
    pub search: SearchState,
    pub autocomplete: AutocompleteState,
    pub tool_start_times: std::collections::HashMap<String, Instant>,
    pub notifications: Vec<Notification>,
    /// Tracks message count to detect new system messages for toast creation.
    last_seen_message_count: usize,
}

impl TuiState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        model: String,
        max_context_tokens: u32,
        workspace: String,
        branch: String,
        effort: String,
        tool_count: usize,
        instruction_files: Vec<String>,
        hook_count: usize,
    ) -> Self {
        Self {
            latest_view: SessionView::default(),
            transcript_snapshot: None,
            messages: Vec::new(),
            screen: Screen::Prompt,
            transcript_show_all: false,
            transcript_hidden_message_count: 0,
            pending_permission: None,
            workspace,
            branch,
            effort,
            tool_count,
            instruction_files,
            hook_count,
            text_area: TextArea::new(),
            scroll: 0,
            status: AssistantStatus::Idle,
            model,
            max_context_tokens,
            input_tokens: 0,
            output_tokens: 0,
            should_quit: false,
            active_tools: Vec::new(),
            last_stop_reason: None,
            history: Vec::new(),
            recent_activity: Vec::new(),
            history_index: None,
            live_activity: None,
            activity_snapshot: None,
            activity_clock: ActivityClock::new(),
            stalled_state: StalledState::new(),
            token_counter: TokenCounter::new(),
            thinking_status: ThinkingStatus::None,
            response_char_count: 0,
            session_cost: 0.0,
            session_duration: Duration::ZERO,
            auto_scroll_paused: false,
            virtual_scroll: VirtualScroll::new(),
            collapsed_tools: BTreeMap::new(),
            search: SearchState::new(),
            autocomplete: AutocompleteState::default(),
            tool_start_times: std::collections::HashMap::new(),
            notifications: Vec::new(),
            last_seen_message_count: 0,
        }
    }

    pub fn apply_view(&mut self, view: SessionView) {
        let previous_status = self.status;
        let previous_live_activity = self.live_activity.clone();
        let previous_mode = previous_live_activity.as_ref().map(|activity| activity.mode);
        let was_in_transcript = self.latest_view.screen == Screen::Transcript;
        let enters_transcript = !was_in_transcript && view.screen == Screen::Transcript;
        let exits_transcript = was_in_transcript && view.screen != Screen::Transcript;

        if enters_transcript {
            self.transcript_snapshot = Some(TranscriptSnapshot {
                view: view.clone(),
            });
            self.scroll = 0;
        } else if exits_transcript {
            self.transcript_snapshot = None;
            self.scroll = 0;
        } else if view.screen == Screen::Transcript
            && let Some(snapshot) = self.transcript_snapshot.as_mut()
        {
            snapshot.view.transcript_show_all = view.transcript_show_all;
            snapshot.view.pending_permission = view.pending_permission.clone();
        }

        self.latest_view = view;
        self.sync_from_active_view();

        if let Some(mode) = self.live_activity.as_ref().map(|activity| activity.mode) {
            if previous_mode != Some(mode) {
                self.transition_thinking_status(mode);
            }
        } else if previous_mode == Some(SpinnerMode::Thinking) {
            self.transition_thinking_status(SpinnerMode::Responding);
        }

        if previous_live_activity.is_some()
            && self.live_activity.is_none()
            && previous_status != AssistantStatus::Idle
            && self.status == AssistantStatus::Idle
        {
            self.activity_snapshot = previous_live_activity.map(|activity| ActivitySnapshot {
                activity,
                expires_at: Instant::now() + Duration::from_millis(ACTIVITY_SNAPSHOT_MS),
            });
        }

        if self.recent_activity.len() > MAX_ACTIVITY_ITEMS {
            let overflow = self.recent_activity.len() - MAX_ACTIVITY_ITEMS;
            self.recent_activity.drain(0..overflow);
        }
    }

    fn sync_from_active_view(&mut self) {
        let active_view = self.active_view().clone();
        // Preserve Search screen — the server only sends Prompt/Transcript.
        if self.screen != Screen::Search {
            self.screen = active_view.screen;
        }
        self.transcript_show_all = active_view.transcript_show_all;
        self.pending_permission = active_view.pending_permission;
        self.status = active_view.status;
        self.input_tokens = active_view.input_tokens;
        self.output_tokens = active_view.output_tokens;
        self.active_tools = active_view.active_tools;
        self.last_stop_reason = active_view.last_stop_reason;
        self.recent_activity = active_view.recent_activity;
        self.live_activity = active_view.live_activity;
        self.response_char_count = active_view.response_char_count;

        // Expire old notifications
        let now = Instant::now();
        self.notifications.retain(|n| now.duration_since(n.created_at) < n.dismiss_after);
        self.session_cost = active_view.session_cost;
        self.session_duration = active_view.session_duration;

        if self.screen == Screen::Transcript && !self.transcript_show_all {
            self.transcript_hidden_message_count =
                active_view.messages.len().saturating_sub(MAX_MESSAGES_TO_SHOW_IN_TRANSCRIPT_MODE);
            if self.transcript_hidden_message_count > 0 {
                self.messages = active_view
                    .messages
                    .into_iter()
                    .skip(self.transcript_hidden_message_count)
                    .collect();
                return;
            }
        }

        self.transcript_hidden_message_count = 0;
        self.messages = active_view.messages;

        // Create notification toasts from pending_toasts in the view.
        let now = Instant::now();
        for toast in &active_view.pending_toasts {
            self.notifications.push(Notification {
                text: toast.text.clone(),
                level: toast.level,
                created_at: now,
                dismiss_after: Duration::from_secs(3),
            });
        }
        let new_count = self.messages.len();
        self.last_seen_message_count = self.last_seen_message_count.min(new_count);
        self.last_seen_message_count = new_count;
    }

    fn active_view(&self) -> &SessionView {
        if self.latest_view.screen == Screen::Transcript
            && let Some(snapshot) = self.transcript_snapshot.as_ref()
        {
            return &snapshot.view;
        }
        &self.latest_view
    }

    pub fn begin_cancel(&mut self) {
        if self.screen == Screen::Prompt {
            self.status = AssistantStatus::Cancelling;
        }
    }

    /// Scroll messages up by n lines. Pauses auto-scroll.
    pub fn scroll_messages_up(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_add(n);
        self.auto_scroll_paused = true;
    }

    /// Scroll messages down. Re-enables auto-scroll when reaching the bottom.
    pub fn scroll_messages_down(&mut self, n: u16, content_height: u16, visible_height: u16) {
        self.scroll = self.scroll.saturating_add(n);
        let max_scroll = content_height.saturating_sub(visible_height);
        if self.scroll >= max_scroll {
            self.scroll = max_scroll;
            self.auto_scroll_paused = false;
        }
    }

    /// Toggle the collapsed state for a tool block at the given message index.
    pub fn toggle_tool_collapse(&mut self, msg_index: usize) {
        if self.collapsed_tools.remove(&msg_index).is_some() {
            // Was collapsed, now expanded
        } else {
            self.collapsed_tools.insert(msg_index, true);
        }
    }

    pub fn take_input(&mut self) -> Option<String> {
        self.text_area.take()
    }

    pub fn current_command_hint(&self) -> Option<&CommandHint> {
        let text = self.text_area.text();
        if !text.starts_with('/') {
            return None;
        }
        let rest = &text[1..];
        let cmd = rest.split_whitespace().next().unwrap_or("");
        SLASH_COMMAND_HINTS.iter().find(|hint| hint.name == cmd)
    }

    pub fn effort_suffix(&self) -> String {
        if self.effort.is_empty() || self.effort == "default effort" {
            String::new()
        } else {
            format!(" with {}", self.effort)
        }
    }

    /// Extract the workspace directory name for display.
    pub fn workspace_name(&self) -> String {
        std::path::Path::new(&self.workspace)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&self.workspace)
            .to_string()
    }

    pub fn transition_thinking_status(&mut self, new_mode: SpinnerMode) {
        let now = Instant::now();
        match (&self.thinking_status, new_mode) {
            (
                ThinkingStatus::None
                | ThinkingStatus::PendingDuration {
                    ..
                }
                | ThinkingStatus::ShowDuration {
                    ..
                },
                SpinnerMode::Thinking,
            ) => {
                self.thinking_status = ThinkingStatus::Active {
                    started_at: now,
                };
            }
            (
                ThinkingStatus::Active {
                    ..
                },
                SpinnerMode::Thinking,
            ) => {}
            (
                ThinkingStatus::Active {
                    started_at,
                },
                _,
            ) => {
                let duration_ms = now.duration_since(*started_at).as_millis() as u64;
                let remaining = THINKING_MIN_DISPLAY_MS.saturating_sub(duration_ms);
                if remaining > 0 {
                    self.thinking_status = ThinkingStatus::PendingDuration {
                        duration_ms,
                        min_display_until: now + Duration::from_millis(remaining),
                    };
                } else {
                    self.thinking_status = ThinkingStatus::ShowDuration {
                        duration_ms,
                        clear_at: now + Duration::from_millis(THINKING_DURATION_SHOW_MS),
                    };
                }
            }
            _ => {}
        }
    }

    pub fn tick_thinking_status(&mut self) {
        let now = Instant::now();
        match &self.thinking_status {
            ThinkingStatus::PendingDuration {
                duration_ms,
                min_display_until,
            } if now >= *min_display_until => {
                self.thinking_status = ThinkingStatus::ShowDuration {
                    duration_ms: *duration_ms,
                    clear_at: now + Duration::from_millis(THINKING_DURATION_SHOW_MS),
                };
            }
            ThinkingStatus::ShowDuration {
                clear_at,
                ..
            } if now >= *clear_at => {
                self.thinking_status = ThinkingStatus::None;
            }
            _ => {}
        }
    }

    pub fn displayed_activity(&self) -> Option<&LiveActivity> {
        self.live_activity
            .as_ref()
            .or_else(|| self.activity_snapshot.as_ref().map(|snapshot| &snapshot.activity))
    }

    pub fn tick_activity_snapshot(&mut self) {
        if self
            .activity_snapshot
            .as_ref()
            .is_some_and(|snapshot| Instant::now() >= snapshot.expires_at)
        {
            self.activity_snapshot = None;
        }
    }

    pub fn transcript_toggle_label(&self) -> &'static str {
        if self.transcript_show_all {
            "collapse"
        } else {
            "show all"
        }
    }

    pub fn transcript_status_text(&self) -> String {
        if self.transcript_hidden_message_count > 0 && !self.transcript_show_all {
            format!(
                "Showing detailed transcript · Ctrl+O to toggle · Ctrl+E to {} · last {} of {} messages",
                self.transcript_toggle_label(),
                self.messages.len(),
                self.messages.len() + self.transcript_hidden_message_count,
            )
        } else {
            format!(
                "Showing detailed transcript · Ctrl+O to toggle · Ctrl+E to {} · ↑↓/PgUp/PgDn scroll",
                self.transcript_toggle_label(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{ActivityEntryKind, SessionView};
    use std::time::Instant;

    fn test_state() -> TuiState {
        TuiState::new(
            "claude-test".to_string(),
            200_000,
            "/tmp/workspace".to_string(),
            "main".to_string(),
            "default effort".to_string(),
            2,
            Vec::new(),
            0,
        )
    }

    #[test]
    fn apply_view_copies_session_fields() {
        let mut state = test_state();
        state.apply_view(SessionView {
            messages: vec![DisplayMessage {
                id: 1,
                message_type: MessageType::Assistant,
                blocks: vec![DisplayBlock::Text("hello".to_string())],
            }],
            status: AssistantStatus::Streaming,
            input_tokens: 12,
            output_tokens: 34,
            active_tools: vec!["read".to_string()],
            last_stop_reason: Some(StopReason::EndTurn),
            recent_activity: vec![ActivityEntry {
                kind: ActivityEntryKind::Assistant,
                text: "Assistant started a turn".to_string(),
            }],
            live_activity: Some(LiveActivity {
                message: "Thinking…".to_string(),
                accent: ActivityAccent::Info,
                mode: SpinnerMode::Responding,
                started_at: Instant::now(),
                last_signal_at: Instant::now(),
            }),
            response_char_count: 5,
            ..SessionView::default()
        });

        assert_eq!(state.messages.len(), 1);
        assert_eq!(state.status, AssistantStatus::Streaming);
        assert_eq!(state.input_tokens, 12);
        assert_eq!(state.output_tokens, 34);
        assert_eq!(state.response_char_count, 5);
    }

    #[test]
    fn end_of_query_keeps_activity_snapshot_briefly_visible() {
        let mut state = test_state();
        state.apply_view(SessionView {
            status: AssistantStatus::Streaming,
            live_activity: Some(LiveActivity {
                message: "Working…".to_string(),
                accent: ActivityAccent::Info,
                mode: SpinnerMode::Responding,
                started_at: Instant::now(),
                last_signal_at: Instant::now(),
            }),
            ..SessionView::default()
        });

        state.apply_view(SessionView::default());

        assert!(state.activity_snapshot.is_some());
    }

    #[test]
    fn transcript_mode_freezes_messages_until_exit() {
        let mut state = test_state();
        let prompt_view = SessionView {
            messages: vec![DisplayMessage {
                id: 1,
                message_type: MessageType::Assistant,
                blocks: vec![DisplayBlock::Text("before".into())],
            }],
            ..SessionView::default()
        };
        state.apply_view(prompt_view.clone());

        state.apply_view(SessionView {
            screen: Screen::Transcript,
            messages: prompt_view.messages.clone(),
            ..SessionView::default()
        });
        assert_eq!(state.messages.len(), 1);

        state.apply_view(SessionView {
            screen: Screen::Transcript,
            messages: vec![
                DisplayMessage {
                    id: 1,
                    message_type: MessageType::Assistant,
                    blocks: vec![DisplayBlock::Text("before".into())],
                },
                DisplayMessage {
                    id: 2,
                    message_type: MessageType::Assistant,
                    blocks: vec![DisplayBlock::Text("after".into())],
                },
            ],
            ..SessionView::default()
        });
        assert_eq!(state.messages.len(), 1);

        state.apply_view(SessionView {
            screen: Screen::Prompt,
            messages: vec![
                DisplayMessage {
                    id: 1,
                    message_type: MessageType::Assistant,
                    blocks: vec![DisplayBlock::Text("before".into())],
                },
                DisplayMessage {
                    id: 2,
                    message_type: MessageType::Assistant,
                    blocks: vec![DisplayBlock::Text("after".into())],
                },
            ],
            ..SessionView::default()
        });
        assert_eq!(state.messages.len(), 2);
    }

    #[test]
    fn toggle_tool_collapse_flips_state() {
        let mut state = test_state();
        assert!(state.collapsed_tools.is_empty());

        state.toggle_tool_collapse(0);
        assert!(state.collapsed_tools.contains_key(&0));

        state.toggle_tool_collapse(0);
        assert!(!state.collapsed_tools.contains_key(&0));
    }

    #[test]
    fn toggle_tool_collapse_independent_indices() {
        let mut state = test_state();
        state.toggle_tool_collapse(1);
        state.toggle_tool_collapse(3);
        assert!(state.collapsed_tools.contains_key(&1));
        assert!(state.collapsed_tools.contains_key(&3));
        assert!(!state.collapsed_tools.contains_key(&2));

        state.toggle_tool_collapse(1);
        assert!(!state.collapsed_tools.contains_key(&1));
        assert!(state.collapsed_tools.contains_key(&3));
    }
}
