use crate::error::AgentError;
use crate::llm::response::StopReason;
use crate::llm::types::{ContentBlock, InputMessage, Role};
use crate::tools::ToolCall;

use super::Session;
use super::types::{PendingBlock, PendingDispatch, TurnState};

impl Session {
	pub(super) async fn continue_after_assistant_turn(
		&mut self,
		assistant_blocks: Vec<ContentBlock>,
		stop_reason: StopReason,
	) -> crate::Result<TurnState> {
		let _ = self
			.handle
			.event_tx
			.send(crate::ui::events::AppEvent::AssistantTurnEnd {
				stop_reason: stop_reason.clone(),
			})
			.await;

		let tool_calls = Self::extract_tool_calls(assistant_blocks.as_slice());

		if !assistant_blocks.is_empty() {
			self.messages.push(InputMessage {
				role: Role::Assistant,
				content: assistant_blocks,
			});
		}

		match stop_reason {
			StopReason::ToolUse => {
				if tool_calls.is_empty() {
					return Err(AgentError::InvalidState(
						"LLM indicated it wants to use a tool but did not provide any tool calls"
							.to_string(),
					)
					.into());
				}

				let mut content = Vec::new();
				for dispatch in tool_calls {
					let tool_result =
						self.execute_tool_call(dispatch.tool_use_id, dispatch.call).await?;
					content.push(tool_result);
				}
				self.messages.push(InputMessage {
					role: Role::User,
					content,
				});

				self.start_model_stream().await
			}
			_ => Ok(TurnState::Idle),
		}
	}

