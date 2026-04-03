use std::collections::HashMap;

use crate::agent::{DisplayBlock, MessageLevel, SessionEvent, ToolStatus};
use crate::error::AgentError;
use crate::llm::request::{ChatRequest, SystemPrompt, ToolChoice};
use crate::llm::response::{Delta, StopReason, StreamEvent};
use crate::llm::types::{ContentBlock, InputMessage, Role};
use crate::ui::UiAction;

use super::Session;
use super::types::{AssistantStream, PendingBlock, StreamingActionOutcome, TurnState};

impl Session {
	pub(super) async fn start_streaming_turn(&mut self, text: String) -> crate::Result<TurnState> {
		self.messages.push(InputMessage {
			role: Role::User,
			content: vec![ContentBlock::Text {
				text: text.clone(),
				cache_control: None,
			}],
		});
		self.apply_event(SessionEvent::UserMessageSubmitted {
			text,
		})
		.await;
		self.apply_event(SessionEvent::QueryStarted).await;

		self.start_model_stream().await
	}

	pub(super) async fn start_model_stream(&mut self) -> crate::Result<TurnState> {
		let tool_definitions = self.tool_registry.definitions();
		let has_tools = !tool_definitions.is_empty();
		let tool_choice = has_tools.then_some(ToolChoice::Auto {
			disable_parallel_tool_use: true,
		});
		let tools = has_tools.then_some(tool_definitions);

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

		let stream = match self.client.chat_stream(request).await {
			Ok(stream) => stream,
			Err(error) => {
				self.apply_event(SessionEvent::SystemMessageAdded {
					content: error.to_string(),
					level: MessageLevel::Error,
				})
				.await;
				self.apply_event(SessionEvent::QueryCompleted {
					stop_reason: None,
				})
				.await;
				return Ok(TurnState::Idle);
			}
		};

		self.apply_event(SessionEvent::AssistantMessageStarted).await;

		Ok(TurnState::Streaming {
			stream,
			stop_reason: StopReason::EndTurn,
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
					&mut tool_names_by_id,
					&mut server_names_by_id,
					&mut completed_assistant_blocks,
					&mut pending_blocks_by_index,
				)
				.await?;

				Ok(TurnState::Streaming {
					stream,
					stop_reason,
					tool_names_by_id,
					server_names_by_id,
					completed_assistant_blocks,
					pending_blocks_by_index,
				})
			}
			Some(Err(error)) => {
				let assistant_blocks = Self::collect_assistant_blocks(
					completed_assistant_blocks,
					pending_blocks_by_index,
				)?;
				self.persist_assistant_turn(
					assistant_blocks,
					StopReason::EndTurn,
					&tool_names_by_id,
					&server_names_by_id,
				)
				.await?;
				self.apply_event(SessionEvent::SystemMessageAdded {
					content: error.to_string(),
					level: MessageLevel::Error,
				})
				.await;
				self.apply_event(SessionEvent::QueryCompleted {
					stop_reason: None,
				})
				.await;
				Ok(TurnState::Idle)
			}
			None => {
				let assistant_blocks = Self::collect_assistant_blocks(
					completed_assistant_blocks,
					pending_blocks_by_index,
				)?;
				self.continue_after_assistant_turn(
					assistant_blocks,
					stop_reason,
					tool_names_by_id,
					server_names_by_id,
				)
				.await
			}
		}
	}

	#[allow(clippy::too_many_arguments)]
	pub(super) async fn handle_stream_action(
		&mut self,
		maybe_action: Option<UiAction>,
		stream: AssistantStream,
		stop_reason: StopReason,
		tool_names_by_id: HashMap<String, String>,
		server_names_by_id: HashMap<String, String>,
		completed_assistant_blocks: Vec<ContentBlock>,
		pending_blocks_by_index: HashMap<u32, PendingBlock>,
	) -> crate::Result<StreamingActionOutcome> {
		match maybe_action {
			Some(UiAction::CancelTurn) => {
				let assistant_blocks = Self::collect_assistant_blocks(
					completed_assistant_blocks,
					pending_blocks_by_index,
				)?;
				self.persist_assistant_turn(
					assistant_blocks,
					StopReason::EndTurn,
					&tool_names_by_id,
					&server_names_by_id,
				)
				.await?;
				self.apply_event(SessionEvent::InterruptRecorded {
					content: "[Request interrupted by user]".to_string(),
				})
				.await;
				self.apply_event(SessionEvent::QueryCompleted {
					stop_reason: None,
				})
				.await;
				Ok(StreamingActionOutcome::Next(TurnState::Idle))
			}
			Some(UiAction::Exit) => {
				let assistant_blocks = Self::collect_assistant_blocks(
					completed_assistant_blocks,
					pending_blocks_by_index,
				)?;
				self.persist_assistant_turn(
					assistant_blocks,
					stop_reason,
					&tool_names_by_id,
					&server_names_by_id,
				)
				.await?;
				self.apply_event(SessionEvent::QueryCompleted {
					stop_reason: None,
				})
				.await;
				Ok(StreamingActionOutcome::Exit)
			}
			Some(UiAction::SendMessage(_))
			| Some(UiAction::SlashCommand {
				..
			}) => Ok(StreamingActionOutcome::Next(TurnState::Streaming {
				stream,
				stop_reason,
				tool_names_by_id,
				server_names_by_id,
				completed_assistant_blocks,
				pending_blocks_by_index,
			})),
			Some(UiAction::SetScreen(screen)) => {
				self.apply_event(SessionEvent::ScreenChanged {
					screen,
				})
				.await;
				Ok(StreamingActionOutcome::Next(TurnState::Streaming {
					stream,
					stop_reason,
					tool_names_by_id,
					server_names_by_id,
					completed_assistant_blocks,
					pending_blocks_by_index,
				}))
			}
			Some(UiAction::SetTranscriptShowAll(show_all)) => {
				self.apply_event(SessionEvent::TranscriptShowAllChanged {
					show_all,
				})
				.await;
				Ok(StreamingActionOutcome::Next(TurnState::Streaming {
					stream,
					stop_reason,
					tool_names_by_id,
					server_names_by_id,
					completed_assistant_blocks,
					pending_blocks_by_index,
				}))
			}
			None => Ok(StreamingActionOutcome::Exit),
		}
	}

	#[allow(clippy::too_many_arguments)]
	pub(super) async fn handle_stream_event(
		&mut self,
		event: StreamEvent,
		stop_reason: &mut StopReason,
		tool_names_by_id: &mut HashMap<String, String>,
		_server_names_by_id: &mut HashMap<String, String>,
		completed_assistant_blocks: &mut Vec<ContentBlock>,
		pending_blocks_by_index: &mut HashMap<u32, PendingBlock>,
	) -> crate::Result<()> {
		match event {
			StreamEvent::ContentBlockStart {
				index,
				content_block,
			} => match content_block {
				ContentBlock::Text {
					text,
					cache_control,
				} => {
					if !text.is_empty() {
						self.apply_event(SessionEvent::AssistantTextDelta {
							text: text.clone(),
						})
						.await;
					}
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
					if !thinking.is_empty() {
						self.apply_event(SessionEvent::AssistantThinkingDelta {
							text: thinking.clone(),
						})
						.await;
					}
					pending_blocks_by_index.insert(
						index,
						PendingBlock::Thinking {
							thinking,
							signature,
						},
					);
				}
				ContentBlock::RedactedThinking {
					data,
				} => {
					self.apply_event(SessionEvent::AssistantRedactedThinking {
						text: data.clone(),
					})
					.await;
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
					tool_names_by_id.insert(id.clone(), name.clone());
					pending_blocks_by_index.insert(
						index,
						PendingBlock::ToolUse(super::types::PendingToolUse {
							id: id.clone(),
							name: name.clone(),
							start_input: input.clone(),
							input_json: serde_json::to_string(&input).unwrap_or_default(),
							cache_control,
						}),
					);
					self.apply_event(SessionEvent::AssistantToolUseStarted {
						id,
						name,
						server_name: None,
						input,
					})
					.await;
				}
				ContentBlock::ToolResult {
					tool_use_id,
					content,
					is_error,
				} => {
					completed_assistant_blocks.push(ContentBlock::ToolResult {
						tool_use_id,
						content: content.clone(),
						is_error,
					});
				}
			},
			StreamEvent::ContentBlockDelta {
				index,
				delta,
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
					self.apply_event(SessionEvent::AssistantTextDelta {
						text,
					})
					.await;
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
					self.apply_event(SessionEvent::AssistantThinkingDelta {
						text: thinking,
					})
					.await;
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
							self.apply_event(SessionEvent::AssistantToolUseInputJsonDelta {
								id: pending_tool_use.id.clone(),
								partial_json,
							})
							.await;
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
						*pending_signature = Some(signature);
					}
				}
			},
			StreamEvent::ContentBlockStop {
				index,
			} => {
				if let Some(pending) = pending_blocks_by_index.remove(&index) {
					completed_assistant_blocks.push(Self::finalize_pending_block(pending, index)?);
				}
			}
			StreamEvent::MessageDelta {
				delta,
				usage,
			} => {
				if let Some(reason) = delta.stop_reason {
					*stop_reason = reason;
				}
				self.apply_event(SessionEvent::UsageUpdated {
					input_tokens: usage.input_tokens,
					output_tokens: usage.output_tokens,
				})
				.await;
			}
			StreamEvent::Error {
				error,
			} => {
				self.apply_event(SessionEvent::SystemMessageAdded {
					content: error.to_string(),
					level: MessageLevel::Error,
				})
				.await;
			}
			StreamEvent::MessageStart {
				..
			}
			| StreamEvent::MessageStop
			| StreamEvent::Ping => {}
		}

		Ok(())
	}

	pub(super) async fn persist_assistant_turn(
		&mut self,
		assistant_blocks: Vec<ContentBlock>,
		stop_reason: StopReason,
		tool_names_by_id: &HashMap<String, String>,
		server_names_by_id: &HashMap<String, String>,
	) -> crate::Result<()> {
		if !assistant_blocks.is_empty() {
			self.messages.push(InputMessage {
				role: Role::Assistant,
				content: assistant_blocks.clone(),
			});
		}

		self.apply_event(SessionEvent::AssistantMessageCommitted {
			blocks: Self::display_blocks_from_content_blocks(
				assistant_blocks.as_slice(),
				tool_names_by_id,
				server_names_by_id,
			),
			stop_reason,
		})
		.await;

		Ok(())
	}

	pub(super) fn collect_assistant_blocks(
		mut completed_assistant_blocks: Vec<ContentBlock>,
		pending_blocks_by_index: HashMap<u32, PendingBlock>,
	) -> crate::Result<Vec<ContentBlock>> {
		if pending_blocks_by_index.is_empty() {
			return Ok(completed_assistant_blocks);
		}

		let mut pending = pending_blocks_by_index.into_iter().collect::<Vec<_>>();
		pending.sort_by_key(|(index, _)| *index);
		for (index, block) in pending {
			completed_assistant_blocks.push(Self::finalize_pending_block(block, index)?);
		}

		Ok(completed_assistant_blocks)
	}

	pub(super) fn display_blocks_from_content_blocks(
		assistant_blocks: &[ContentBlock],
		tool_names_by_id: &HashMap<String, String>,
		server_names_by_id: &HashMap<String, String>,
	) -> Vec<DisplayBlock> {
		assistant_blocks
			.iter()
			.map(|block| match block {
				ContentBlock::Text {
					text,
					..
				} => DisplayBlock::Text(text.clone()),
				ContentBlock::Thinking {
					thinking,
					..
				} => DisplayBlock::Thinking(thinking.clone()),
				ContentBlock::RedactedThinking {
					data,
				} => DisplayBlock::RedactedThinking(data.clone()),
				ContentBlock::ToolUse {
					id,
					name,
					input,
					..
				} => DisplayBlock::ToolUse {
					id: id.clone(),
					name: name.clone(),
					server_name: server_names_by_id.get(id).cloned(),
					input: input.clone(),
					input_json: serde_json::to_string(input).unwrap_or_default(),
					input_preview: Self::format_json_preview(input),
					status: ToolStatus::Pending,
				},
				ContentBlock::ToolResult {
					tool_use_id,
					content,
					is_error,
				} => DisplayBlock::ToolResult {
					tool_use_id: tool_use_id.clone(),
					name: tool_names_by_id
						.get(tool_use_id)
						.cloned()
						.unwrap_or_else(|| "tool".to_string()),
					server_name: server_names_by_id.get(tool_use_id).cloned(),
					output: content.clone(),
					is_error: is_error.unwrap_or(false),
				},
			})
			.collect()
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
	async fn handle_stream_event_text_block_updates_store() {
		let mut session = new_test_session();
		let mut stop_reason = StopReason::EndTurn;
		let mut tool_names_by_id = HashMap::new();
		let mut server_names_by_id = HashMap::new();
		let mut completed_assistant_blocks = Vec::new();
		let mut pending_blocks_by_index = HashMap::new();

		session.apply_event(SessionEvent::QueryStarted).await;
		session.apply_event(SessionEvent::AssistantMessageStarted).await;
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
					index: 0,
					delta: Delta::TextDelta {
						text: "hello".to_string(),
					},
				},
				&mut stop_reason,
				&mut tool_names_by_id,
				&mut server_names_by_id,
				&mut completed_assistant_blocks,
				&mut pending_blocks_by_index,
			)
			.await
			.unwrap();

		let view = session.store.view();
		assert!(matches!(
			view.messages.last().and_then(|msg| msg.blocks.last()),
			Some(DisplayBlock::Text(text)) if text == "hello"
		));
	}

	#[test]
	fn collect_assistant_blocks_finalizes_pending_blocks() {
		let blocks = Session::collect_assistant_blocks(
			Vec::new(),
			HashMap::from([(
				0,
				PendingBlock::Text {
					text: "partial".to_string(),
					cache_control: None,
				},
			)]),
		)
		.unwrap();

		assert_eq!(blocks.len(), 1);
		assert!(matches!(blocks[0], ContentBlock::Text { ref text, .. } if text == "partial"));
	}
}
