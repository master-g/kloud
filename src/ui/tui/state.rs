//! TUI view state — the data model that the render functions draw from.
#![allow(missing_docs)]

use std::time::Instant;

use crate::llm::response::StopReason;

// ============================================================================
// Animation Clock & Activity State
// ============================================================================

/// Activity 行动画时钟，每 50ms 递增一次 tick。
#[derive(Debug, Clone)]
pub struct ActivityClock {
	/// 50ms 单位的时间滴答
	pub tick: u64,
	last_update: Instant,
}

impl ActivityClock {
	/// 创建新的活动时钟，初始化为当前时间。
	pub fn new() -> Self {
		Self {
			tick: 0,
			last_update: Instant::now(),
		}
	}

	/// 尝试推进时钟。如果过去了至少 50ms，返回 true 并递增 tick。
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
// Stalled State — 卡顿检测
// ============================================================================

/// 卡顿检测状态，检测响应是否卡住（3秒无输出变化）。
#[derive(Debug, Clone)]
pub struct StalledState {
	/// 上次响应长度
	last_response_length: usize,
	/// 上次有响应的时间
	last_token_at: Instant,
	/// 当前卡顿强度 (0.0 - 1.0)
	intensity: f32,
	/// 是否正在卡顿
	is_stalled: bool,
}

impl StalledState {
	/// 创建新的卡顿检测状态。
	pub fn new() -> Self {
		Self {
			last_response_length: 0,
			last_token_at: Instant::now(),
			intensity: 0.0,
			is_stalled: false,
		}
	}

	/// 更新响应长度，判断是否卡顿。
	///
	/// Uses exponential moving average (EMA) smoothing matching Claude Code's
	/// `stalledIntensityRef += (target - current) * 0.1` approach for
	/// smooth transitions both into and out of the stalled state.
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

		// EMA smoothing: alpha = 0.1 per 50ms tick
		if target > 0.0 || self.intensity > 0.0 {
			self.intensity += (target - self.intensity) * 0.1;
			if self.intensity < 0.001 {
				self.intensity = 0.0;
			}
		}
	}

	/// 返回当前卡顿强度 (0.0 - 1.0)。
	pub fn intensity(&self) -> f32 {
		self.intensity
	}

	/// 返回是否正在卡顿。
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
// Token Counter — 平滑递增计数
// ============================================================================

/// Smooth character-count counter matching CC's `tokenCounterRef`.
///
/// CC smoothly increments a *character count* toward the real response length,
/// then divides by 4 to derive the displayed token count. Gap thresholds
/// (70 / 200 / +50) operate at the character level, producing a ~4x slower
/// (smoother) animation than if they operated on tokens directly.
#[derive(Debug, Clone)]
pub struct TokenCounter {
	/// Smoothed character count (approaches `target_chars`).
	displayed_chars: usize,
	/// Real cumulative response character count.
	target_chars: usize,
}

impl TokenCounter {
	pub fn new() -> Self {
		Self {
			displayed_chars: 0,
			target_chars: 0,
		}
	}

	/// Set the target character count (typically `response_char_count`).
	pub fn set_target(&mut self, chars: usize) {
		self.target_chars = chars;
	}

	/// Advance the smooth counter one frame toward the target.
	///
	/// Gap thresholds match CC's `SpinnerAnimationRow`:
	/// - gap < 70: +3/frame
	/// - gap < 200: max(8, ceil(gap * 0.15))
	/// - gap >= 200: +50/frame
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

