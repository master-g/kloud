//! Session — drives the conversation loop between user and LLM.

use std::collections::HashMap;
use std::pin::Pin;

use futures::Stream;
use futures_util::StreamExt;
use tokio::select;

use crate::llm::client::LlmClient;
use crate::llm::request::{ChatRequest, SystemPrompt};
use crate::llm::response::{Delta, StopReason, StreamEvent};
use crate::llm::types::{ContentBlock, InputMessage, Role};
use crate::ui::UiAction;
use crate::ui::backend::UiHandle;
use crate::ui::events::{AppEvent, BlockType};

/// Basic slash-command metadata.
struct CommandInfo {
	name: &'static str,
	summary: &'static str,
}

const SLASH_COMMANDS: &[CommandInfo] = &[
	CommandInfo {
		name: "help",
		summary: "Show available commands",
	},
	CommandInfo {
		name: "exit",
		summary: "Quit kloud",
	},
	CommandInfo {
		name: "quit",
		summary: "Alias for /exit",
	},
];

/// The conversation session that owns the LLM client, message history,
/// and the session-side channel handle.
pub struct Session {
	client: Box<dyn LlmClient>,
	messages: Vec<InputMessage>,
	system_prompt: String,
	max_tokens: u32,
	handle: UiHandle,
}

/// Internal state for the currently streaming assistant turn.
enum TurnState {
	Idle,
	Streaming {
		stream: Pin<Box<dyn Stream<Item = Result<StreamEvent, crate::error::LlmError>> + Send>>,
		text_buf: String,
		stop_reason: StopReason,
		block_types: HashMap<u32, BlockType>,
		tool_names_by_id: HashMap<String, String>,
	},
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
		let mut turn_state = TurnState::Idle;

		loop {
			let current_state = std::mem::replace(&mut turn_state, TurnState::Idle);
			turn_state = match current_state {
				TurnState::Idle => {
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
					mut text_buf,
					mut stop_reason,
					mut block_types,
					mut tool_names_by_id,
				} => {
					select! {
						maybe_event = stream.next() => {
							let Some(event) = maybe_event else {
								self.finish_streaming_turn(Some(text_buf), stop_reason).await?;
								continue;
							};

							match event {
								Ok(event) => {
									self
										.handle_stream_event(
											event,
											&mut text_buf,
											&mut stop_reason,
											&mut block_types,
											&mut tool_names_by_id,
										)
										.await?;
								}
								Err(e) => {
									let _ = self.handle.event_tx.send(AppEvent::Error(e.to_string())).await;
									self.finish_streaming_turn(Some(text_buf), stop_reason).await?;
									continue;
								}
							}

							TurnState::Streaming {
								stream,
								text_buf,
								stop_reason,
								block_types,
								tool_names_by_id,
							}
						}
						maybe_action = self.handle.action_rx.recv() => {
							match maybe_action {
								Some(UiAction::CancelTurn) => {
									self.finish_streaming_turn(None, StopReason::EndTurn).await?;
									TurnState::Idle
								}
								Some(UiAction::Exit) => {
									self.finish_streaming_turn(Some(text_buf), stop_reason).await?;
									break;
								}
								Some(UiAction::SendMessage(_)) | Some(UiAction::SlashCommand { .. }) => {
									TurnState::Streaming {
										stream,
										text_buf,
										stop_reason,
										block_types,
										tool_names_by_id,
									}
								}
								None => break,
							}
						}
					}
				}
			};
		}

