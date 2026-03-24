//! Example: deterministic ratatui showcase without calling a real LLM API.
//!
//! Autoplays a handful of scripted prompts to demonstrate:
//! - transcript streaming
//! - thinking / redacted thinking
//! - successful tool use continuation
//! - tool failure continuation
//! - the dashboard and activity feed
//!
//! ```bash
//! cargo run --example tui_showcase
//! ```

use std::pin::Pin;
use std::time::Duration;

use futures::{Stream, stream};
use futures_util::StreamExt;
use kloud::Result;
use kloud::app::Session;
use kloud::error::LlmError;
use kloud::llm::client::{LlmClient, ModelInfo};
use kloud::llm::request::ChatRequest;
use kloud::llm::response::{ChatResponse, MessageDelta, StopReason, StreamEvent, Usage};
use kloud::llm::types::{ContentBlock, Role};
use kloud::tools::builtin::create_builtin_tools_registry;
use kloud::ui::{UiAction, UiBackend, create_ui_channels, tui::RatatuiBackend};

const SHOWCASE_PACE_MULTIPLIER: u64 = 1;

#[tokio::main]
async fn main() -> Result<()> {
	let workspace = std::env::current_dir().map_err(kloud::Error::Io)?;
	let tool_registry = create_builtin_tools_registry(&workspace);
	let tool_count = tool_registry.list_names().len();
	let read_path = demo_read_path(&workspace);

	let (ui_channels, ui_handle) = create_ui_channels();
	let action_tx = ui_channels.action_tx.clone();

	let session = Session::new(
		Box::new(DemoClient {
			read_path,
		}),
		tool_registry,
		"You are a deterministic showcase assistant.".to_string(),
		1_024,
		ui_handle,
	);

	let backend = RatatuiBackend {
		model: "demo-model".to_string(),
		max_context_tokens: 200_000,
		workspace: workspace.display().to_string(),
		branch: "demo/showcase".to_string(),
		effort: "showcase".to_string(),
		tool_count,
		instruction_files: vec!["AGENTS.md".to_string()],
		hook_count: 1,
	};

	let driver = tokio::spawn(async move {
		run_showcase(action_tx).await;
	});

	let (session_result, ui_result, _driver_result) =
		tokio::join!(session.run(), backend.run(ui_channels), driver);

	session_result?;
	ui_result?;

	Ok(())
}

struct DemoClient {
	read_path: String,
}

#[async_trait::async_trait]
impl LlmClient for DemoClient {
	async fn chat(&self, _request: ChatRequest) -> std::result::Result<ChatResponse, LlmError> {
		panic!("chat() is not used in tui_showcase");
	}

	async fn chat_stream(
		&self,
		request: ChatRequest,
	) -> std::result::Result<
		Pin<Box<dyn Stream<Item = std::result::Result<StreamEvent, LlmError>> + Send>>,
		LlmError,
	> {
		let steps = self.stream_plan(&request);
		let stream = stream::iter(steps).then(|(delay_ms, event)| async move {
			tokio::time::sleep(Duration::from_millis(delay_ms)).await;
			Ok(event)
		});
		Ok(Box::pin(stream))
	}

	fn model_info(&self) -> ModelInfo {
		ModelInfo {
			name: "demo-model".to_string(),
			max_context_length: 200_000,
			max_response_tokens: 4_096,
		}
	}
}

impl DemoClient {
	fn stream_plan(&self, request: &ChatRequest) -> Vec<(u64, StreamEvent)> {
		if let Some((tool_use_id, is_error)) = latest_tool_result(request) {
			return match tool_use_id.as_str() {
				"toolu_read_demo" => self.followup_text_stream(
					"Read completed. The transcript should now show a tool result block and the assistant continuing normally.",
					StopReason::EndTurn,
				),
				"toolu_missing_demo" => {
					if is_error {
						self.followup_text_stream(
							"Tool dispatch failed as expected. The UI should still continue and show the error result inline.",
							StopReason::EndTurn,
						)
					} else {
						self.followup_text_stream(
							"Unexpectedly succeeded.",
							StopReason::EndTurn,
						)
					}
				}
				_ => self.followup_text_stream("Unhandled tool result.", StopReason::EndTurn),
			};
		}

		match latest_user_text(request).as_deref() {
			Some(text) if text.contains("thinking demo") => self.thinking_text_stream(),
			Some(text) if text.contains("read demo") => self.tool_use_stream(
				"toolu_read_demo",
				"read",
				serde_json::json!({
					"path": self.read_path,
					"offset": 0,
					"limit": 12
				}),
			),
			Some(text) if text.contains("error demo") => {
				self.tool_use_stream("toolu_missing_demo", "missing_tool", serde_json::json!({}))
			}
			Some(text) if text.contains("redacted demo") => self.redacted_thinking_stream(),
			_ => self.followup_text_stream(
				"Showcase ready. Waiting for scripted prompts.",
				StopReason::EndTurn,
			),
		}
	}