	pub(super) fn finalize_pending_block(
		pending: PendingBlock,
		index: u32,
	) -> crate::Result<ContentBlock> {
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

	/// Executes the given tool call and returns a content block with the result to be appended to the assistant message.
	pub(super) async fn execute_tool_call(
		&self,
		tool_use_id: String,
		call: ToolCall,
	) -> crate::Result<ContentBlock> {
		let result = match self.tool_registry.dispatch(&call).await {
			Ok(result) => result,
			Err(e) => {
				return Ok(ContentBlock::ToolResult {
					tool_use_id,
					content: e.to_string(),
					is_error: Some(true),
				});
			}
		};

		let (output, is_error) = match result.output {
			Ok(output) => (output, Some(false)),
			Err(e) => (e, Some(true)),
		};

		Ok(ContentBlock::ToolResult {
			tool_use_id,
			content: output,
			is_error,
		})
	}

	/// Extracts tool calls from the given content blocks, which we can then dispatch to the tool registry.
	pub(super) fn extract_tool_calls(blocks: &[ContentBlock]) -> Vec<PendingDispatch> {
		blocks
			.iter()
			.filter_map(|block| {
				if let ContentBlock::ToolUse {
					id,
					name,
					input,
					..
				} = block
				{
					let call = ToolCall {
						name: name.clone(),
						args: input.clone(),
					};
					Some(PendingDispatch {
						tool_use_id: id.clone(),
						call,
					})
				} else {
					None
				}
			})
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use std::pin::Pin;
	use std::sync::{Arc, Mutex};

	use futures::Stream;

	use super::*;
	use crate::app::session::types::PendingToolUse;
	use crate::error::LlmError;
	use crate::llm::client::{LlmClient, ModelInfo};
	use crate::llm::request::ChatRequest;
	use crate::llm::response::{ChatResponse, StreamEvent};
	use crate::tools::ToolRegistry;
	use crate::tools::builtin::EchoTool;

	struct StubClient;

	struct RecordingClient {
		requests: Arc<Mutex<Vec<ChatRequest>>>,
	}

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

	#[async_trait::async_trait]
	impl LlmClient for RecordingClient {
		async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, LlmError> {
			panic!("chat() is not used in this test");
		}

		async fn chat_stream(
			&self,
			request: ChatRequest,
		) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>, LlmError> {
			self.requests.lock().unwrap().push(request);
			Ok(Box::pin(futures::stream::empty()))
		}

		fn model_info(&self) -> ModelInfo {
			ModelInfo {
				name: "test-model".to_string(),
				max_context_length: 200_000,
				max_response_tokens: 4_096,
			}
		}
	}

	#[tokio::test]
	async fn execute_tool_call_wraps_dispatch_error_as_tool_result() {
		let registry = ToolRegistry::new();
		let (_ui_channels, ui_handle) = crate::ui::create_ui_channels();
		let session = Session::new(
			Box::new(StubClient),
			registry,
			"test system prompt".to_string(),
			1_024,
			ui_handle,
		);

		let block = session
			.execute_tool_call(
				"toolu_test".to_string(),
				ToolCall {
					name: "missing_tool".to_string(),
					args: serde_json::json!({}),
				},
			)
			.await
			.expect("dispatch errors should be wrapped as tool_result blocks");

		match block {
			ContentBlock::ToolResult {
				tool_use_id,
				content,
				is_error,
			} => {
				assert_eq!(tool_use_id, "toolu_test");
				assert_eq!(is_error, Some(true));
				assert!(content.contains("tool not found"));
			}
			other => panic!("expected ToolResult, got: {other:?}"),
		}
	}

	#[tokio::test]
	async fn continue_after_tool_use_dispatches_tool_and_starts_next_stream() {
		let requests = Arc::new(Mutex::new(Vec::<ChatRequest>::new()));
		let client = RecordingClient {
			requests: Arc::clone(&requests),
		};

		let mut registry = ToolRegistry::new();
		registry.register(EchoTool);

		let (_ui_channels, ui_handle) = crate::ui::create_ui_channels();
		let mut session = Session::new(
			Box::new(client),
			registry,
			"test system prompt".to_string(),
			1_024,
			ui_handle,
		);

		let next_state = session
			.continue_after_assistant_turn(
				vec![ContentBlock::ToolUse {
					id: "toolu_test".to_string(),
					name: "echo".to_string(),
					input: serde_json::json!({
						"message": "hello from tool"
					}),
					cache_control: None,
				}],
				StopReason::ToolUse,
			)
			.await
			.expect("tool_use should continue into another model stream");

		assert!(matches!(next_state, TurnState::Streaming { .. }));

		let requests = requests.lock().unwrap();
		assert_eq!(requests.len(), 1);

		let request = &requests[0];
		assert_eq!(request.messages.len(), 2);

		match &request.messages[0].role {
			Role::Assistant => {}
			other => panic!("expected assistant role, got: {other:?}"),
		}

		match &request.messages[0].content[0] {
			ContentBlock::ToolUse {
				id,
				name,
				input,
				..
			} => {
				assert_eq!(id, "toolu_test");
				assert_eq!(name, "echo");
				assert_eq!(input["message"], "hello from tool");
			}
			other => panic!("expected ToolUse block, got: {other:?}"),
		}

		match &request.messages[1].role {
			Role::User => {}
			other => panic!("expected user role for tool result, got: {other:?}"),
		}

		match &request.messages[1].content[0] {
			ContentBlock::ToolResult {
				tool_use_id,
				content,
				is_error,
			} => {
				assert_eq!(tool_use_id, "toolu_test");
				assert_eq!(content, "hello from tool");
				assert_eq!(*is_error, Some(false));
			}
			other => panic!("expected ToolResult block, got: {other:?}"),
		}
	}

	#[tokio::test]
	async fn continue_after_tool_use_wraps_dispatch_error_and_starts_next_stream() {
		let requests = Arc::new(Mutex::new(Vec::<ChatRequest>::new()));
		let client = RecordingClient {
			requests: Arc::clone(&requests),
		};

		let registry = ToolRegistry::new();
		let (_ui_channels, ui_handle) = crate::ui::create_ui_channels();
		let mut session = Session::new(
			Box::new(client),
			registry,
			"test system prompt".to_string(),
			1_024,
			ui_handle,
		);

		let next_state = session
			.continue_after_assistant_turn(
				vec![ContentBlock::ToolUse {
					id: "toolu_missing".to_string(),
					name: "missing_tool".to_string(),
					input: serde_json::json!({}),
					cache_control: None,
				}],
				StopReason::ToolUse,
			)
			.await
			.expect("dispatch failures should be wrapped and still continue");

		assert!(matches!(next_state, TurnState::Streaming { .. }));

		let requests = requests.lock().unwrap();
		assert_eq!(requests.len(), 1);

		let request = &requests[0];
		assert_eq!(request.messages.len(), 2);

		match &request.messages[1].content[0] {
			ContentBlock::ToolResult {
				tool_use_id,
				content,
				is_error,
			} => {
				assert_eq!(tool_use_id, "toolu_missing");
				assert_eq!(*is_error, Some(true));
				assert!(content.contains("tool not found"));
			}
			other => panic!("expected ToolResult block, got: {other:?}"),
		}
	}

	#[tokio::test]
	async fn continue_after_end_turn_returns_idle_without_starting_new_stream() {
		let requests = Arc::new(Mutex::new(Vec::<ChatRequest>::new()));
		let client = RecordingClient {
			requests: Arc::clone(&requests),
		};

		let mut registry = ToolRegistry::new();
		registry.register(EchoTool);

		let (_ui_channels, ui_handle) = crate::ui::create_ui_channels();
		let mut session = Session::new(
			Box::new(client),
			registry,
			"test system prompt".to_string(),
			1_024,
			ui_handle,
		);

		let next_state = session
			.continue_after_assistant_turn(
				vec![ContentBlock::Text {
					text: "final answer".to_string(),
					cache_control: None,
				}],
				StopReason::EndTurn,
			)
			.await
			.expect("end_turn should return to idle");

		assert!(matches!(next_state, TurnState::Idle));
		assert!(requests.lock().unwrap().is_empty());
		assert_eq!(session.messages.len(), 1);

		match &session.messages[0].role {
			Role::Assistant => {}
			other => panic!("expected assistant role, got: {other:?}"),
		}

		match &session.messages[0].content[0] {
			ContentBlock::Text {
				text,
				..
			} => assert_eq!(text, "final answer"),
			other => panic!("expected Text block, got: {other:?}"),
		}
	}

	#[test]
	fn finalize_pending_block_tool_use_falls_back_to_start_input() {
		let block = Session::finalize_pending_block(
			PendingBlock::ToolUse(PendingToolUse {
				id: "toolu_test".to_string(),
				name: "echo".to_string(),
				start_input: serde_json::json!({
					"message": "fallback"
				}),
				input_json: String::new(),
				cache_control: None,
			}),
			3,
		)
		.expect("empty input_json should fall back to start_input");

		match block {
			ContentBlock::ToolUse {
				id,
				name,
				input,
				..
			} => {
				assert_eq!(id, "toolu_test");
				assert_eq!(name, "echo");
				assert_eq!(input["message"], "fallback");
			}
			other => panic!("expected ToolUse block, got: {other:?}"),
		}
	}

	#[test]
	fn finalize_pending_block_tool_use_reports_parse_error() {
		let err = Session::finalize_pending_block(
			PendingBlock::ToolUse(PendingToolUse {
				id: "toolu_test".to_string(),
				name: "echo".to_string(),
				start_input: serde_json::json!({}),
				input_json: "{not json".to_string(),
				cache_control: None,
			}),
			7,
		)
		.expect_err("invalid JSON should fail to parse");

		match err {
			crate::Error::Agent(AgentError::MessageFailed(msg)) => {
				assert!(msg.contains("failed to parse final tool input for block 7"));
			}
			other => panic!("expected AgentError::MessageFailed, got: {other:?}"),
		}
	}
}
