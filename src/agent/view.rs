//! Session view model consumed by UI backends.
#![allow(missing_docs)]

use std::time::Duration;

use crate::llm::response::StopReason;

use super::message::{ActivityEntry, DisplayMessage, LiveActivity, MessageLevel};

/// Current high-level query status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssistantStatus {
    Idle,
    Streaming,
    Cancelling,
}

/// Current top-level screen. Only `Prompt` is rendered today; `Transcript`
/// exists so the state model can grow without reshaping the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Prompt,
    Transcript,
    Search,
}

/// Minimal permission-prompt skeleton exposed by the session store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingPermissionView {
    pub label: String,
    pub description: String,
}

/// A toast notification pending pickup by the TUI.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingToast {
    pub text: String,
    pub level: MessageLevel,
}

/// Full state snapshot consumed by UI backends.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionView {
    pub revision: u64,
    pub messages: Vec<DisplayMessage>,
    pub status: AssistantStatus,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub active_tools: Vec<String>,
    pub last_stop_reason: Option<StopReason>,
    pub recent_activity: Vec<ActivityEntry>,
    pub live_activity: Option<LiveActivity>,
    pub response_char_count: usize,
    pub screen: Screen,
    pub transcript_show_all: bool,
    pub pending_permission: Option<PendingPermissionView>,
    pub session_cost: f64,
    pub session_duration: Duration,
    pub pending_toasts: Vec<PendingToast>,
}

impl Default for SessionView {
    fn default() -> Self {
        Self {
            revision: 0,
            messages: Vec::new(),
            status: AssistantStatus::Idle,
            input_tokens: 0,
            output_tokens: 0,
            active_tools: Vec::new(),
            last_stop_reason: None,
            recent_activity: Vec::new(),
            live_activity: None,
            response_char_count: 0,
            screen: Screen::Prompt,
            transcript_show_all: false,
            pending_permission: None,
            session_cost: 0.0,
            session_duration: Duration::ZERO,
            pending_toasts: Vec::new(),
        }
    }
}
