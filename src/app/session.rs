//! Session — drives the conversation loop between user and LLM.

use std::collections::HashMap;
use std::pin::Pin;

use futures::Stream;
use futures_util::StreamExt;
use tokio::select;

use crate::error::AgentError;
use crate::llm::client::LlmClient;
use crate::llm::request::{ChatRequest, SystemPrompt, ToolChoice};
use crate::llm::response::{Delta, StopReason, StreamEvent};
use crate::llm::types::{CacheControl, ContentBlock, InputMessage, Role};
use crate::tools::ToolRegistry;
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

/// Long-lived session state for the whole interactive conversation.
///
/// Think of this as the "outer loop" owner:
/// - it waits for user actions from the UI
/// - it keeps the full message history across turns
/// - it starts a new assistant turn when the user sends a message
///
/// `TurnState` below is intentionally separate: that enum only tracks the
/// short-lived runtime state for the *current* assistant turn.
pub struct Session {
	/// Transport used to talk to the configured LLM backend.
	client: Box<dyn LlmClient>,
	/// Full conversation history sent back to the model on each request.
	messages: Vec<InputMessage>,
	/// Stable top-level system instruction for this session.
	system_prompt: String,
	/// Max output tokens budget for each model request.
	max_tokens: u32,
	/// Session-side handle for receiving UI actions and sending UI events.
	handle: UiHandle,
	/// Registered tools available to the session.
	///
	/// It is already wired into session construction, but the actual tool loop
	/// is not implemented yet. That upcoming work will consume this registry.
	tool_registry: ToolRegistry,
}

/// Represents a pending tool use request that the model has requested.
struct PendingToolUse {
	/// Unique identifier for this pending tool use request.
	id: String,
	/// Name of the tool being requested.
	name: String,
	/// The input JSON that was passed to the model for this tool use request.
	start_input: serde_json::Value,
	/// The current input JSON for this tool use request.
	input_json: String,
	/// Optional cache control directives for this tool use request, which may affect how the UI displays it.
	cache_control: Option<CacheControl>,
}

/// Represents a content block that is currently being streamed from the model but has not yet completed.
enum PendingBlock {
	/// A text block that is being streamed from the model but has not yet completed.
	Text {
		text: String,
		cache_control: Option<CacheControl>,
	},
	/// A thinking block that is being streamed from the model but has not yet completed.
	Thinking {
		thinking: String,
		signature: Option<String>,
	},
	/// A tool use block that is being streamed from the model but has not yet completed.
	ToolUse(PendingToolUse),
}

/// Short-lived state for the *current* assistant turn only.
///
/// This is the "inner loop" runtime state:
/// - `Idle` means no model stream is active, so the session is waiting for the
///   next user action
/// - `Streaming` means one assistant turn is in progress and we are consuming
///   SSE events from the model
///
/// Keeping this separate from [`Session`] makes the control flow easier to
/// evolve into the later agent/tool loop, where one user turn may contain
/// multiple model -> tool -> model steps.
///
/// Current high-level flow:
///
/// ```text
/// +------+      +------------------+      +------------------------+
/// | UI   | ---> | Session::run()   | ---> | TurnState::Idle        |
/// +------+      +------------------+      +------------------------+
///                      |                           |
///                      | UiAction::SendMessage     |
///                      v                           |
///               +-------------------------+        |
///               | start_streaming_turn()  |        |
///               +-------------------------+        |
///                      |                           |
///                      v                           |
///               +-------------------------+        |
///               | TurnState::Streaming    | -------+
///               | - stream                |
///               | - stop_reason           |
///               | - completed blocks      |
///               | - pending blocks        |
///               +-------------------------+
///                      |
///                      | StreamEvent*
///                      v
///               +-------------------------+
///               | handle_stream_event()   |
///               | - start => pending      |
///               | - delta => update       |
///               | - stop  => finalize     |
///               +-------------------------+
///                      |
///                      | stream ends
///                      v
///               +-------------------------+
///               | finish_streaming_turn() |
///               +-------------------------+
///                      |
///                      v
///               +-------------------------+
///               | TurnState::Idle         |
///               +-------------------------+
/// ```
///
/// Future tool-loop extension point:
///
/// ```text
/// user message
///   -> LLM stream
///   -> assistant tool_use
///   -> dispatch tool_registry
///   -> tool_result message
///   -> LLM stream again
///   -> ... until stop_reason == end_turn
/// ```
enum TurnState {
	Idle,
	Streaming {
		/// The live SSE stream for the assistant turn.
		stream: Pin<Box<dyn Stream<Item = Result<StreamEvent, crate::error::LlmError>> + Send>>,
		/// Updated near the end of the stream from `MessageDelta`.
		/// This tells us whether the model finished normally or stopped for a
		/// special reason such as `tool_use`.
		stop_reason: StopReason,
		/// Tracks the content-block type by stream index because
		/// `ContentBlockStop` only gives us the index.
		block_types: HashMap<u32, BlockType>,
		/// Remembers tool names by tool-use id so later tool-result UI events can
		/// display a stable label.
		tool_names_by_id: HashMap<String, String>,
		/// Completed blocks that we have seen the end of but haven't yet persisted
		completed_assistant_blocks: Vec<ContentBlock>,
		/// Pending blocks that have started but not yet completed. Indexed by stream index.
		pending_blocks_by_index: HashMap<u32, PendingBlock>,
	},
}