		Ok(())
	}

	async fn start_streaming_turn(&mut self, text: String) -> crate::Result<TurnState> {
		// Append user message to history.
		self.messages.push(InputMessage {
			role: Role::User,
			content: vec![ContentBlock::Text {
				text,
				cache_control: None,
			}],
		});

		// Build request with full history.
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

		let _ = self.handle.event_tx.send(AppEvent::AssistantTurnStart).await;

		let stream = match self.client.chat_stream(request).await {
			Ok(stream) => stream,
			Err(e) => {
				let _ = self.handle.event_tx.send(AppEvent::Error(e.to_string())).await;
				return Ok(TurnState::Idle);
			}
		};

		Ok(TurnState::Streaming {
			stream,
			text_buf: String::new(),
			stop_reason: StopReason::EndTurn,
			block_types: HashMap::new(),
			tool_names_by_id: HashMap::new(),
		})
	}

	async fn handle_stream_event(
		&mut self,
		event: StreamEvent,
		text_buf: &mut String,
		stop_reason: &mut StopReason,
		block_types: &mut HashMap<u32, BlockType>,
		tool_names_by_id: &mut HashMap<String, String>,
	) -> crate::Result<()> {
		match event {
			StreamEvent::ContentBlockStart {
				index,
				content_block,
			} => match content_block {
				ContentBlock::Text {
					..
				} => {
					block_types.insert(index, BlockType::Text);
				}
				ContentBlock::Thinking {
					..
				}
				| ContentBlock::RedactedThinking {
					..
				} => {
					block_types.insert(index, BlockType::Thinking);
				}
				ContentBlock::ToolUse {
					id,
					name,
					input,
					..
				} => {
					let input_preview = Self::format_json_preview(&input);
					tool_names_by_id.insert(id.clone(), name.clone());
					block_types.insert(index, BlockType::ToolUse);
					let _ = self
						.handle
						.event_tx
						.send(AppEvent::ToolUseStart {
							id,
							name,
							input_preview,
						})
						.await;
				}
				ContentBlock::ToolResult {
					tool_use_id,
					content,
					is_error,
				} => {
					let name = tool_names_by_id
						.get(&tool_use_id)
						.cloned()
						.unwrap_or_else(|| "tool".to_string());
					block_types.insert(index, BlockType::ToolResult);
					let _ = self
						.handle
						.event_tx
						.send(AppEvent::ToolResult {
							id: tool_use_id,
							name,
							output: content,
							is_error: is_error.unwrap_or(false),
						})
						.await;
				}
			},
			StreamEvent::ContentBlockDelta {
				delta,
				..
			} => match delta {
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
				Delta::InputJsonDelta {
					..
				}
				| Delta::SignatureDelta {
					..
				} => {}
			},
			StreamEvent::ContentBlockStop {
				index,
			} => {
				let block_type = block_types.remove(&index).unwrap_or(BlockType::Text);
				let _ = self
					.handle
					.event_tx
					.send(AppEvent::BlockComplete {
						index,
						block_type,
					})
					.await;
			}
			StreamEvent::MessageDelta {
				delta,
				usage,
			} => {
				if let Some(reason) = delta.stop_reason {
					*stop_reason = reason;
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
			StreamEvent::Error {
				error,
			} => {
				let _ = self.handle.event_tx.send(AppEvent::Error(error.to_string())).await;
			}
			StreamEvent::MessageStart {
				..
			}
			| StreamEvent::MessageStop
			| StreamEvent::Ping => {}
		}

		Ok(())
	}

	async fn finish_streaming_turn(
		&mut self,
		text_buf: Option<String>,
		stop_reason: StopReason,
	) -> crate::Result<()> {
		if let Some(text) = text_buf
			&& !text.is_empty()
		{
			self.messages.push(InputMessage {
				role: Role::Assistant,
				content: vec![ContentBlock::Text {
					text,
					cache_control: None,
				}],
			});
		}

		let _ = self
			.handle
			.event_tx
			.send(AppEvent::AssistantTurnEnd {
				stop_reason,
			})
			.await;

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

	fn format_json_preview(value: &serde_json::Value) -> String {
		let formatted = match serde_json::to_string_pretty(value) {
			Ok(json) => json,
			Err(_) => value.to_string(),
		};
		Self::truncate_preview(formatted, 240)
	}

	fn truncate_preview(value: String, max_chars: usize) -> String {
		let total_chars = value.chars().count();
		if total_chars <= max_chars {
			return value;
		}

		let mut truncated = value.chars().take(max_chars).collect::<String>();
		truncated.push('…');
		truncated
	}
}
