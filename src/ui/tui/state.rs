//! TUI view state — the data model that the render functions draw from.
#![allow(missing_docs)]

use std::time::Instant;

use crate::llm::response::StopReason;

/// What the assistant is currently doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantStatus {
	/// Idle, waiting for user input.
	Idle,
	/// Streaming a response.
	Streaming,
	/// Cancellation requested; waiting for the turn to finish.
	Cancelling,
}

/// Status of a running tool block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStatus {
	Running,
	Done,
	Errored,
}

/// The kind of ephemeral activity currently shown in the activity bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityKind {
	Thinking,
	Tool {
		name: String,
	},
}

/// Short-lived work the assistant is currently performing.
#[derive(Debug, Clone)]
pub struct LiveActivity {
	pub kind: ActivityKind,
	pub started_at: Instant,
}

/// A single content block within a display message.
#[derive(Debug, Clone)]
pub enum DisplayBlock {
	/// Plain text content.
	Text(String),
	/// Thinking / chain-of-thought content.
	Thinking(String),
	/// Redacted thinking content.
	RedactedThinking(String),
	/// A tool invocation within the assistant turn.
	ToolUse {
		id: String,
		name: String,
		input_preview: String,
		status: ToolStatus,
	},
	/// Result of a previously-started tool.
	ToolResult {
		id: String,
		name: String,
		output: String,
		is_error: bool,
	},
}

/// A message shown in the messages area.
#[derive(Debug, Clone)]
pub struct DisplayMessage {
	/// "You" or "Assistant" or "Error".
	pub role: String,
	/// Content blocks in this message.
	pub blocks: Vec<DisplayBlock>,
}

/// Metadata for slash commands used for inline help in the status bar.
#[derive(Debug, Clone, Copy)]
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

const MAX_ACTIVITY_ITEMS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityEntryKind {
	User,
	Assistant,
	Tool,
	Success,
	Error,
	Meta,
}

#[derive(Debug, Clone)]
pub struct ActivityEntry {
	pub kind: ActivityEntryKind,
	pub text: String,
}

/// All mutable state the TUI needs for rendering.
pub struct TuiState {
	/// Conversation messages to display.
	pub messages: Vec<DisplayMessage>,
	/// Workspace path shown in the dashboard and title bar.
	pub workspace: String,
	/// Git branch shown in the dashboard and status bar.
	pub branch: String,
	/// Effort label shown in the dashboard and title bar.
	pub effort: String,
	/// Number of tools currently registered for the session.
	pub tool_count: usize,
	/// Instruction files detected for the current workspace.
	pub instruction_files: Vec<String>,
	/// Number of active git hooks in the current repository.
	pub hook_count: usize,
	/// The user's current input buffer.
	pub input: String,
	/// Cursor position within `input` (byte offset).
	pub cursor: usize,
	/// Vertical scroll offset for the messages area.
	pub scroll: u16,
	/// Current assistant status.
	pub status: AssistantStatus,
	/// Model name for the status bar.
	pub model: String,
	/// Maximum context window tokens for the current model.
	pub max_context_tokens: u32,
	/// Last reported token usage.
	pub input_tokens: u32,
	/// Last reported token usage.
	pub output_tokens: u32,
	/// Whether the UI should exit.
	pub should_quit: bool,
	/// Currently running tool names (for the status bar).
	pub active_tools: Vec<String>,
	/// Last assistant stop reason for the status bar.
	pub last_stop_reason: Option<StopReason>,
	/// Input history (messages and slash commands).
	pub history: Vec<String>,
	/// Recent session activity shown in the dashboard side panel.
	pub recent_activity: Vec<ActivityEntry>,
	/// Current history index when browsing with Up/Down.
	pub history_index: Option<usize>,
	/// Tick counter used for simple ASCII status animation.
	pub animation_tick: u64,
	/// Ephemeral activity line shown above the status bar.
	pub live_activity: Option<LiveActivity>,
}

