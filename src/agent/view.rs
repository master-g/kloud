//! Session view model consumed by UI backends.
#![allow(missing_docs)]

use crate::llm::response::StopReason;

use super::message::{ActivityEntry, DisplayMessage, LiveActivity};

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
}

/// Minimal permission-prompt skeleton exposed by the session store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingPermissionView {
	pub label: String,
	pub description: String,
}

/// Full state snapshot consumed by UI backends.
#[derive(Debug, Clone, PartialEq, Eq)]
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
		}
	}
}
