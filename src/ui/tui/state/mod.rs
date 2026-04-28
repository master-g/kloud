//! TUI-local state and animation helpers.
#![allow(missing_docs)]

pub mod anim_state;
pub mod app_state;
pub mod input_state;
pub mod scroll_state;
pub mod theme_state;

use std::time::{Duration, Instant};

pub use crate::agent::message::{
    ActivityAccent, ActivityEntry, ActivityEntryKind, DisplayBlock, DisplayMessage, LiveActivity,
    MessageLevel, MessageType, SpinnerMode, ToolStatus,
};
pub use crate::agent::view::{
    AssistantStatus, PendingPermissionView, PendingToast, Screen, SessionView,
};
use crate::llm::response::StopReason;
use crate::ui::tui::constants::{
    ACTIVITY_SNAPSHOT_MS, MAX_ACTIVITY_ITEMS, THINKING_DURATION_SHOW_MS, THINKING_MIN_DISPLAY_MS,
};
pub use anim_state::{ActivityClock, ActivitySnapshot, StalledState, ThinkingStatus, TokenCounter};
pub use app_state::AppState;
pub use input_state::{AutocompleteState, CommandHint, InputState, SearchState};
pub use scroll_state::ScrollState;

const MAX_MESSAGES_TO_SHOW_IN_TRANSCRIPT_MODE: usize = 30;

#[derive(Debug, Clone)]
pub struct TranscriptSnapshot {
    pub view: SessionView,
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
    pub app: AppState,
    pub transcript_show_all: bool,
    pub transcript_hidden_message_count: usize,
    pub workspace: String,
    pub branch: String,
    pub effort: String,
    pub tool_count: usize,
    pub instruction_files: Vec<String>,
    pub hook_count: usize,
    pub input: InputState,
    pub scroll: ScrollState,
    pub theme: theme_state::ThemeState,
    pub status: AssistantStatus,
    pub model: String,
    pub max_context_tokens: u32,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub should_quit: bool,
    pub last_stop_reason: Option<StopReason>,
    pub activity_clock: ActivityClock,
    pub stalled_state: StalledState,
    pub token_counter: TokenCounter,
    pub thinking_status: ThinkingStatus,
    pub response_char_count: usize,
    pub session_cost: f64,
    pub session_duration: Duration,
    /// Tracks message count to detect new system messages for toast creation.
    last_seen_message_count: usize,
}

