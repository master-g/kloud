//! Reducer inputs for the session store.
#![allow(missing_docs)]

use crate::llm::response::StopReason;

use super::message::{DisplayBlock, MessageLevel};
use super::view::{PendingPermissionView, Screen};

/// Events emitted by the session runtime and applied to the store.
#[derive(Debug, Clone)]
pub enum SessionEvent {
	UserMessageSubmitted {
		text: String,
	},
	QueryStarted,
	QueryCancelling,
	QueryCompleted {
		stop_reason: Option<StopReason>,
	},
	InterruptRecorded {
		content: String,
	},
	AssistantMessageStarted,
	AssistantTextDelta {
		text: String,
	},
	AssistantThinkingDelta {
		text: String,
	},
	AssistantRedactedThinking {
		text: String,
	},
	AssistantToolUseStarted {
		id: String,
		name: String,
		server_name: Option<String>,
		input: serde_json::Value,
	},
	AssistantToolUseInputJsonDelta {
		id: String,
		partial_json: String,
	},
	AssistantMessageCommitted {
		blocks: Vec<DisplayBlock>,
		stop_reason: StopReason,
	},
	ToolExecutionStarted {
		id: String,
		name: String,
		server_name: Option<String>,
	},
	ToolExecutionFinished {
		id: String,
		name: String,
		server_name: Option<String>,
		output: String,
		is_error: bool,
	},
	SystemMessageAdded {
		content: String,
		level: MessageLevel,
	},
	UsageUpdated {
		input_tokens: u32,
		output_tokens: u32,
	},
	ScreenChanged {
		screen: Screen,
	},
	TranscriptShowAllChanged {
		show_all: bool,
	},
	PendingPermissionChanged {
		pending_permission: Option<PendingPermissionView>,
	},
}