impl Session {
	/// Create a new [`Session`] with the given client, tool registry, system prompt, max tokens, and handle.
	pub fn new(
		client: Box<dyn LlmClient>,
		tool_registry: ToolRegistry,
		system_prompt: String,
		max_tokens: u32,
		handle: UiHandle,
	) -> Self {
		Self {
			client,
			tool_registry,
			system_prompt,
			max_tokens,
			handle,
			messages: Vec::new(),
		}
	}

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
					mut stop_reason,
					mut block_types,
					mut tool_names_by_id,
					mut completed_assistant_blocks,
					mut pending_blocks_by_index,
				} => {
					// While a model turn is active we react to two event sources:
					// 1. the next streamed event from the LLM
					// 2. a cancellation/exit action from the UI
					select! {
						maybe_event = stream.next() => {
							let Some(event) = maybe_event else {
								// Stream ended cleanly. Persist the assistant text we
								// assembled during this turn and return to idle.
								self.finish_streaming_turn(Some(completed_assistant_blocks), stop_reason).await?;
								continue;
							};

							match event {
								Ok(event) => {
									self
										.handle_stream_event(
											event,
											&mut stop_reason,
											&mut block_types,
											&mut tool_names_by_id,
											&mut completed_assistant_blocks,
											&mut pending_blocks_by_index,
										)
										.await?;
								}
								Err(e) => {
									let _ = self.handle.event_tx.send(AppEvent::Error(e.to_string())).await;
									self.finish_streaming_turn(Some(completed_assistant_blocks), stop_reason).await?;
									continue;
								}
							}

							TurnState::Streaming {
								stream,
								stop_reason,
								block_types,
								tool_names_by_id,
								completed_assistant_blocks,
								pending_blocks_by_index
							}
						}
						maybe_action = self.handle.action_rx.recv() => {
							match maybe_action {
								Some(UiAction::CancelTurn) => {
									self.finish_streaming_turn(None, StopReason::EndTurn).await?;
									TurnState::Idle
								}
								Some(UiAction::Exit) => {
									self.finish_streaming_turn(Some(completed_assistant_blocks), stop_reason).await?;
									break;
								}
								Some(UiAction::SendMessage(_)) | Some(UiAction::SlashCommand { .. }) => {
									TurnState::Streaming {
										stream,
										stop_reason,
										block_types,
										tool_names_by_id,
										completed_assistant_blocks,
										pending_blocks_by_index
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

		let tool_definitions = self.tool_registry.definitions();
		let has_tools = !tool_definitions.is_empty();
		let tool_choice = has_tools.then_some(ToolChoice::Auto {
			disable_parallel_tool_use: true,
		});
		let tools = has_tools.then_some(tool_definitions);

		// Anthropic's Messages API is stateless on the server side, so each
		// request carries the full conversation history accumulated in `messages`.
		let request = ChatRequest {
			model: self.client.model_info().name.clone(),
			messages: self.messages.clone(),
			system: SystemPrompt::Single(self.system_prompt.clone()),
			max_tokens: Some(self.max_tokens),
			stream: true,
			temperature: None,
			top_p: None,
			tool_choice,
			tools,
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
			stop_reason: StopReason::EndTurn,
			block_types: HashMap::new(),
			tool_names_by_id: HashMap::new(),
			completed_assistant_blocks: Vec::new(),
			pending_blocks_by_index: HashMap::new(),
		})
	}

	async fn handle_stream_event(
		&mut self,
		event: StreamEvent,
		stop_reason: &mut StopReason,
		block_types: &mut HashMap<u32, BlockType>,
		tool_names_by_id: &mut HashMap<String, String>,
		completed_assistant_blocks: &mut Vec<ContentBlock>,
		pending_blocks_by_index: &mut HashMap<u32, PendingBlock>,
	) -> crate::Result<()> {
		// This function translates low-level streaming protocol events into two
		// higher-level effects:
		// 1. update the per-turn runtime state we need to finish the turn
		// 2. forward user-visible events to the UI
		match event {
			StreamEvent::ContentBlockStart {
				index,
				content_block,
			} => match content_block {
				ContentBlock::Text {
					text,
					cache_control,
					..
				} => {
					block_types.insert(index, BlockType::Text);
					pending_blocks_by_index.insert(
						index,
						PendingBlock::Text {
							text,
							cache_control,
						},
					);
				}
				ContentBlock::Thinking {
					thinking,
					signature,
				} => {
					block_types.insert(index, BlockType::Thinking);
					pending_blocks_by_index.insert(
						index,
						PendingBlock::Thinking {
							thinking: thinking.clone(),
							signature,
						},
					);
					let _ = self.handle.event_tx.send(AppEvent::ThinkingDelta(thinking)).await;
				}
				ContentBlock::RedactedThinking {
					data,
				} => {
					block_types.insert(index, BlockType::Thinking);
					completed_assistant_blocks.push(ContentBlock::RedactedThinking {
						data,
					});
				}
				ContentBlock::ToolUse {
					id,
					name,
					input,
					cache_control,
				} => {
					let input_preview = Self::format_json_preview(&input);
					tool_names_by_id.insert(id.clone(), name.clone());
					block_types.insert(index, BlockType::ToolUse);

					pending_blocks_by_index.insert(
						index,
						PendingBlock::ToolUse(PendingToolUse {
							id: id.clone(),
							name: name.clone(),
							start_input: input,
							// input will be appended as deltas arrive
							input_json: String::new(),
							cache_control,
						}),
					);

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
				index,
				delta,
				..
			} => match delta {
				Delta::TextDelta {
					text,
				} => {
					let pending = pending_blocks_by_index.get_mut(&index).ok_or_else(|| {
						AgentError::InvalidState(format!(
							"received text delta for block index {index} without a matching pending block"
						))
					})?;

					match pending {
						PendingBlock::Text {
							text: pending_text,
							..
						} => {
							pending_text.push_str(&text);
						}
						_ => {
							return Err(AgentError::InvalidState(format!(
								"received text delta for block index {index} but pending block is not text"
							))
							.into());
						}
					}

					let _ = self.handle.event_tx.send(AppEvent::TextDelta(text)).await;
				}
				Delta::ThinkingDelta {
					thinking,
				} => {
					if let Some(PendingBlock::Thinking {
						thinking: pending_thinking,
						..
					}) = pending_blocks_by_index.get_mut(&index)
					{
						pending_thinking.push_str(&thinking);
					}

					let _ = self.handle.event_tx.send(AppEvent::ThinkingDelta(thinking)).await;
				}
				Delta::InputJsonDelta {
					partial_json,
				} => {
					let pending = pending_blocks_by_index.get_mut(&index).ok_or_else(|| {
						AgentError::InvalidState(format!(
							"received input JSON delta for block index {index} without a matching pending block"
						))
					})?;

					match pending {
						PendingBlock::ToolUse(pending_tool_use) => {
							pending_tool_use.input_json.push_str(&partial_json);
						}
						_ => {
							return Err(AgentError::InvalidState(format!(
								"received input JSON delta for block index {index} but pending block is not a tool use"
							))
							.into());
						}
					}
				}
				Delta::SignatureDelta {
					signature,
				} => {
					if let Some(PendingBlock::Thinking {
						signature: pending_signature,
						..
					}) = pending_blocks_by_index.get_mut(&index)
					{
						*pending_signature = Some(signature.clone());
					}
				}
			},
			StreamEvent::ContentBlockStop {
				index,
			} => {
				let block_type = block_types.remove(&index).unwrap_or(BlockType::Text);

				if let Some(pending) = pending_blocks_by_index.remove(&index) {
					let block = Self::finalize_pending_block(pending, index)?;
					completed_assistant_blocks.push(block);
				}

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
		assistant_blocks: Option<Vec<ContentBlock>>,
		stop_reason: StopReason,
	) -> crate::Result<()> {
		if let Some(blocks) = assistant_blocks
			&& !blocks.is_empty()
		{
			self.messages.push(InputMessage {
				role: Role::Assistant,
				content: blocks,
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

	fn finalize_pending_block(pending: PendingBlock, index: u32) -> crate::Result<ContentBlock> {
		match pending {
			PendingBlock::Text {
				text,
				cache_control,
			} => Ok(ContentBlock::Text {
				text,
				cache_control,
			}),
			PendingBlock::Thinking {
				thinking,
				signature,
			} => Ok(ContentBlock::Thinking {
				thinking,
				signature,
			}),
			PendingBlock::ToolUse(tool_use) => {
				let input = if tool_use.input_json.is_empty() {
					tool_use.start_input
				} else {
					serde_json::from_str::<serde_json::Value>(&tool_use.input_json).map_err(
						|e| {
							AgentError::MessageFailed(format!(
								"failed to parse final tool input for block {index}: {e}"
							))
						},
					)?
				};

				Ok(ContentBlock::ToolUse {
					id: tool_use.id,
					name: tool_use.name,
					input,
					cache_control: tool_use.cache_control,
				})
			}
		}
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