	/// Estimated token count derived from the smoothed character count.
	/// Matches CC: `Math.round(displayedResponseLength / 4)`.
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
use std::time::Duration;

use crate::ui::constants::{
	ACTIVITY_PREVIEW_MAX_CHARS, ACTIVITY_SNAPSHOT_MS, ELLIPSIS, MAX_ACTIVITY_ITEMS, SPINNER_VERBS,
	THINKING_DURATION_SHOW_MS, THINKING_MIN_DISPLAY_MS,
};

// ============================================================================
// Thinking Status — 3-state machine (対標 Spinner.tsx `thinkingStatus`)
// ============================================================================

/// Display state machine for thinking indicator.
///
/// Three states with minimum display times to avoid UI jank:
/// `Active` → `PendingDuration` → `ShowDuration` → `None`
#[derive(Debug, Clone)]
pub enum ThinkingStatus {
	/// Not thinking / nothing to display.
	None,
	/// Actively thinking — display "thinking" with shimmer animation.
	Active {
		started_at: Instant,
	},
	/// Thinking ended but "thinking" hasn't been shown for the minimum 2s yet.
	PendingDuration {
		duration_ms: u64,
		min_display_until: Instant,
	},
	/// Showing "thought for Xs" for 2 seconds.
	ShowDuration {
		duration_ms: u64,
		clear_at: Instant,
	},
}

impl ThinkingStatus {
	/// Returns the display text for the thinking indicator, if any.
	pub fn display_text(&self, effort_suffix: &str) -> Option<String> {
		match self {
			ThinkingStatus::None => None,
			ThinkingStatus::Active {
				..
			}
			| ThinkingStatus::PendingDuration {
				..
			} => Some(format!("thinking{effort_suffix}")),
			ThinkingStatus::ShowDuration {
				duration_ms,
				..
			} => {
				let secs = (*duration_ms as f64 / 1000.0).round().max(1.0) as u64;
				Some(format!("thought for {secs}s"))
			}
		}
	}

	/// Whether the text should use shimmer animation (active or pending states).
	pub fn is_shimmering(&self) -> bool {
		matches!(self, ThinkingStatus::Active { .. } | ThinkingStatus::PendingDuration { .. })
	}

	/// Whether thinking is currently visible (any non-None state).
	pub fn is_visible(&self) -> bool {
		!matches!(self, ThinkingStatus::None)
	}
}
use crate::ui::events::BlockType;

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

/// Accent family for the ephemeral activity bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityAccent {
	Info,
	Tool,
}

/// Spinner display mode, controlling glyph icon and animation behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerMode {
	/// Waiting for user input or API response, shows ↑ icon.
	Requesting,
	/// Streaming text response, shows ↓ icon.
	Responding,
	/// Executing a tool, shows ↓ icon.
	ToolUse,
	/// Extended thinking, shows ↓ icon.
	Thinking,
}

/// Short-lived work the assistant is currently performing.
///
/// CC model: one random verb per turn (e.g. `"Cooking…"`), stable across
/// all mode changes. Only `mode` and `accent` change during the turn.
#[derive(Debug, Clone)]
pub struct LiveActivity {
	pub message: String,
	pub accent: ActivityAccent,
	pub mode: SpinnerMode,
	pub started_at: Instant,
	pub last_signal_at: Instant,
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
		server_name: Option<String>,
		input_preview: String,
		status: ToolStatus,
	},
	/// Result of a previously-started tool.
	ToolResult {
		id: String,
		name: String,
		server_name: Option<String>,
		output: String,
		is_error: bool,
	},
}

/// Severity level for system messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageLevel {
	Info,
	Warning,
	Error,
}

/// The kind of message in the conversation transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
	User,
	Assistant,
	System {
		level: MessageLevel,
	},
}

/// Auto-incrementing ID source for display messages.
static NEXT_MESSAGE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// A message shown in the messages area.
#[derive(Debug, Clone)]
pub struct DisplayMessage {
	pub id: u64,
	pub message_type: MessageType,
	/// Content blocks in this message.
	pub blocks: Vec<DisplayBlock>,
}

