//! Ratatui-based TUI backend.
//!
//! Runs a full-screen terminal UI with a messages area, input bar, and
//! status line. Uses `tokio::select!` to multiplex terminal events,
//! application events, and a render tick.

pub mod input;
pub mod state;
pub mod widgets;

use std::io;
use std::time::Duration;

use crossterm::event::EventStream;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event as ct_event, execute};
use futures_util::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::time;

use self::state::{ActivityEntryKind, AssistantStatus, TuiState};
use super::backend::{UiBackend, UiChannels};
use super::events::AppEvent;

/// Frames per second for the render tick.
const FPS: u64 = 20;

/// A full-screen ratatui terminal UI.
pub struct RatatuiBackend {
	/// Model name shown in the status bar.
	pub model: String,
	/// Maximum context window tokens for the current model.
	pub max_context_tokens: u32,
	/// Workspace path shown in the dashboard.
	pub workspace: String,
	/// Current git branch shown in the dashboard.
	pub branch: String,
	/// Effort label shown in the dashboard.
	pub effort: String,
	/// Number of tools available in the current session.
	pub tool_count: usize,
	/// Instruction files discovered for the current workspace.
	pub instruction_files: Vec<String>,
	/// Number of active hooks in the current repository.
	pub hook_count: usize,
}

/// RAII guard that restores the terminal on drop (even on panic).
struct TerminalGuard;

impl TerminalGuard {
	fn setup() -> io::Result<Self> {
		terminal::enable_raw_mode()?;
		execute!(io::stderr(), EnterAlternateScreen)?;
		Ok(Self)
	}
}

impl Drop for TerminalGuard {
	fn drop(&mut self) {
		let _ = execute!(io::stderr(), LeaveAlternateScreen);
		let _ = terminal::disable_raw_mode();
	}
}

#[async_trait::async_trait]
impl UiBackend for RatatuiBackend {
	async fn run(self, mut channels: UiChannels) -> crate::Result<()> {
		// Set up terminal with RAII cleanup guard
		let _guard = TerminalGuard::setup()?;
		let backend = CrosstermBackend::new(io::stderr());
		let mut terminal = Terminal::new(backend)?;

		let mut state = TuiState::new(
			self.model,
			self.max_context_tokens,
			self.workspace,
			self.branch,
			self.effort,
			self.tool_count,
			self.instruction_files,
			self.hook_count,
		);
		let mut event_stream = EventStream::new();
		let mut tick = time::interval(Duration::from_millis(1000 / FPS));
		tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

		loop {
			tokio::select! {
				// (a) Terminal events (keyboard, resize)
				maybe_event = event_stream.next() => {
					if let Some(Ok(event)) = maybe_event {
						if let ct_event::Event::Resize(_, _) = &event {
							// Just re-render on next tick
						} else if let Some(action) = input::handle_event(&event, &mut state) {
							let _ = channels.action_tx.send(action).await;
						}
					}
				}
				// (b) Application events (stream deltas, errors, shutdown)
				maybe_app = channels.event_rx.recv() => {
					match maybe_app {
						Some(AppEvent::AssistantTurnStart) => {
							state.begin_assistant_turn();
						}
						Some(AppEvent::TextDelta(text)) => {
							state.push_text(&text);
						}
						Some(AppEvent::ThinkingDelta(text)) => {
							state.push_thinking(&text);
						}
						Some(AppEvent::RedactedThinking(text)) => {
							state.push_redacted_thinking(&text);
						}
						Some(AppEvent::BlockComplete { .. }) => {}
						Some(AppEvent::ToolUseStart { id, name, input_preview }) => {
							state.start_tool_use(id, name, input_preview);
						}
						Some(AppEvent::ToolResult { id, name, output, is_error }) => {
							state.complete_tool_result(id, name, output, is_error);
						}
						Some(AppEvent::AssistantTurnEnd { stop_reason }) => {
							let stop_reason_label = format_stop_reason(&stop_reason);
							state.last_stop_reason = Some(stop_reason);
							state.record_activity(
								ActivityEntryKind::Meta,
								format!("Stop reason: {stop_reason_label}"),
							);
							if state.status == AssistantStatus::Cancelling {
								state.cancel_complete();
							} else {
								state.end_assistant_turn();
							}
						}
						Some(AppEvent::Error(msg)) => {
							state.messages.push(state::DisplayMessage {
								role: "Error".into(),
								blocks: vec![state::DisplayBlock::Text(msg)],
							});
							state.record_activity(ActivityEntryKind::Error, "Session error reported");
							state.end_assistant_turn();
						}
						Some(AppEvent::UsageReport { input_tokens, output_tokens }) => {
							state.input_tokens = input_tokens;
							state.output_tokens = output_tokens;
						}
						Some(AppEvent::Shutdown) | None => {
							state.should_quit = true;
						}
					}
				}
				// (c) Render tick
				_ = tick.tick() => {
					state.animation_tick = state.animation_tick.wrapping_add(1);
				}
			}

			// Draw
			terminal.draw(|frame| widgets::render(frame, &state))?;

			if state.should_quit {
				break;
			}
		}

		Ok(())
	}
}

fn format_stop_reason(reason: &crate::llm::response::StopReason) -> &'static str {
	match reason {
		crate::llm::response::StopReason::EndTurn => "end_turn",
		crate::llm::response::StopReason::MaxTokens => "max_tokens",
		crate::llm::response::StopReason::StopSequence => "stop_sequence",
		crate::llm::response::StopReason::ToolUse => "tool_use",
		crate::llm::response::StopReason::PauseTurn => "pause_turn",
		crate::llm::response::StopReason::Refusal => "refusal",
		crate::llm::response::StopReason::ModelContextWindowExceeded => "ctx_exceeded",
	}
}