impl TuiState {
    pub fn input_height(&self, max_height: u16) -> u16 {
        let lines = self.input.text_area.line_count() as u16;
        // clamp input to at most half the terminal, add 2 border rows, floor at 3
        lines.clamp(1, max_height / 2).saturating_add(2).max(3)
    }

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
        theme: crate::ui::tui::theme::Theme,
    ) -> Self {
        Self {
            latest_view: SessionView::default(),
            transcript_snapshot: None,
            app: AppState::default(),
            transcript_show_all: false,
            transcript_hidden_message_count: 0,
            workspace,
            branch,
            effort,
            tool_count,
            instruction_files,
            hook_count,
            input: InputState::new(),
            scroll: ScrollState::new(),
            theme: theme_state::ThemeState::new(theme),
            status: AssistantStatus::Idle,
            model,
            max_context_tokens,
            input_tokens: 0,
            output_tokens: 0,
            should_quit: false,
            last_stop_reason: None,
            activity_clock: ActivityClock::new(),
            stalled_state: StalledState::new(),
            token_counter: TokenCounter::new(),
            thinking_status: ThinkingStatus::None,
            response_char_count: 0,
            session_cost: 0.0,
            session_duration: Duration::ZERO,
            last_seen_message_count: 0,
        }
    }

    pub fn apply_view(&mut self, view: SessionView) {
        let previous_status = self.status;
        let previous_live_activity = self.app.live_activity.clone();
        let previous_mode = previous_live_activity.as_ref().map(|activity| activity.mode);
        let was_in_transcript = self.latest_view.screen == Screen::Transcript;
        let enters_transcript = !was_in_transcript && view.screen == Screen::Transcript;
        let exits_transcript = was_in_transcript && view.screen != Screen::Transcript;

        if enters_transcript {
            self.transcript_snapshot = Some(TranscriptSnapshot {
                view: view.clone(),
            });
            self.scroll.reset();
        } else if exits_transcript {
            self.transcript_snapshot = None;
            self.scroll.reset();
        } else if view.screen == Screen::Transcript
            && let Some(snapshot) = self.transcript_snapshot.as_mut()
        {
            snapshot.view.transcript_show_all = view.transcript_show_all;
            snapshot.view.pending_permission = view.pending_permission.clone();
        }

        self.latest_view = view;
        self.sync_from_active_view();

        if previous_status != AssistantStatus::Streaming
            && self.status == AssistantStatus::Streaming
        {
            self.stalled_state.reset();
        }

        if let Some(mode) = self.app.live_activity.as_ref().map(|activity| activity.mode) {
            if previous_mode != Some(mode) {
                self.transition_thinking_status(mode);
            }
        } else if previous_mode == Some(SpinnerMode::Thinking) {
            self.transition_thinking_status(SpinnerMode::Responding);
        }

        if previous_live_activity.is_some()
            && self.app.live_activity.is_none()
            && previous_status != AssistantStatus::Idle
            && self.status == AssistantStatus::Idle
        {
            self.app.activity_snapshot = previous_live_activity.map(|activity| ActivitySnapshot {
                activity,
                expires_at: Instant::now() + Duration::from_millis(ACTIVITY_SNAPSHOT_MS),
            });
        }

        if self.app.recent_activity.len() > MAX_ACTIVITY_ITEMS {
            let overflow = self.app.recent_activity.len() - MAX_ACTIVITY_ITEMS;
            self.app.recent_activity.drain(0..overflow);
        }
    }

    fn sync_from_active_view(&mut self) {
        let active_view = self.active_view().clone();
        // Preserve Search screen — the server only sends Prompt/Transcript.
        if self.app.screen != Screen::Search {
            self.app.screen = active_view.screen;
        }
        self.transcript_show_all = active_view.transcript_show_all;
        self.app.pending_permission = active_view.pending_permission;
        self.status = active_view.status;
        self.input_tokens = active_view.input_tokens;
        self.output_tokens = active_view.output_tokens;
        self.app.active_tools = active_view.active_tools;
        self.last_stop_reason = active_view.last_stop_reason;
        self.app.recent_activity = active_view.recent_activity;
        self.app.live_activity = active_view.live_activity;
        self.response_char_count = active_view.response_char_count;

        // Expire old notifications
        let now = Instant::now();
        self.app.notifications.retain(|n| now.duration_since(n.created_at) < n.dismiss_after);
        self.session_cost = active_view.session_cost;
        self.session_duration = active_view.session_duration;

        if self.app.screen == Screen::Transcript && !self.transcript_show_all {
            self.transcript_hidden_message_count =
                active_view.messages.len().saturating_sub(MAX_MESSAGES_TO_SHOW_IN_TRANSCRIPT_MODE);
            if self.transcript_hidden_message_count > 0 {
                self.app.messages = active_view
                    .messages
                    .into_iter()
                    .skip(self.transcript_hidden_message_count)
                    .collect();
                return;
            }
        }

        self.transcript_hidden_message_count = 0;
        self.app.messages = active_view.messages;

        // Create notification toasts from pending_toasts in the view.
        for toast in &active_view.pending_toasts {
            self.app.notifications.push(Notification {
                text: toast.text.clone(),
                level: toast.level,
                created_at: now,
                dismiss_after: Duration::from_secs(3),
            });
        }
        let new_count = self.app.messages.len();
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
        if self.app.screen == Screen::Prompt {
            self.status = AssistantStatus::Cancelling;
        }
    }

    /// Scroll messages up by n lines. Pauses auto-scroll.
    pub fn scroll_messages_up(&mut self, n: u16) {
        self.scroll.scroll_up(n as usize);
    }

    /// Scroll messages down without content-height clamping.
    /// The render loop handles clamping to `max_scroll` and re-enabling auto-scroll.
    pub fn scroll_messages_down(&mut self, n: u16) {
        self.scroll.scroll_down_by(n as usize);
    }

    /// Toggle the collapsed state for a tool block at the given message index.
    pub fn toggle_tool_collapse(&mut self, msg_index: usize) {
        if self.app.collapsed_tools.remove(&msg_index).is_some() {
            // Was collapsed, now expanded
        } else {
            self.app.collapsed_tools.insert(msg_index, true);
        }
    }

    pub fn take_input(&mut self) -> Option<String> {
        self.input.text_area.take()
    }

    pub fn current_command_hint(&self) -> Option<&CommandHint> {
        let text = self.input.text_area.text();
        if !text.starts_with('/') {
            return None;
        }
        let rest = &text[1..];
        let cmd = rest.split_whitespace().next().unwrap_or("");
        input_state::SLASH_COMMAND_HINTS.iter().find(|hint| hint.name == cmd)
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
        self.app
            .live_activity
            .as_ref()
            .or_else(|| self.app.activity_snapshot.as_ref().map(|snapshot| &snapshot.activity))
    }

    pub fn tick_activity_snapshot(&mut self) {
        if self
            .app
            .activity_snapshot
            .as_ref()
            .is_some_and(|snapshot| Instant::now() >= snapshot.expires_at)
        {
            self.app.activity_snapshot = None;
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
                self.app.messages.len(),
                self.app.messages.len() + self.transcript_hidden_message_count,
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
            crate::ui::tui::theme::Theme::from_scheme(
                crate::ui::tui::theme::ColorScheme::Dark,
                false,
            ),
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

        assert_eq!(state.app.messages.len(), 1);
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

        assert!(state.app.activity_snapshot.is_some());
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
        assert_eq!(state.app.messages.len(), 1);

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
        assert_eq!(state.app.messages.len(), 1);

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
        assert_eq!(state.app.messages.len(), 2);
    }

    #[test]
    fn toggle_tool_collapse_flips_state() {
        let mut state = test_state();
        assert!(state.app.collapsed_tools.is_empty());

        state.toggle_tool_collapse(0);
        assert!(state.app.collapsed_tools.contains_key(&0));

        state.toggle_tool_collapse(0);
        assert!(!state.app.collapsed_tools.contains_key(&0));
    }

    #[test]
    fn toggle_tool_collapse_independent_indices() {
        let mut state = test_state();
        state.toggle_tool_collapse(1);
        state.toggle_tool_collapse(3);
        assert!(state.app.collapsed_tools.contains_key(&1));
        assert!(state.app.collapsed_tools.contains_key(&3));
        assert!(!state.app.collapsed_tools.contains_key(&2));

        state.toggle_tool_collapse(1);
        assert!(!state.app.collapsed_tools.contains_key(&1));
        assert!(state.app.collapsed_tools.contains_key(&3));
    }
}