impl DisplayMessage {
	pub fn new(message_type: MessageType, blocks: Vec<DisplayBlock>) -> Self {
		Self {
			id: NEXT_MESSAGE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
			message_type,
			blocks,
		}
	}
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

#[derive(Debug, Clone)]
pub struct ActivitySnapshot {
	pub activity: LiveActivity,
	pub expires_at: Instant,
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
	/// Ephemeral activity line shown above the status bar.
	pub live_activity: Option<LiveActivity>,
	/// Last completed activity line, kept briefly to avoid visual jump.
	pub activity_snapshot: Option<ActivitySnapshot>,
	/// Random verb chosen at turn start, stable for the entire turn (CC model).
	pub turn_verb: String,
	/// Activity 行动画时钟（50ms tick）。
	pub activity_clock: ActivityClock,
	/// 卡顿检测状态。
	pub stalled_state: StalledState,
	/// Token 计数动画。
	pub token_counter: TokenCounter,
	/// Thinking 显示状态机（`Active` → `PendingDuration` → `ShowDuration` → `None`）。
	pub thinking_status: ThinkingStatus,
	/// Cumulative byte length of streamed response content (text + thinking).
	/// Used to estimate token count in real-time (chars / 4), matching CC's
	/// `responseLengthRef.current / 4` approach.
	pub response_char_count: usize,
}

impl TuiState {
	/// Create initial state for a given model name.
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
			live_activity: None,
			activity_snapshot: None,
			turn_verb: pick_random_verb(),
			activity_clock: ActivityClock::new(),
			stalled_state: StalledState::new(),
			token_counter: TokenCounter::new(),
			thinking_status: ThinkingStatus::None,
			response_char_count: 0,
		}
	}

	/// Start a new assistant turn — append an empty assistant message.
	pub fn begin_assistant_turn(&mut self) {
		self.status = AssistantStatus::Streaming;
		self.activity_snapshot = None;
		self.thinking_status = ThinkingStatus::None;
		self.response_char_count = 0;
		self.stalled_state = StalledState::new();
		self.turn_verb = pick_random_verb();
		self.set_activity(SpinnerMode::Requesting, ActivityAccent::Info);
		self.messages.push(DisplayMessage::new(MessageType::Assistant, Vec::new()));
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
		self.activity_snapshot = None;
		self.record_activity(ActivityEntryKind::Meta, "Cancellation completed");
	}

	/// Append text to the last text block of the current assistant message.
	pub fn push_text(&mut self, text: &str) {
		self.touch_or_set_activity(SpinnerMode::Responding, ActivityAccent::Info);
		self.ensure_assistant_message();
		self.response_char_count += text.len();
		if let Some(msg) = self.messages.last_mut() {
			match msg.blocks.last_mut() {
				Some(DisplayBlock::Text(buf)) => buf.push_str(text),
				_ => msg.blocks.push(DisplayBlock::Text(text.to_string())),
			}
		}
	}

	/// Append thinking text to the current assistant message.
	pub fn push_thinking(&mut self, text: &str) {
		self.touch_or_set_activity(SpinnerMode::Thinking, ActivityAccent::Info);
		self.response_char_count += text.len();
		if let Some(msg) = self.messages.last_mut() {
			match msg.blocks.last_mut() {
				Some(DisplayBlock::Thinking(buf)) => buf.push_str(text),
				_ => msg.blocks.push(DisplayBlock::Thinking(text.to_string())),
			}
		}
	}

	/// Append redacted thinking text to the current assistant message.
	pub fn push_redacted_thinking(&mut self, text: &str) {
		self.touch_or_set_activity(SpinnerMode::Thinking, ActivityAccent::Info);
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
		let now = Instant::now();
		if let Some(activity) = self.live_activity.clone() {
			self.activity_snapshot = Some(ActivitySnapshot {
				activity,
				expires_at: now + Duration::from_millis(ACTIVITY_SNAPSHOT_MS),
			});
		}
		self.live_activity = None;
		self.thinking_status = ThinkingStatus::None;
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
		self.messages
			.push(DisplayMessage::new(MessageType::User, vec![DisplayBlock::Text(text.clone())]));
		self.record_activity(
			ActivityEntryKind::User,
			format!("You asked: {}", truncate_for_activity(&text)),
		);
		Some(text)
	}

	/// Start tracking a tool use block on the current assistant message.
	pub fn start_tool_use(
		&mut self,
		id: String,
		name: String,
		server_name: Option<String>,
		input_preview: String,
	) {
		self.ensure_assistant_message();

		if let Some(msg) = self.messages.last_mut() {
			msg.blocks.push(DisplayBlock::ToolUse {
				id: id.clone(),
				name: name.clone(),
				server_name,
				input_preview,
				status: ToolStatus::Running,
			});
		}

		if !self.active_tools.iter().any(|n| n == &name) {
			self.active_tools.push(name.clone());
		}

		self.set_activity(SpinnerMode::ToolUse, ActivityAccent::Tool);
		self.record_activity(ActivityEntryKind::Tool, format!("Started tool `{name}`"));
	}

	/// Complete a tool use and attach its result block.
	pub fn complete_tool_result(
		&mut self,
		id: String,
		name: String,
		server_name: Option<String>,
		output: String,
		is_error: bool,
	) {
		let activity_preview = truncate_for_activity(&output);

		self.ensure_assistant_message();

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
				server_name,
				output,
				is_error,
			});
		}

		if let Some(index) = self.active_tools.iter().position(|n| n == &name) {
			self.active_tools.remove(index);
		}

		if self.status == AssistantStatus::Streaming {
			self.set_activity(SpinnerMode::Thinking, ActivityAccent::Info);
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

	/// Note that a content block finished (used for thinking status transitions).
	pub fn note_block_complete(&mut self, _block_type: BlockType) {
		// In CC's model the verb stays stable for the entire turn, so no
		// rotation happens here. The method is kept for potential future
		// block-level state transitions.
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

	/// Returns the effort suffix string for thinking display (e.g. " with high effort").
	pub fn effort_suffix(&self) -> String {
		if self.effort.is_empty() || self.effort == "default" {
			String::new()
		} else {
			format!(" with {} effort", self.effort)
		}
	}

	/// Transition the thinking status when the spinner mode changes.
	pub fn transition_thinking_status(&mut self, new_mode: SpinnerMode) {
		let now = Instant::now();
		match (&self.thinking_status, new_mode) {
			// Entering (or re-entering) thinking from non-active states
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
			// Already thinking — no change
			(
				ThinkingStatus::Active {
					..
				},
				SpinnerMode::Thinking,
			) => {}
			// Leaving thinking — transition to duration display
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

	/// Advance timer-based thinking status transitions (call on each tick).
	pub fn tick_thinking_status(&mut self) {
		let now = Instant::now();
		match &self.thinking_status {
			ThinkingStatus::PendingDuration {
				duration_ms,
				min_display_until,
			} => {
				if now >= *min_display_until {
					self.thinking_status = ThinkingStatus::ShowDuration {
						duration_ms: *duration_ms,
						clear_at: now + Duration::from_millis(THINKING_DURATION_SHOW_MS),
					};
				}
			}
			ThinkingStatus::ShowDuration {
				clear_at,
				..
			} => {
				if now >= *clear_at {
					self.thinking_status = ThinkingStatus::None;
				}
			}
			_ => {}
		}
	}

	/// Returns the activity line source to render (live first, then snapshot).
	pub fn displayed_activity(&self) -> Option<&LiveActivity> {
		self.live_activity
			.as_ref()
			.or_else(|| self.activity_snapshot.as_ref().map(|snapshot| &snapshot.activity))
	}

	/// Expire the retained activity snapshot when its TTL elapses.
	pub fn tick_activity_snapshot(&mut self) {
		if self.activity_snapshot.as_ref().is_some_and(|s| Instant::now() >= s.expires_at) {
			self.activity_snapshot = None;
		}
	}

	fn ensure_assistant_message(&mut self) {
		if self.messages.last().is_none_or(|msg| msg.message_type != MessageType::Assistant) {
			self.begin_assistant_turn();
		}
	}

	/// Push a system-level message (errors, warnings, info).
	pub fn push_system_message(&mut self, level: MessageLevel, text: String) {
		let kind = match level {
			MessageLevel::Error => ActivityEntryKind::Error,
			MessageLevel::Warning | MessageLevel::Info => ActivityEntryKind::Meta,
		};
		self.record_activity(kind, truncate_for_activity(&text));
		self.messages.push(DisplayMessage::new(
			MessageType::System {
				level,
			},
			vec![DisplayBlock::Text(text)],
		));
	}

	/// Set (or create) the live activity with a given mode and accent.
	/// The verb text comes from `turn_verb`, stable for the entire turn.
	fn set_activity(&mut self, mode: SpinnerMode, accent: ActivityAccent) {
		self.transition_thinking_status(mode);
		let now = Instant::now();
		let started_at = self.live_activity.as_ref().map_or(now, |a| a.started_at);
		self.live_activity = Some(LiveActivity {
			message: self.turn_verb.clone(),
			accent,
			mode,
			started_at,
			last_signal_at: now,
		});
	}

	/// Touch the existing activity's signal timestamp, or create one if absent.
	fn touch_or_set_activity(&mut self, mode: SpinnerMode, accent: ActivityAccent) {
		if let Some(activity) = self.live_activity.as_mut() {
			activity.mode = mode;
			activity.accent = accent;
			activity.last_signal_at = Instant::now();
			self.transition_thinking_status(mode);
		} else {
			self.set_activity(mode, accent);
		}
	}
}

/// Pick a random verb from `SPINNER_VERBS` and append the ellipsis.
fn pick_random_verb() -> String {
	let idx = fastrand::usize(0..SPINNER_VERBS.len());
	format!("{}{ELLIPSIS}", SPINNER_VERBS[idx])
}

fn truncate_for_activity(text: &str) -> String {
	let mut out = text.chars().take(ACTIVITY_PREVIEW_MAX_CHARS).collect::<String>();
	if text.chars().count() > ACTIVITY_PREVIEW_MAX_CHARS {
		out.push('…');
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	fn test_state() -> TuiState {
		TuiState::new(
			"claude-test".to_string(),
			200_000,
			"/tmp/workspace".to_string(),
			"main".to_string(),
			"default".to_string(),
			2,
			Vec::new(),
			0,
		)
	}

	#[test]
	fn verb_stays_stable_within_a_turn() {
		let mut state = test_state();
		state.begin_assistant_turn();
		let first = state.live_activity.clone().expect("live activity");

		state.push_thinking("still thinking");
		let after_thinking = state.live_activity.clone().expect("live activity");

		state.push_text("some text");
		let after_text = state.live_activity.clone().expect("live activity");

		assert_eq!(first.message, after_thinking.message);
		assert_eq!(first.message, after_text.message);
		assert_eq!(first.started_at, after_text.started_at);
	}

	#[test]
	fn verb_stays_stable_across_mode_changes() {
		let mut state = test_state();
		state.begin_assistant_turn();
		let initial = state.live_activity.clone().expect("live activity");
		assert_eq!(initial.mode, SpinnerMode::Requesting);

		state.push_text("response");
		let after_text = state.live_activity.clone().expect("live activity");
		assert_eq!(after_text.mode, SpinnerMode::Responding);
		assert_eq!(initial.message, after_text.message);

		state.start_tool_use("t1".into(), "read".into(), None, "preview".into());
		let after_tool = state.live_activity.clone().expect("live activity");
		assert_eq!(after_tool.mode, SpinnerMode::ToolUse);
		assert_eq!(initial.message, after_tool.message);
	}

	#[test]
	fn verb_changes_between_turns() {
		let mut found_different = false;
		for _ in 0..50 {
			let mut state = test_state();
			state.begin_assistant_turn();
			let first = state.live_activity.clone().expect("live activity");
			state.end_assistant_turn();

			state.begin_assistant_turn();
			let second = state.live_activity.clone().expect("live activity");

			if first.message != second.message {
				found_different = true;
				break;
			}
		}
		assert!(found_different, "verb should change between turns (statistical)");
	}

	#[test]
	fn verb_message_contains_ellipsis() {
		let mut state = test_state();
		state.begin_assistant_turn();
		let activity = state.live_activity.clone().expect("live activity");
		assert!(
			activity.message.ends_with('\u{2026}'),
			"verb message should end with ellipsis: {}",
			activity.message
		);
	}
}
