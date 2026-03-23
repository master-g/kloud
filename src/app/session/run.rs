use futures_util::StreamExt;
use tokio::select;

use crate::llm::response::StopReason;
use crate::ui::UiAction;
use crate::ui::events::AppEvent;

use super::types::{StreamingActionOutcome, TurnState};
use super::{SLASH_COMMANDS, Session};

impl Session {
	/// Run the conversation loop until the user exits.
	pub async fn run(mut self) -> crate::Result<()> {
		// Start with no active model turn. The loop below repeatedly takes the
		// current turn state, advances it by one step, then stores the next state.
		let mut turn_state = TurnState::Idle;

		loop {
			let current_state = std::mem::replace(&mut turn_state, TurnState::Idle);
			turn_state = match current_state {
				TurnState::Idle => {
					// Outer loop: wait for the next user-driven action.
					let Some(action) = self.handle.action_rx.recv().await else {
						break;
					};

					match action {
						UiAction::SendMessage(text) => self.start_streaming_turn(text).await?,
						UiAction::Exit => break,
						UiAction::SlashCommand {
							command,
							args,
						} => {
							if !self.handle_slash_command(&command, &args).await {
								break;
							}
							TurnState::Idle
						}
						UiAction::CancelTurn => TurnState::Idle,
					}
				}
				TurnState::Streaming {
					mut stream,
					stop_reason,
					block_types,
					tool_names_by_id,
					completed_assistant_blocks,
					pending_blocks_by_index,
				} => {
					// While a model turn is active we react to two event sources:
					// 1. the next streamed event from the LLM
					// 2. a cancellation/exit action from the UI
					select! {
						maybe_event = stream.next() => {
							self
								.handle_stream_poll(
									maybe_event,
									stream,
									stop_reason,
									block_types,
									tool_names_by_id,
									completed_assistant_blocks,
									pending_blocks_by_index,
								)
								.await?
						}
						maybe_action = self.handle.action_rx.recv() => {
							match self
								.handle_stream_action(
									maybe_action,
									stream,
									stop_reason,
									block_types,
									tool_names_by_id,
									completed_assistant_blocks,
									pending_blocks_by_index,
								)
								.await?
							{
								StreamingActionOutcome::Next(next_state) => next_state,
								StreamingActionOutcome::Exit => break,
							}
						}
					}
				}
			};
		}

		Ok(())
	}

	/// Returns `true` to continue the loop, `false` to exit.
	async fn handle_slash_command(&mut self, command: &str, _args: &str) -> bool {
		match command {
			"help" => {
				let mut help_lines = vec![String::from("Available commands:")];
				for entry in SLASH_COMMANDS {
					help_lines.push(format!("  /{} — {}", entry.name, entry.summary));
				}

				let _ = self.handle.event_tx.send(AppEvent::AssistantTurnStart).await;
				let _ = self.handle.event_tx.send(AppEvent::TextDelta(help_lines.join("\n"))).await;
				let _ = self
					.handle
					.event_tx
					.send(AppEvent::AssistantTurnEnd {
						stop_reason: StopReason::EndTurn,
					})
					.await;
			}
			"exit" | "quit" => {
				let _ = self.handle.event_tx.send(AppEvent::Shutdown).await;
				return false;
			}
			_ => {
				let known = SLASH_COMMANDS
					.iter()
					.map(|entry| format!("/{}", entry.name))
					.collect::<Vec<_>>()
					.join(", ");
				let msg = format!("Unknown command: /{command}. Try /help. Available: {known}");
				let _ = self.handle.event_tx.send(AppEvent::Error(msg)).await;
			}
		}
		true
	}
}
