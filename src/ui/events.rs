//! App ↔ UI communication protocol.
//!
//! [`AppEvent`] flows from the application session to the UI backend.
//! [`UiAction`] flows from the UI backend back to the session.

use crate::llm::response::StopReason;

/// The type of a completed content block.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum BlockType {
	Text,
	Thinking,
	ToolUse,
	ToolResult,
}

/// Events sent **from** the application session **to** the UI.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum AppEvent {
	/// A new assistant turn has begun (clear any "waiting" indicator).
	AssistantTurnStart,
	/// Incremental text from the assistant.
	TextDelta(String),
	/// Incremental thinking text from the assistant.
	ThinkingDelta(String),
	/// Redacted thinking content from the assistant.
	RedactedThinking(String),
	/// A content block is complete.
	BlockComplete {
		index: u32,
		block_type: BlockType,
	},
	/// The assistant turn finished.
	AssistantTurnEnd {
		stop_reason: StopReason,
	},
	/// A tool invocation has started (future M2+).
	ToolUseStart {
		id: String,
		name: String,
		server_name: Option<String>,
		input_preview: String,
	},
	/// A tool returned a result (future M2+).
	ToolResult {
		id: String,
		name: String,
		server_name: Option<String>,
		output: String,
		is_error: bool,
	},
	/// An error occurred (shorthand for `SystemMessage` with error level).
	Error(String),
	/// A system-level notification (info, warning, or error).
	SystemMessage {
		content: String,
		level: crate::ui::tui::state::MessageLevel,
	},
	/// Token usage report for the status bar.
	UsageReport {
		input_tokens: u32,
		output_tokens: u32,
	},
	/// The session is shutting down; the UI should exit its event loop.
	Shutdown,
}

/// Actions sent **from** the UI **to** the application session.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum UiAction {
	/// User submitted a message.
	SendMessage(String),
	/// User requested cancellation of the current turn.
	CancelTurn,
	/// User wants to exit the application.
	Exit,
	/// User entered a slash command.
	SlashCommand {
		command: String,
		args: String,
	},
}