	fn thinking_text_stream(&self) -> Vec<(u64, StreamEvent)> {
		vec![
			(
				pace(200),
				StreamEvent::ContentBlockStart {
					index: 0,
					content_block: ContentBlock::Thinking {
						thinking: String::new(),
						signature: None,
					},
				},
			),
			(
				pace(420),
				StreamEvent::ContentBlockDelta {
					index: 0,
					delta: kloud::llm::response::Delta::ThinkingDelta {
						thinking: "Inspecting the current transcript layout...".to_string(),
					},
				},
			),
			(
				pace(420),
				StreamEvent::ContentBlockDelta {
					index: 0,
					delta: kloud::llm::response::Delta::ThinkingDelta {
						thinking: "mapping transcript headers and activity state...".to_string(),
					},
				},
			),
			(
				pace(360),
				StreamEvent::ContentBlockDelta {
					index: 0,
					delta: kloud::llm::response::Delta::SignatureDelta {
						signature: "sig-showcase".to_string(),
					},
				},
			),
			(
				pace(180),
				StreamEvent::ContentBlockStop {
					index: 0,
				},
			),
			(
				pace(240),
				StreamEvent::ContentBlockStart {
					index: 1,
					content_block: ContentBlock::Text {
						text: String::new(),
						cache_control: None,
					},
				},
			),
			text_delta(1, pace(260), "I can stream "),
			text_delta(1, pace(260), "thinking, "),
			text_delta(1, pace(260), "loading motion, "),
			text_delta(1, pace(260), "and final text "),
			text_delta(1, pace(260), "deterministically."),
			(
				pace(180),
				StreamEvent::ContentBlockStop {
					index: 1,
				},
			),
			message_delta(pace(140), StopReason::EndTurn, 220, 96),
		]
	}

	fn tool_use_stream(
		&self,
		tool_use_id: &str,
		tool_name: &str,
		input: serde_json::Value,
	) -> Vec<(u64, StreamEvent)> {
		let input_json = serde_json::to_string(&input).expect("tool input should serialize");
		let chunks = split_json_into_chunks(&input_json, 3);

		vec![
			(
				pace(220),
				StreamEvent::ContentBlockStart {
					index: 0,
					content_block: ContentBlock::Thinking {
						thinking: String::new(),
						signature: None,
					},
				},
			),
			(
				pace(360),
				StreamEvent::ContentBlockDelta {
					index: 0,
					delta: kloud::llm::response::Delta::ThinkingDelta {
						thinking: format!("Preparing tool call `{tool_name}`..."),
					},
				},
			),
			(
				pace(360),
				StreamEvent::ContentBlockDelta {
					index: 0,
					delta: kloud::llm::response::Delta::ThinkingDelta {
						thinking: "serializing arguments and checking workspace boundaries..."
							.to_string(),
					},
				},
			),
			(
				pace(180),
				StreamEvent::ContentBlockStop {
					index: 0,
				},
			),
			(
				pace(260),
				StreamEvent::ContentBlockStart {
					index: 1,
					content_block: ContentBlock::ToolUse {
						id: tool_use_id.to_string(),
						name: tool_name.to_string(),
						input,
						cache_control: None,
					},
				},
			),
			input_json_delta(1, pace(320), &chunks[0]),
			input_json_delta(1, pace(320), &chunks[1]),
			input_json_delta(1, pace(320), &chunks[2]),
			(
				pace(180),
				StreamEvent::ContentBlockStop {
					index: 1,
				},
			),
			message_delta(pace(140), StopReason::ToolUse, 310, 88),
		]
	}

	fn redacted_thinking_stream(&self) -> Vec<(u64, StreamEvent)> {
		vec![
			(
				pace(220),
				StreamEvent::ContentBlockStart {
					index: 0,
					content_block: ContentBlock::RedactedThinking {
						data: "[filtered]".to_string(),
					},
				},
			),
			(
				pace(260),
				StreamEvent::ContentBlockStop {
					index: 0,
				},
			),
			(
				pace(220),
				StreamEvent::ContentBlockStart {
					index: 1,
					content_block: ContentBlock::Text {
						text: String::new(),
						cache_control: None,
					},
				},
			),
			text_delta(1, pace(260), "This turn includes "),
			text_delta(1, pace(260), "a redacted thinking block "),
			text_delta(1, pace(260), "to exercise that UI path."),
			(
				pace(180),
				StreamEvent::ContentBlockStop {
					index: 1,
				},
			),
			message_delta(pace(140), StopReason::EndTurn, 180, 84),
		]
	}

