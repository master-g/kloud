//! TUI view state — the data model that the render functions draw from.

/// What the assistant is currently doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantStatus {
	/// Idle, waiting for user input.
	Idle,
	/// Streaming a response.
	Streaming,
}

/// A single content block within a display message.
#[derive(Debug, Clone)]
pub enum DisplayBlock {
	/// Plain text content.
	Text(String),
	/// Thinking / chain-of-thought content.
	Thinking(String),
}

/// A message shown in the messages area.
#[derive(Debug, Clone)]
pub struct DisplayMessage {
	/// `"You"` or `"Assistant"`.
	pub role: String,
	/// Content blocks in this message.
	pub blocks: Vec<DisplayBlock>,
}

/// All mutable state the TUI needs for rendering.
pub struct TuiState {
	/// Conversation messages to display.
	pub messages: Vec<DisplayMessage>,
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
	/// Last reported token usage.
	pub input_tokens: u32,
	/// Last reported token usage.
	pub output_tokens: u32,
	/// Whether the UI should exit.
	pub should_quit: bool,
}

impl TuiState {
	/// Create initial state for a given model name.
	pub fn new(model: String) -> Self {
		Self {
			messages: Vec::new(),
			input: String::new(),
			cursor: 0,
			scroll: 0,
			status: AssistantStatus::Idle,
			model,
			input_tokens: 0,
			output_tokens: 0,
			should_quit: false,
		}
	}

	/// Start a new assistant turn — append an empty assistant message.
	pub fn begin_assistant_turn(&mut self) {
		self.status = AssistantStatus::Streaming;
		self.messages.push(DisplayMessage {
			role: "Assistant".into(),
			blocks: vec![DisplayBlock::Text(String::new())],
		});
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
		if let Some(msg) = self.messages.last_mut() {
			match msg.blocks.last_mut() {
				Some(DisplayBlock::Thinking(buf)) => buf.push_str(text),
				_ => msg.blocks.push(DisplayBlock::Thinking(text.to_string())),
			}
		}
	}

	/// Mark the assistant turn as finished.
	pub fn end_assistant_turn(&mut self) {
		self.status = AssistantStatus::Idle;
	}

	/// Submit the current input as a user message, returning the text.
	/// Clears the input buffer and cursor.
	pub fn take_input(&mut self) -> Option<String> {
		let text = self.input.trim().to_string();
		if text.is_empty() {
			return None;
		}
		self.input.clear();
		self.cursor = 0;
		self.messages.push(DisplayMessage {
			role: "You".into(),
			blocks: vec![DisplayBlock::Text(text.clone())],
		});
		Some(text)
	}
}
