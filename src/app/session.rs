//! Session — drives the conversation loop between user and LLM.

use futures_util::StreamExt;

use crate::llm::client::LlmClient;
use crate::llm::request::{ChatRequest, SystemPrompt};
use crate::llm::response::{Delta, StopReason, StreamEvent};
use crate::llm::types::{ContentBlock, InputMessage, Role};
use crate::ui::UiAction;
use crate::ui::backend::UiHandle;
use crate::ui::events::{AppEvent, BlockType};

/// The conversation session that owns the LLM client, message history,
/// and the session-side channel handle.
pub struct Session {
	client: Box<dyn LlmClient>,
	messages: Vec<InputMessage>,
	system_prompt: String,
	max_tokens: u32,
	handle: UiHandle,
}

impl Session {
	/// Create a new session.
	pub fn new(
		client: Box<dyn LlmClient>,
		system_prompt: String,
		max_tokens: u32,
		handle: UiHandle,
	) -> Self {
		Self {
			client,
			messages: Vec::new(),
			system_prompt,
			max_tokens,
			handle,
		}
	}

	/// Run the conversation loop until the user exits.
	pub async fn run(mut self) -> crate::Result<()> {
		loop {
			// Wait for the next user action
			let Some(action) = self.handle.action_rx.recv().await else {
				// Channel closed — UI exited
				break;
			};

			match action {
				UiAction::SendMessage(text) => {
					self.handle_user_message(text).await?;
				}
				UiAction::Exit => break,
				UiAction::SlashCommand {
					command,
					args,
				} => {
					if !self.handle_slash_command(&command, &args).await {
						break;
					}
				}
				UiAction::CancelTurn => {
					// Nothing streaming right now at the top of the loop
				}
			}
		}

		Ok(())
	}

	async fn handle_user_message(&mut self, text: String) -> crate::Result<()> {
		// Append user message to history
		self.messages.push(InputMessage {
			role: Role::User,
			content: vec![ContentBlock::Text {
				text,
				cache_control: None,
			}],
		});

		// Build request with full history
		let request = ChatRequest {
			model: self.client.model_info().name.clone(),
			messages: self.messages.clone(),
			system: SystemPrompt::Single(self.system_prompt.clone()),
			max_tokens: Some(self.max_tokens),
			stream: true,
			temperature: None,
			top_p: None,
			tool_choice: None,
			tools: None,
			thinking: None,
		};

		// Signal turn start
		let _ = self.handle.event_tx.send(AppEvent::AssistantTurnStart).await;

		// Stream the response
		let mut stream = match self.client.chat_stream(request).await {
			Ok(s) => s,
			Err(e) => {
				let _ = self.handle.event_tx.send(AppEvent::Error(e.to_string())).await;
				return Ok(());
			}
		};

		// Accumulate assistant content blocks for history
		let mut text_buf = String::new();
		let mut stop_reason = StopReason::EndTurn;

		while let Some(event) = stream.next().await {
			match event {
				Ok(StreamEvent::ContentBlockDelta {
					index: _,
					delta,
				}) => match delta {
					Delta::TextDelta {
						text,
					} => {
						text_buf.push_str(&text);
						let _ = self.handle.event_tx.send(AppEvent::TextDelta(text)).await;
					}
					Delta::ThinkingDelta {
						thinking,
					} => {
						let _ = self.handle.event_tx.send(AppEvent::ThinkingDelta(thinking)).await;
					}
					_ => {}
				},
				Ok(StreamEvent::ContentBlockStop {
					index,
				}) => {
					let _ = self
						.handle
						.event_tx
						.send(AppEvent::BlockComplete {
							index,
							block_type: BlockType::Text,
						})
						.await;
				}
				Ok(StreamEvent::MessageDelta {
					delta,
					usage,
				}) => {
					if let Some(reason) = delta.stop_reason {
						stop_reason = reason;
					}
					let _ = self
						.handle
						.event_tx
						.send(AppEvent::UsageReport {
							input_tokens: usage.input_tokens,
							output_tokens: usage.output_tokens,
						})
						.await;
				}
				Ok(StreamEvent::Error {
					error,
				}) => {
					let _ = self.handle.event_tx.send(AppEvent::Error(error.to_string())).await;
				}
				Ok(_) => {} // Ping, MessageStart, MessageStop — ignored
				Err(e) => {
					let _ = self.handle.event_tx.send(AppEvent::Error(e.to_string())).await;
					break;
				}
			}
		}

		// Append assistant response to history
		if !text_buf.is_empty() {
			self.messages.push(InputMessage {
				role: Role::Assistant,
				content: vec![ContentBlock::Text {
					text: text_buf,
					cache_control: None,
				}],
			});
		}

		let _ = self
			.handle
			.event_tx
			.send(AppEvent::AssistantTurnEnd {
				stop_reason: stop_reason.clone(),
			})
			.await;

		Ok(())
	}

	/// Returns `true` to continue the loop, `false` to exit.
	async fn handle_slash_command(&self, command: &str, _args: &str) -> bool {
		match command {
			"help" => {
				let help_text =
					"Available commands:\n  /help  — show this message\n  /exit  — quit kloud";
				let _ = self.handle.event_tx.send(AppEvent::TextDelta(help_text.to_string())).await;
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
				let msg = format!("Unknown command: /{command}");
				let _ = self.handle.event_tx.send(AppEvent::Error(msg)).await;
			}
		}
		true
	}
}