	fn followup_text_stream(&self, text: &str, stop_reason: StopReason) -> Vec<(u64, StreamEvent)> {
		let parts = split_text(text, 18);
		let mut steps = vec![(
			pace(240),
			StreamEvent::ContentBlockStart {
				index: 0,
				content_block: ContentBlock::Text {
					text: String::new(),
					cache_control: None,
				},
			},
		)];

		for part in parts {
			steps.push(text_delta(0, pace(260), &part));
		}

		steps.push((
			pace(180),
			StreamEvent::ContentBlockStop {
				index: 0,
			},
		));
		steps.push(message_delta(pace(140), stop_reason, 420, 128));
		steps
	}
}

fn latest_user_text(request: &ChatRequest) -> Option<String> {
	request.messages.iter().rev().find_map(|message| {
		if !matches!(message.role, Role::User) {
			return None;
		}
		message.content.iter().find_map(|block| match block {
			ContentBlock::Text {
				text,
				..
			} => Some(text.clone()),
			_ => None,
		})
	})
}

fn latest_tool_result(request: &ChatRequest) -> Option<(String, bool)> {
	request.messages.iter().rev().find_map(|message| {
		if !matches!(message.role, Role::User) {
			return None;
		}
		message.content.iter().find_map(|block| match block {
			ContentBlock::ToolResult {
				tool_use_id,
				is_error,
				..
			} => Some((tool_use_id.clone(), is_error.unwrap_or(false))),
			_ => None,
		})
	})
}

fn text_delta(index: u32, delay_ms: u64, text: &str) -> (u64, StreamEvent) {
	(
		delay_ms,
		StreamEvent::ContentBlockDelta {
			index,
			delta: kloud::llm::response::Delta::TextDelta {
				text: text.to_string(),
			},
		},
	)
}

fn input_json_delta(index: u32, delay_ms: u64, partial_json: &str) -> (u64, StreamEvent) {
	(
		delay_ms,
		StreamEvent::ContentBlockDelta {
			index,
			delta: kloud::llm::response::Delta::InputJsonDelta {
				partial_json: partial_json.to_string(),
			},
		},
	)
}

fn message_delta(
	delay_ms: u64,
	stop_reason: StopReason,
	input_tokens: u32,
	output_tokens: u32,
) -> (u64, StreamEvent) {
	(
		delay_ms,
		StreamEvent::MessageDelta {
			delta: MessageDelta {
				stop_reason: Some(stop_reason),
				stop_sequence: None,
			},
			usage: Usage {
				input_tokens,
				output_tokens,
				cache_creation_input_tokens: None,
				cache_read_input_tokens: None,
				cache_creation: None,
				server_tool_use: None,
				service_tier: None,
			},
		},
	)
}

fn split_text(text: &str, chunk_len: usize) -> Vec<String> {
	let mut out = Vec::new();
	let mut current = String::new();

	for word in text.split_whitespace() {
		let candidate_len = if current.is_empty() {
			word.len()
		} else {
			current.len() + 1 + word.len()
		};

		if candidate_len > chunk_len && !current.is_empty() {
			out.push(current);
			current = word.to_string();
		} else if current.is_empty() {
			current = word.to_string();
		} else {
			current.push(' ');
			current.push_str(word);
		}
	}

	if !current.is_empty() {
		out.push(current);
	}

	out
}

fn split_json_into_chunks(input: &str, chunks: usize) -> Vec<String> {
	let mut out = Vec::new();
	let len = input.len();
	let chunks = chunks.max(1);

	for i in 0..chunks {
		let start = i * len / chunks;
		let end = (i + 1) * len / chunks;
		out.push(input[start..end].to_string());
	}

	out
}

fn demo_read_path(workspace: &std::path::Path) -> String {
	for candidate in ["TODO.md", "README.md", "Cargo.toml"] {
		if workspace.join(candidate).is_file() {
			return candidate.to_string();
		}
	}
	"Cargo.toml".to_string()
}

async fn run_showcase(action_tx: tokio::sync::mpsc::Sender<UiAction>) {
	let script = [
		(pace(900), "show thinking demo"),
		(pace(7_500), "show read demo"),
		(pace(8_500), "show error demo"),
		(pace(8_500), "show redacted demo"),
	];

	for (delay_ms, prompt) in script {
		tokio::time::sleep(Duration::from_millis(delay_ms)).await;
		let _ = action_tx.send(UiAction::SendMessage(prompt.to_string())).await;
	}

	tokio::time::sleep(Duration::from_millis(pace(8_000))).await;
	let _ = action_tx.send(UiAction::Exit).await;
}

fn pace(ms: u64) -> u64 {
	ms * SHOWCASE_PACE_MULTIPLIER
}