impl TuiState {
	/// Create initial state for a given model name.
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
			messages: Vec::new(),
			workspace,
			branch,
			effort,
			tool_count,
			instruction_files,
			hook_count,
			input: String::new(),
			cursor: 0,
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
			animation_tick: 0,
			live_activity: None,
		}
	}

	/// Start a new assistant turn — append an empty assistant message.
	pub fn begin_assistant_turn(&mut self) {
		self.status = AssistantStatus::Streaming;
		self.live_activity = Some(LiveActivity {
			kind: ActivityKind::Thinking,
			started_at: Instant::now(),
		});
		self.messages.push(DisplayMessage {
			role: "Assistant".into(),
			blocks: vec![DisplayBlock::Text(String::new())],
		});
		self.record_activity(ActivityEntryKind::Assistant, "Assistant started a turn");
	}

	/// Mark the assistant as cancelling in response to a user request.
	pub fn begin_cancel(&mut self) {
		self.status = AssistantStatus::Cancelling;
		self.record_activity(ActivityEntryKind::Meta, "Cancellation requested");
	}

	/// Mark cancellation as complete and return to idle.
	pub fn cancel_complete(&mut self) {
		self.status = AssistantStatus::Idle;
		self.live_activity = None;
		self.record_activity(ActivityEntryKind::Meta, "Cancellation completed");
	}

	/// Append text to the last text block of the current assistant message.
	pub fn push_text(&mut self, text: &str) {
		if let Some(msg) = self.messages.last_mut() {
			match msg.blocks.last_mut() {
				Some(DisplayBlock::Text(buf)) => buf.push_str(text),
				_ => msg.blocks.push(DisplayBlock::Text(text.to_string())),
			}
		}
	}

	/// Append thinking text to the current assistant message.
	pub fn push_thinking(&mut self, text: &str) {
		self.live_activity = Some(LiveActivity {
			kind: ActivityKind::Thinking,
			started_at: self.live_activity.as_ref().map_or_else(Instant::now, |activity| {
				if activity.kind == ActivityKind::Thinking {
					activity.started_at
				} else {
					Instant::now()
				}
			}),
		});
		if let Some(msg) = self.messages.last_mut() {
			match msg.blocks.last_mut() {
				Some(DisplayBlock::Thinking(buf)) => buf.push_str(text),
				_ => msg.blocks.push(DisplayBlock::Thinking(text.to_string())),
			}
		}
	}

	/// Append redacted thinking text to the current assistant message.
	pub fn push_redacted_thinking(&mut self, text: &str) {
		self.live_activity = Some(LiveActivity {
			kind: ActivityKind::Thinking,
			started_at: self.live_activity.as_ref().map_or_else(Instant::now, |activity| {
				if activity.kind == ActivityKind::Thinking {
					activity.started_at
				} else {
					Instant::now()
				}
			}),
		});
		if let Some(msg) = self.messages.last_mut() {
			match msg.blocks.last_mut() {
				Some(DisplayBlock::RedactedThinking(buf)) => buf.push_str(text),
				_ => msg.blocks.push(DisplayBlock::RedactedThinking(text.to_string())),
			}
		}

		if text.is_empty() {
			self.record_activity(ActivityEntryKind::Meta, "Redacted thinking block received");
		}
	}

	/// Mark the assistant turn as finished.
	pub fn end_assistant_turn(&mut self) {
		self.status = AssistantStatus::Idle;
		self.live_activity = None;
	}

	/// Submit the current input as a user message, returning the text.
	/// Clears the input buffer and cursor and records the input in history.
	pub fn take_input(&mut self) -> Option<String> {
		let text = self.input.trim().to_string();
		if text.is_empty() {
			return None;
		}

		// Record in history
		self.history.push(text.clone());
		self.history_index = None;

		self.input.clear();
		self.cursor = 0;
		self.messages.push(DisplayMessage {
			role: "You".into(),
			blocks: vec![DisplayBlock::Text(text.clone())],
		});
		self.record_activity(
			ActivityEntryKind::User,
			format!("You asked: {}", truncate_for_activity(&text)),
		);
		Some(text)
	}

	/// Start tracking a tool use block on the current assistant message.
	pub fn start_tool_use(&mut self, id: String, name: String, input_preview: String) {
		if self.messages.last().is_none_or(|msg| msg.role != "Assistant") {
			self.begin_assistant_turn();
		}

		if let Some(msg) = self.messages.last_mut() {
			msg.blocks.push(DisplayBlock::ToolUse {
				id: id.clone(),
				name: name.clone(),
				input_preview,
				status: ToolStatus::Running,
			});
		}

		if !self.active_tools.iter().any(|n| n == &name) {
			self.active_tools.push(name.clone());
		}

		self.live_activity = Some(LiveActivity {
			kind: ActivityKind::Tool {
				name: name.clone(),
			},
			started_at: Instant::now(),
		});
		self.record_activity(ActivityEntryKind::Tool, format!("Started tool `{name}`"));
	}

	/// Complete a tool use and attach its result block.
	pub fn complete_tool_result(
		&mut self,
		id: String,
		name: String,
		output: String,
		is_error: bool,
	) {
		let activity_preview = truncate_for_activity(&output);

		if self.messages.last().is_none_or(|msg| msg.role != "Assistant") {
			self.begin_assistant_turn();
		}

		if let Some(msg) = self.messages.last_mut() {
			for block in msg.blocks.iter_mut().rev() {
				if let DisplayBlock::ToolUse {
					id: block_id,
					status,
					..
				} = block && block_id == &id
				{
					*status = if is_error {
						ToolStatus::Errored
					} else {
						ToolStatus::Done
					};
					break;
				}
			}

			msg.blocks.push(DisplayBlock::ToolResult {
				id,
				name: name.clone(),
				output,
				is_error,
			});
		}

		if let Some(index) = self.active_tools.iter().position(|n| n == &name) {
			self.active_tools.remove(index);
		}

		if self.status == AssistantStatus::Streaming {
			self.live_activity = Some(LiveActivity {
				kind: ActivityKind::Thinking,
				started_at: Instant::now(),
			});
		}

		if is_error {
			self.record_activity(
				ActivityEntryKind::Error,
				format!("Tool `{name}` failed: {activity_preview}"),
			);
		} else {
			self.record_activity(
				ActivityEntryKind::Success,
				format!("Tool `{name}` finished: {activity_preview}"),
			);
		}
	}

	/// Return the current slash command hint (if any) based on the input buffer.
	pub fn current_command_hint(&self) -> Option<&CommandHint> {
		if !self.input.starts_with('/') {
			return None;
		}
		let rest = &self.input[1..];
		let cmd = rest.split_whitespace().next().unwrap_or("");
		SLASH_COMMAND_HINTS.iter().find(|hint| hint.name == cmd)
	}

	/// Record a short session activity item for the dashboard side panel.
	pub fn record_activity(&mut self, kind: ActivityEntryKind, entry: impl Into<String>) {
		self.recent_activity.push(ActivityEntry {
			kind,
			text: entry.into(),
		});
		if self.recent_activity.len() > MAX_ACTIVITY_ITEMS {
			let overflow = self.recent_activity.len() - MAX_ACTIVITY_ITEMS;
			self.recent_activity.drain(0..overflow);
		}
	}
}

fn truncate_for_activity(text: &str) -> String {
	const MAX_CHARS: usize = 48;

	let mut out = text.chars().take(MAX_CHARS).collect::<String>();
	if text.chars().count() > MAX_CHARS {
		out.push('…');
	}
	out
}
