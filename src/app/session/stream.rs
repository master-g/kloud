use std::collections::HashMap;

use crate::error::AgentError;
use crate::llm::request::{ChatRequest, SystemPrompt, ToolChoice};
use crate::llm::response::{Delta, StopReason, StreamEvent};
use crate::llm::types::{ContentBlock, InputMessage, Role};
use crate::ui::UiAction;
use crate::ui::events::{AppEvent, BlockType};

use super::Session;
use super::types::{AssistantStream, PendingBlock, StreamingActionOutcome, TurnState};

impl Session {
	pub(super) async fn start_streaming_turn(&mut self, text: String) -> crate::Result<TurnState> {
		// Append user message to history.
		self.messages.push(InputMessage {
			role: Role::User,
			content: vec![ContentBlock::Text {
				text,
				cache_control: None,
			}],
		});

		self.start_model_stream().await
	}

	pub(super) async fn start_model_stream(&mut self) -> crate::Result<TurnState> {
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
			server_names_by_id: HashMap::new(),
			completed_assistant_blocks: Vec::new(),
			pending_blocks_by_index: HashMap::new(),
		})
	}

	#[allow(clippy::too_many_arguments)]
	pub(super) async fn handle_stream_poll(
		&mut self,
		maybe_event: Option<Result<StreamEvent, crate::error::LlmError>>,
		stream: AssistantStream,
		mut stop_reason: StopReason,
		mut block_types: HashMap<u32, BlockType>,
		mut tool_names_by_id: HashMap<String, String>,
		mut server_names_by_id: HashMap<String, String>,
		mut completed_assistant_blocks: Vec<ContentBlock>,
		mut pending_blocks_by_index: HashMap<u32, PendingBlock>,
	) -> crate::Result<TurnState> {
		match maybe_event {
			Some(Ok(event)) => {
				self.handle_stream_event(
					event,
					&mut stop_reason,
					&mut block_types,
					&mut tool_names_by_id,
					&mut server_names_by_id,
					&mut completed_assistant_blocks,
					&mut pending_blocks_by_index,
				)
				.await?;

				Ok(TurnState::Streaming {
					stream,
					stop_reason,
					block_types,
					tool_names_by_id,
					server_names_by_id,
					completed_assistant_blocks,
					pending_blocks_by_index,
				})
			}
			Some(Err(e)) => {
				let _ = self.handle.event_tx.send(AppEvent::Error(e.to_string())).await;
				self.finish_streaming_turn(Some(completed_assistant_blocks), stop_reason).await?;
				Ok(TurnState::Idle)
			}
			None => {
				self.continue_after_assistant_turn(completed_assistant_blocks, stop_reason).await
			}
		}
	}

	#[allow(clippy::too_many_arguments)]
	pub(super) async fn handle_stream_action(
		&mut self,
		maybe_action: Option<UiAction>,
		stream: AssistantStream,
		stop_reason: StopReason,
		block_types: HashMap<u32, BlockType>,
		tool_names_by_id: HashMap<String, String>,
		server_names_by_id: HashMap<String, String>,
		completed_assistant_blocks: Vec<ContentBlock>,
		pending_blocks_by_index: HashMap<u32, PendingBlock>,
	) -> crate::Result<StreamingActionOutcome> {
		match maybe_action {
			Some(UiAction::CancelTurn) => {
				self.finish_streaming_turn(None, StopReason::EndTurn).await?;
				Ok(StreamingActionOutcome::Next(TurnState::Idle))
			}
			Some(UiAction::Exit) => {
				self.finish_streaming_turn(Some(completed_assistant_blocks), stop_reason).await?;
				Ok(StreamingActionOutcome::Exit)
			}
			Some(UiAction::SendMessage(_))
			| Some(UiAction::SlashCommand {
				..
			}) => Ok(StreamingActionOutcome::Next(TurnState::Streaming {
				stream,
				stop_reason,
				block_types,
				tool_names_by_id,
				server_names_by_id,
				completed_assistant_blocks,
				pending_blocks_by_index,
			})),
			None => Ok(StreamingActionOutcome::Exit),
		}
	}

	#[allow(clippy::too_many_arguments)]
	pub(super) async fn handle_stream_event(
		&mut self,
		event: StreamEvent,
		stop_reason: &mut StopReason,
		block_types: &mut HashMap<u32, BlockType>,
		tool_names_by_id: &mut HashMap<String, String>,
		server_names_by_id: &mut HashMap<String, String>,
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
					let _ =
						self.handle.event_tx.send(AppEvent::RedactedThinking(data.clone())).await;
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
						PendingBlock::ToolUse(super::types::PendingToolUse {
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
							server_name: None,
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
					let server_name = server_names_by_id.remove(&tool_use_id);
					block_types.insert(index, BlockType::ToolResult);
					let _ = self
						.handle
						.event_tx
						.send(AppEvent::ToolResult {
							id: tool_use_id,
							name,
							server_name,
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
						} => pending_text.push_str(&text),
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

	pub(super) async fn finish_streaming_turn(
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

#[cfg(test)]
mod tests {
	use std::pin::Pin;

	use futures::Stream;

	use super::*;
	use crate::error::LlmError;
	use crate::llm::client::{LlmClient, ModelInfo};
	use crate::llm::request::ChatRequest;
	use crate::llm::response::ChatResponse;
	use crate::tools::ToolRegistry;

	struct StubClient;

	#[async_trait::async_trait]
	impl LlmClient for StubClient {
		async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, LlmError> {
			panic!("chat() is not used in this test");
		}

		async fn chat_stream(
			&self,
			_request: ChatRequest,
		) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>, LlmError> {
			panic!("chat_stream() is not used in this test");
		}

		fn model_info(&self) -> ModelInfo {
			ModelInfo {
				name: "test-model".to_string(),
				max_context_length: 200_000,
				max_response_tokens: 4_096,
			}
		}
	}

	fn new_test_session() -> Session {
		let (_ui_channels, ui_handle) = crate::ui::create_ui_channels();
		Session::new(
			Box::new(StubClient),
			ToolRegistry::new(),
			"test system prompt".to_string(),
			1_024,
			ui_handle,
		)
	}

	#[tokio::test]
	async fn handle_stream_event_text_block_lifecycle() {
		let mut session = new_test_session();
		let mut stop_reason = StopReason::EndTurn;
		let mut block_types = HashMap::new();
		let mut tool_names_by_id = HashMap::new();
		let mut server_names_by_id = HashMap::new();
		let mut completed_assistant_blocks = Vec::new();
		let mut pending_blocks_by_index = HashMap::new();

		session
			.handle_stream_event(
				StreamEvent::ContentBlockStart {
					index: 0,
					content_block: ContentBlock::Text {
						text: String::new(),
						cache_control: None,
					},
				},
				&mut stop_reason,
				&mut block_types,
				&mut tool_names_by_id,
				&mut server_names_by_id,
				&mut completed_assistant_blocks,
				&mut pending_blocks_by_index,
			)
			.await
			.unwrap();

		assert!(matches!(
			pending_blocks_by_index.get(&0),
			Some(PendingBlock::Text { text, .. }) if text.is_empty()
		));

		session
			.handle_stream_event(
				StreamEvent::ContentBlockDelta {
					index: 0,
					delta: Delta::TextDelta {
						text: "hello".to_string(),
					},
				},
				&mut stop_reason,
				&mut block_types,
				&mut tool_names_by_id,
				&mut server_names_by_id,
				&mut completed_assistant_blocks,
				&mut pending_blocks_by_index,
			)
			.await
			.unwrap();

		session
			.handle_stream_event(
				StreamEvent::ContentBlockStop {
					index: 0,
				},
				&mut stop_reason,
				&mut block_types,
				&mut tool_names_by_id,
				&mut server_names_by_id,
				&mut completed_assistant_blocks,
				&mut pending_blocks_by_index,
			)
			.await
			.unwrap();

		assert!(pending_blocks_by_index.is_empty());
		assert!(block_types.is_empty());
		assert_eq!(completed_assistant_blocks.len(), 1);
		match &completed_assistant_blocks[0] {
			ContentBlock::Text {
				text,
				..
			} => assert_eq!(text, "hello"),
			other => panic!("expected Text block, got: {other:?}"),
		}
	}

	#[tokio::test]
	async fn handle_stream_event_tool_use_block_lifecycle_with_input_json_delta() {
		let mut session = new_test_session();
		let mut stop_reason = StopReason::EndTurn;
		let mut block_types = HashMap::new();
		let mut tool_names_by_id = HashMap::new();
		let mut server_names_by_id = HashMap::new();
		let mut completed_assistant_blocks = Vec::new();
		let mut pending_blocks_by_index = HashMap::new();

		session
			.handle_stream_event(
				StreamEvent::ContentBlockStart {
					index: 1,
					content_block: ContentBlock::ToolUse {
						id: "toolu_test".to_string(),
						name: "echo".to_string(),
						input: serde_json::json!({}),
						cache_control: None,
					},
				},
				&mut stop_reason,
				&mut block_types,
				&mut tool_names_by_id,
				&mut server_names_by_id,
				&mut completed_assistant_blocks,
				&mut pending_blocks_by_index,
			)
			.await
			.unwrap();

		session
			.handle_stream_event(
				StreamEvent::ContentBlockDelta {
					index: 1,
					delta: Delta::InputJsonDelta {
						partial_json: r#"{"message":"hi"}"#.to_string(),
					},
				},
				&mut stop_reason,
				&mut block_types,
				&mut tool_names_by_id,
				&mut server_names_by_id,
				&mut completed_assistant_blocks,
				&mut pending_blocks_by_index,
			)
			.await
			.unwrap();

		session
			.handle_stream_event(
				StreamEvent::ContentBlockStop {
					index: 1,
				},
				&mut stop_reason,
				&mut block_types,
				&mut tool_names_by_id,
				&mut server_names_by_id,
				&mut completed_assistant_blocks,
				&mut pending_blocks_by_index,
			)
			.await
			.unwrap();

		assert!(pending_blocks_by_index.is_empty());
		assert_eq!(completed_assistant_blocks.len(), 1);
		match &completed_assistant_blocks[0] {
			ContentBlock::ToolUse {
				id,
				name,
				input,
				..
			} => {
				assert_eq!(id, "toolu_test");
				assert_eq!(name, "echo");
				assert_eq!(input["message"], "hi");
			}
			other => panic!("expected ToolUse block, got: {other:?}"),
		}
	}

	#[tokio::test]
	async fn handle_stream_event_thinking_block_lifecycle_with_signature() {
		let mut session = new_test_session();
		let mut stop_reason = StopReason::EndTurn;
		let mut block_types = HashMap::new();
		let mut tool_names_by_id = HashMap::new();
		let mut server_names_by_id = HashMap::new();
		let mut completed_assistant_blocks = Vec::new();
		let mut pending_blocks_by_index = HashMap::new();

		session
			.handle_stream_event(
				StreamEvent::ContentBlockStart {
					index: 2,
					content_block: ContentBlock::Thinking {
						thinking: String::new(),
						signature: None,
					},
				},
				&mut stop_reason,
				&mut block_types,
				&mut tool_names_by_id,
				&mut server_names_by_id,
				&mut completed_assistant_blocks,
				&mut pending_blocks_by_index,
			)
			.await
			.unwrap();

		session
			.handle_stream_event(
				StreamEvent::ContentBlockDelta {
					index: 2,
					delta: Delta::ThinkingDelta {
						thinking: "reason".to_string(),
					},
				},
				&mut stop_reason,
				&mut block_types,
				&mut tool_names_by_id,
				&mut server_names_by_id,
				&mut completed_assistant_blocks,
				&mut pending_blocks_by_index,
			)
			.await
			.unwrap();

		session
			.handle_stream_event(
				StreamEvent::ContentBlockDelta {
					index: 2,
					delta: Delta::SignatureDelta {
						signature: "sig".to_string(),
					},
				},
				&mut stop_reason,
				&mut block_types,
				&mut tool_names_by_id,
				&mut server_names_by_id,
				&mut completed_assistant_blocks,
				&mut pending_blocks_by_index,
			)
			.await
			.unwrap();

		session
			.handle_stream_event(
				StreamEvent::ContentBlockStop {
					index: 2,
				},
				&mut stop_reason,
				&mut block_types,
				&mut tool_names_by_id,
				&mut server_names_by_id,
				&mut completed_assistant_blocks,
				&mut pending_blocks_by_index,
			)
			.await
			.unwrap();

		assert!(pending_blocks_by_index.is_empty());
		assert_eq!(completed_assistant_blocks.len(), 1);
		match &completed_assistant_blocks[0] {
			ContentBlock::Thinking {
				thinking,
				signature,
			} => {
				assert_eq!(thinking, "reason");
				assert_eq!(signature.as_deref(), Some("sig"));
			}
			other => panic!("expected Thinking block, got: {other:?}"),
		}
	}
}
