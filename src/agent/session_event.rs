//! Reducer inputs for the session store.
#![allow(missing_docs)]

use crate::llm::response::StopReason;
use crate::tools::ToolResultKind;

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
        display_name: String,
        server_name: Option<String>,
        input: serde_json::Value,
        rendered_use: Vec<ratatui::text::Line<'static>>,
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
        display_name: String,
        server_name: Option<String>,
        rendered_use: Vec<ratatui::text::Line<'static>>,
    },
    ToolExecutionFinished {
        id: String,
        name: String,
        server_name: Option<String>,
        output: String,
        result_kind: ToolResultKind,
        rendered_result: Vec<ratatui::text::Line<'static>>,
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
    /// Emitted when no tokens have arrived for the stall threshold duration.
    StreamStalled,
    /// Emitted when tokens resume after a stall.
    StreamResumed,
    PendingPermissionChanged {
        pending_permission: Option<PendingPermissionView>,
    },
    ToolProgress {
        id: String,
        text: String,
    },
}
