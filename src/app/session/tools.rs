use crate::agent::SessionEvent;
use crate::error::AgentError;
use crate::llm::response::StopReason;
use crate::llm::types::{ContentBlock, InputMessage, Role};
use crate::tools::{ToolCall, ToolResultKind};

use super::Session;
use super::types::{PendingBlock, PendingDispatch, TurnState};

impl Session {
    pub(super) async fn continue_after_assistant_turn(
        &mut self,
        assistant_blocks: Vec<ContentBlock>,
        stop_reason: StopReason,
        tool_names_by_id: std::collections::HashMap<String, String>,
        server_names_by_id: std::collections::HashMap<String, String>,
    ) -> crate::Result<TurnState> {
        self.persist_assistant_turn(
            assistant_blocks.clone(),
            stop_reason.clone(),
            &tool_names_by_id,
            &server_names_by_id,
        )
        .await?;

        let tool_calls = Self::extract_tool_calls(assistant_blocks.as_slice());

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
                    let tool_ref = self.tool_registry.get(&dispatch.call.name);
                    let rendered_use = tool_ref
                        .map(|tool| tool.render_tool_use_message(&dispatch.call.args, &self.theme))
                        .unwrap_or_default();
                    let display_name = tool_ref
                        .map(|tool| tool.user_facing_name())
                        .unwrap_or_else(|| dispatch.call.name.clone());

                    self.apply_event(SessionEvent::ToolExecutionStarted {
                        id: dispatch.tool_use_id.clone(),
                        name: dispatch.call.name.clone(),
                        display_name,
                        server_name: None,
                        rendered_use,
                    })
                    .await;
                    let tool_result =
                        self.execute_tool_call(dispatch.tool_use_id.clone(), dispatch.call).await?;
                    let (name, server_name, output, is_error) = match &tool_result {
                        ContentBlock::ToolResult {
                            tool_use_id: _,
                            content,
                            is_error,
                        } => (
                            tool_names_by_id
                                .get(&dispatch.tool_use_id)
                                .cloned()
                                .unwrap_or_else(|| "tool".to_string()),
                            server_names_by_id.get(&dispatch.tool_use_id).cloned(),
                            content.clone(),
                            is_error.unwrap_or(false),
                        ),
                        _ => {
                            return Err(AgentError::InvalidState(
                                "tool execution did not produce a tool result block".to_string(),
                            )
                            .into());
                        }
                    };

                    let result_kind = if is_error {
                        ToolResultKind::Error
                    } else {
                        ToolResultKind::Success
                    };

                    let rendered_result = self
                        .tool_registry
                        .get(&name)
                        .map(|tool| {
                            tool.render_tool_result_message(&output, result_kind, &self.theme)
                        })
                        .unwrap_or_default();

                    self.apply_event(SessionEvent::ToolExecutionFinished {
                        id: dispatch.tool_use_id.clone(),
                        name,
                        server_name,
                        output,
                        result_kind,
                        rendered_result,
                    })
                    .await;
                    content.push(tool_result);
                }

                self.messages.push(InputMessage {
                    role: Role::User,
                    content,
                });

                self.start_model_stream().await
            }
            _ => {
                self.apply_event(SessionEvent::QueryCompleted {
                    stop_reason: Some(stop_reason),
                })
                .await;
                Ok(TurnState::Idle)
            }
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
                        |error| {
                            AgentError::MessageFailed(format!(
                                "failed to parse final tool input for block {index}: {error}"
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

    pub(super) async fn execute_tool_call(
        &self,
        tool_use_id: String,
        call: ToolCall,
    ) -> crate::Result<ContentBlock> {
        let result = match self.tool_registry.dispatch(&call).await {
            Ok(result) => result,
            Err(error) => {
                return Ok(ContentBlock::ToolResult {
                    tool_use_id,
                    content: error.to_string(),
                    is_error: Some(true),
                });
            }
        };

        let is_error = matches!(result.kind, ToolResultKind::Error);

        Ok(ContentBlock::ToolResult {
            tool_use_id,
            content: result.output,
            is_error: Some(is_error),
        })
    }

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
                    Some(PendingDispatch {
                        tool_use_id: id.clone(),
                        call: ToolCall {
                            name: name.clone(),
                            args: input.clone(),
                        },
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
    use crate::agent::DisplayBlock;
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
        ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>, LlmError>
        {
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
        ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>, LlmError>
        {
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
    async fn continue_after_tool_use_dispatches_tool_and_keeps_query_running() {
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

        session.apply_event(SessionEvent::QueryStarted).await;

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
                std::collections::HashMap::from([("toolu_test".to_string(), "echo".to_string())]),
                std::collections::HashMap::new(),
            )
            .await
            .expect("tool_use should continue into another model stream");

        assert!(matches!(next_state, TurnState::Streaming { .. }));
        assert!(matches!(
            session.store.view().status,
            crate::agent::view::AssistantStatus::Streaming
        ));
        assert!(session.store.view().messages.iter().any(|message| {
            message.blocks.iter().any(|block| matches!(block, DisplayBlock::ToolResult { .. }))
        }));
    }
}
