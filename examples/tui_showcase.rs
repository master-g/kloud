//! Example: deterministic ratatui showcase without calling a real LLM API.
//!
//! Autoplays scripted prompts to demonstrate:
//! - transcript streaming with activity line animations
//! - thinking / redacted thinking
//! - successful tool use continuation
//! - tool failure continuation
//! - markdown rendering with code blocks, emphasis, bold, lists, blockquotes
//! - shimmer animation (fast 50ms / slow 200ms) and thinking breathing effect
//! - stall detection (4s silence → red color transition)
//! - token counting animation (smooth counter + status bar arrows)
//! - thinking duration display (`thinking` → `thought for Xs`)
//! - the dashboard and activity feed
//!
//! ```bash
//! cargo run --example tui_showcase
//! ```

use std::pin::Pin;
use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use futures::{Stream, stream};
use futures_util::StreamExt;
use kloud::Result;
use kloud::agent::view::Screen;
use kloud::app::Session;
use kloud::error::LlmError;
use kloud::llm::client::{LlmClient, ModelInfo};
use kloud::llm::request::ChatRequest;
use kloud::llm::response::{ChatResponse, MessageDelta, StopReason, StreamEvent, Usage};
use kloud::llm::types::{ContentBlock, Role};
use kloud::tools::builtin::create_builtin_tools_registry;
use kloud::ui::tui::theme::{ColorScheme, Theme};
use kloud::ui::{UiAction, UiBackend, create_ui_channels, tui::RatatuiBackend};

const SHOWCASE_PACE_MULTIPLIER: u64 = 1;

#[tokio::main]
async fn main() -> Result<()> {
    let workspace = std::env::current_dir().map_err(kloud::Error::Io)?;
    let tool_registry = create_builtin_tools_registry(&workspace);
    let tool_count = tool_registry.list_names().len();
    let read_path = demo_read_path(&workspace);

    let (mut ui_channels, ui_handle) = create_ui_channels();
    let action_tx = ui_channels.action_tx.clone();

    // Create key injection channel for demo
    let (key_inject_tx, key_inject_rx) = tokio::sync::mpsc::channel::<CrosstermEvent>(64);
    ui_channels.key_inject_rx = Some(key_inject_rx);

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
        theme: Theme::from_scheme(ColorScheme::Dark, false),
        show_title_bar: true,
    };

    let driver = tokio::spawn(async move {
        run_showcase(action_tx, key_inject_tx).await;
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
				// Diff tool result
				"toolu_diff_demo" => self.followup_text_stream(
					"Diff tool result rendered. The output should show colored diff lines with add/remove markers.",
					StopReason::EndTurn,
				),
				// Spinner tool results
				id if id.starts_with("toolu_spinner_") => {
					self.followup_text_stream(
						"Tool spinners displayed. Multiple active tools should have shown compact spinners.",
						StopReason::EndTurn,
					)
				}
				// Batch echo tool results
					id if id.starts_with("toolu_batch_") => {
						self.followup_text_stream(
							"Batch tool execution complete. The transcript should show the batch header with all 3 tool results.",
							StopReason::EndTurn,
						)
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
			Some(text) if text.contains("markdown demo") => self.markdown_text_stream(),
			Some(text) if text.contains("shimmer demo") => self.shimmer_text_stream(),
			Some(text) if text.contains("stall demo") => self.stall_stream(),
			Some(text) if text.contains("token demo") => self.token_counting_stream(),
			Some(text) if text.contains("markdown enhanced") => self.markdown_enhanced_stream(),
			Some(text) if text.contains("thinking duration") => self.thinking_duration_stream(),
			// New demos
			Some(text) if text.contains("batch tools demo") => self.batch_tools_demo(),
			Some(text) if text.contains("mcp demo") => self.mcp_tool_demo(),
			Some(text) if text.contains("max tokens demo") => self.max_tokens_demo(),
			Some(text) if text.contains("refusal demo") => self.refusal_demo(),
			Some(text) if text.contains("long text demo") => self.long_text_demo(),
			Some(text) if text.contains("context demo") => self.context_full_demo(),
			// Widget demos
			Some(text) if text.contains("tool spinners demo") => self.tool_spinners_demo(),
			Some(text) if text.contains("diff tool demo") => self.diff_tool_result_demo(),
			// Extended demos
			Some(text) if text.contains("diff demo") => self.diff_demo(),
			Some(text) if text.contains("pause turn demo") => self.pause_turn_demo(),
			Some(text) if text.contains("stop sequence demo") => self.stop_sequence_demo(),
			_ => self.followup_text_stream(
				"Showcase ready. Available demos: thinking, read, error, redacted, markdown, shimmer, stall, token, markdown enhanced, thinking duration, batch tools, mcp, max tokens, refusal, long text, context, tool spinners, diff tool",
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

    /// Demonstrates markdown rendering with code blocks and language labels.
    fn markdown_text_stream(&self) -> Vec<(u64, StreamEvent)> {
        vec![
            (
                pace(240),
                StreamEvent::ContentBlockStart {
                    index: 0,
                    content_block: ContentBlock::Text {
                        text: String::new(),
                        cache_control: None,
                    },
                },
            ),
            // Plain text line
            text_delta(0, pace(300), "Here's how the markdown renderer handles code blocks:"),
            // Paragraph break
            (
                pace(200),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::TextDelta {
                        text: "\n\n".to_string(),
                    },
                },
            ),
            // Fenced code block with language label
            text_delta(
                0,
                pace(200),
                "```rust\nfn greet(name: &str) -> String {\n    format!(\"Hello, {name}!\")\n}\n```",
            ),
            // Another paragraph
            (
                pace(200),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::TextDelta {
                        text: "\n\n".to_string(),
                    },
                },
            ),
            // Indented code block
            text_delta(0, pace(200), "    // Indented code block\n    let x = 42;"),
            // Paragraph break
            (
                pace(200),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::TextDelta {
                        text: "\n\n".to_string(),
                    },
                },
            ),
            // Another fenced block with different language
            text_delta(0, pace(200), "```python\ndef main():\n    print('hello world')\n```"),
            // Paragraph break
            (
                pace(200),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::TextDelta {
                        text: "\n\n".to_string(),
                    },
                },
            ),
            // Plain text following code blocks
            text_delta(
                0,
                pace(300),
                "The ⎿ prefix marks assistant messages, and code blocks show their language label.",
            ),
            (
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: 0,
                },
            ),
            message_delta(pace(140), StopReason::EndTurn, 320, 196),
        ]
    }

    /// Demonstrates the shimmer animation with a long activity verb.
    /// Uses thinking mode to trigger slow shimmer (200ms), then switches to
    /// responding mode for fast shimmer (50ms). Includes a `UsageReport` to
    /// populate token counting.
    fn shimmer_text_stream(&self) -> Vec<(u64, StreamEvent)> {
        vec![
            (
                pace(240),
                StreamEvent::ContentBlockStart {
                    index: 0,
                    content_block: ContentBlock::Thinking {
                        thinking: String::new(),
                        signature: None,
                    },
                },
            ),
            // Stream thinking for 4+ seconds to exercise slow shimmer (200ms, right-to-left)
            // and to approach the 3-second threshold for thinking shimmer breathing
            (
                pace(500),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: "This demonstrates the activity line shimmer animation..."
                            .to_string(),
                    },
                },
            ),
            (
                pace(600),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: " watch the verb — it glows hot near the shimmer index."
                            .to_string(),
                    },
                },
            ),
            (
                pace(600),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: " The thinking shimmer breathes after 3 seconds.".to_string(),
                    },
                },
            ),
            (
                pace(700),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: " Requesting mode uses fast shimmer (50ms, left-to-right)."
                            .to_string(),
                    },
                },
            ),
            (
                pace(700),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: " ToolUse mode shows ↓ with the tool name.".to_string(),
                    },
                },
            ),
            (
                pace(700),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: " Token counting starts after 30 seconds of activity."
                            .to_string(),
                    },
                },
            ),
            (
                pace(300),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::SignatureDelta {
                        signature: "sig-shimmer".to_string(),
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
            // Respond with text that exercises fast shimmer (requesting -> responding transition)
            text_delta(1, pace(260), "The "),
            text_delta(1, pace(260), "activity "),
            text_delta(1, pace(260), "line "),
            text_delta(1, pace(260), "above "),
            text_delta(1, pace(260), "showcases "),
            text_delta(1, pace(260), "shimmer, "),
            text_delta(1, pace(260), "breathing, "),
            text_delta(1, pace(260), "and "),
            text_delta(1, pace(260), "token "),
            text_delta(1, pace(260), "counting "),
            text_delta(1, pace(260), "animations."),
            (
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: 1,
                },
            ),
            // Report tokens — the counter will animate smoothly toward this value
            message_delta(pace(100), StopReason::EndTurn, 420, 1842),
        ]
    }

    /// Demonstrates stall detection: start thinking, then go silent for 4+ seconds.
    /// The activity line should transition from normal color to red as stalled
    /// intensity increases via EMA smoothing.
    fn stall_stream(&self) -> Vec<(u64, StreamEvent)> {
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
                pace(400),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: "Starting analysis...".to_string(),
                    },
                },
            ),
            // 4.5 second gap — triggers stall detection (threshold is 3s)
            (
                pace(4500),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: " ...recovered after a long pause.".to_string(),
                    },
                },
            ),
            (
                pace(300),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::SignatureDelta {
                        signature: "sig-stall".to_string(),
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
            text_delta(1, pace(260), "The activity line should have turned "),
            text_delta(1, pace(260), "red during the 4.5s gap "),
            text_delta(1, pace(260), "and smoothly recovered once tokens resumed."),
            (
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: 1,
                },
            ),
            message_delta(pace(140), StopReason::EndTurn, 350, 120),
        ]
    }

    /// Demonstrates token counting animation with large token jumps.
    /// The `MessageDelta` reports a high `output_tokens` value, and the smooth
    /// counter in the status bar animates toward it over multiple ticks.
    fn token_counting_stream(&self) -> Vec<(u64, StreamEvent)> {
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
                pace(600),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: "Generating a large response to show token counting..."
                            .to_string(),
                    },
                },
            ),
            (
                pace(300),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::SignatureDelta {
                        signature: "sig-tokens".to_string(),
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
            text_delta(1, pace(260), "Watch the status bar tokens — "),
            text_delta(1, pace(260), "the output counter will animate "),
            text_delta(1, pace(260), "smoothly from its current value "),
            text_delta(1, pace(260), "toward the large target (8192). "),
            text_delta(1, pace(260), "The input_tokens (12800↓) and "),
            text_delta(1, pace(260), "output_tokens (8192↑) also update "),
            text_delta(1, pace(260), "the context meter percentage."),
            (
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: 1,
                },
            ),
            // Large token jump to exercise the smooth counter animation
            message_delta(pace(140), StopReason::EndTurn, 12_800, 8_192),
        ]
    }

    /// Demonstrates enhanced markdown rendering: emphasis, strong, lists,
    /// blockquotes, horizontal rules, and inline code.
    fn markdown_enhanced_stream(&self) -> Vec<(u64, StreamEvent)> {
        let md = "\
Here's a **bold statement** and some *italic emphasis* plus `inline code`.

## Key Features

- **Streaming**: real-time token delivery
- *Shimmer*: animated activity line
- `tool_use`: structured tool invocations

> This is a blockquote that demonstrates
> how multi-line quotes are rendered.

---

### Ordered Steps

1. Start the showcase with `cargo run --example tui_showcase`
2. Watch each demo play automatically
3. Observe the activity line animations

Here's a mixed example with **bold _and italic_** together, plus a [link](https://github.com).

---

Final paragraph after the horizontal rule.";

        let chunks = split_text(md, 40);
        let mut steps: Vec<(u64, StreamEvent)> = vec![(
            pace(240),
            StreamEvent::ContentBlockStart {
                index: 0,
                content_block: ContentBlock::Text {
                    text: String::new(),
                    cache_control: None,
                },
            },
        )];

        for chunk in chunks {
            steps.push(text_delta(0, pace(200), &chunk));
        }

        steps.push((
            pace(180),
            StreamEvent::ContentBlockStop {
                index: 0,
            },
        ));
        steps.push(message_delta(pace(140), StopReason::EndTurn, 480, 260));
        steps
    }

    /// Demonstrates thinking duration display: 5+ seconds of thinking, then
    /// transition to text. The UI should show "thinking" → "thought for Xs".
    fn thinking_duration_stream(&self) -> Vec<(u64, StreamEvent)> {
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
                pace(800),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: "Deep analysis in progress...".to_string(),
                    },
                },
            ),
            (
                pace(1200),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: " evaluating multiple approaches...".to_string(),
                    },
                },
            ),
            (
                pace(1200),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: " weighing trade-offs carefully...".to_string(),
                    },
                },
            ),
            (
                pace(1200),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: " synthesizing final conclusion...".to_string(),
                    },
                },
            ),
            (
                pace(1200),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: " confirming reasoning chain is sound.".to_string(),
                    },
                },
            ),
            (
                pace(300),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::SignatureDelta {
                        signature: "sig-duration".to_string(),
                    },
                },
            ),
            (
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: 0,
                },
            ),
            // Now switch to text — the UI should transition to "thought for ~6s"
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
            text_delta(1, pace(260), "After ~6 seconds of thinking, "),
            text_delta(1, pace(260), "the activity line should now show "),
            text_delta(1, pace(260), "\"thought for 6s\" "),
            text_delta(1, pace(260), "before fading away."),
            (
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: 1,
                },
            ),
            message_delta(pace(140), StopReason::EndTurn, 520, 340),
        ]
    }

    /// 3 consecutive ToolUse blocks targeting "echo" with StopReason::ToolUse.
    fn batch_tools_demo(&self) -> Vec<(u64, StreamEvent)> {
        let input = serde_json::json!({"message": "hello"});
        let input_json = serde_json::to_string(&input).expect("serialize");
        let chunks = split_json_into_chunks(&input_json, 3);
        let mut steps = vec![(
            pace(200),
            StreamEvent::ContentBlockStart {
                index: 0,
                content_block: ContentBlock::Text {
                    text: String::new(),
                    cache_control: None,
                },
            },
        )];
        steps.push(text_delta(0, pace(260), "I need to call echo three times in batch."));
        steps.push((
            pace(180),
            StreamEvent::ContentBlockStop {
                index: 0,
            },
        ));
        for i in 0..3u32 {
            let id = format!("toolu_batch_{i}");
            steps.push((
                pace(260),
                StreamEvent::ContentBlockStart {
                    index: i + 1,
                    content_block: ContentBlock::ToolUse {
                        id: id.clone(),
                        name: "echo".to_string(),
                        input: input.clone(),
                        cache_control: None,
                    },
                },
            ));
            for chunk in &chunks {
                steps.push(input_json_delta(i + 1, pace(320), chunk));
            }
            steps.push((
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: i + 1,
                },
            ));
        }
        steps.push(message_delta(pace(140), StopReason::ToolUse, 420, 256));
        steps
    }

    /// Single ToolUse block with server_name populated.
    fn mcp_tool_demo(&self) -> Vec<(u64, StreamEvent)> {
        let input = serde_json::json!({"query": "list_repos"});
        let input_json = serde_json::to_string(&input).expect("serialize");
        let chunks = split_json_into_chunks(&input_json, 3);
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
                pace(360),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::ThinkingDelta {
                        thinking: "Using MCP server to query GitHub...".to_string(),
                    },
                },
            ),
            (
                pace(300),
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: kloud::llm::response::Delta::SignatureDelta {
                        signature: "sig-mcp".to_string(),
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
                        id: "toolu_mcp_demo".to_string(),
                        name: "github_query".to_string(),
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
            message_delta(pace(140), StopReason::ToolUse, 380, 128),
        ]
    }

    /// Text stream ending with StopReason::MaxTokens.
    fn max_tokens_demo(&self) -> Vec<(u64, StreamEvent)> {
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
        for chunk in [
            "The response ",
            "was cut short ",
            "because the maximum ",
            "token limit ",
            "was reached.",
        ] {
            steps.push(text_delta(0, pace(260), chunk));
        }
        steps.push((
            pace(180),
            StreamEvent::ContentBlockStop {
                index: 0,
            },
        ));
        steps.push(message_delta(pace(140), StopReason::MaxTokens, 190_000, 4_096));
        steps
    }

    /// Text stream ending with StopReason::Refusal.
    fn refusal_demo(&self) -> Vec<(u64, StreamEvent)> {
        vec![
            (
                pace(240),
                StreamEvent::ContentBlockStart {
                    index: 0,
                    content_block: ContentBlock::Text {
                        text: String::new(),
                        cache_control: None,
                    },
                },
            ),
            text_delta(0, pace(260), "I apologize, "),
            text_delta(0, pace(260), "but I must decline "),
            text_delta(0, pace(260), "this request."),
            (
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: 0,
                },
            ),
            message_delta(pace(140), StopReason::Refusal, 120, 48),
        ]
    }

    /// Large repeated output to stress virtual scroll.
    fn long_text_demo(&self) -> Vec<(u64, StreamEvent)> {
        let paragraph = "The quick brown fox jumps over the lazy dog. This paragraph is repeated many times to generate a very long output stream that exercises the virtual scroll rendering path and tests how the TUI handles large amounts of text content efficiently.

";
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
        // 120 paragraphs
        for i in 0..120 {
            let chunk = if i % 10 == 0 {
                format!(
                    "
--- Section {} ---

{}",
                    i / 10 + 1,
                    paragraph
                )
            } else {
                paragraph.to_string()
            };
            steps.push(text_delta(0, pace(20), &chunk));
        }
        steps.push((
            pace(180),
            StreamEvent::ContentBlockStop {
                index: 0,
            },
        ));
        steps.push(message_delta(pace(140), StopReason::EndTurn, 8_000, 12_000));
        steps
    }

    /// Stream with high input_tokens to trigger red context meter.
    fn context_full_demo(&self) -> Vec<(u64, StreamEvent)> {
        vec![
            (
                pace(240),
                StreamEvent::ContentBlockStart {
                    index: 0,
                    content_block: ContentBlock::Text {
                        text: String::new(),
                        cache_control: None,
                    },
                },
            ),
            text_delta(0, pace(260), "The context meter "),
            text_delta(0, pace(260), "should now be "),
            text_delta(0, pace(260), "nearly full (red). "),
            text_delta(0, pace(260), "This exercises the "),
            text_delta(0, pace(260), "warning style rendering."),
            (
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: 0,
                },
            ),
            message_delta(pace(140), StopReason::EndTurn, 190_000, 256),
        ]
    }

    /// Demonstrates diff-colored output via markdown code block.
    fn diff_demo(&self) -> Vec<(u64, StreamEvent)> {
        let diff = "Here is the proposed change:\n\n```diff\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -10,6 +10,8 @@\n fn main() {\n     let greeting = \"hello\";\n+    let name = \"world\";\n+    let message = format!(\"{greeting}, {name}!\");\n-    println!(\"{greeting}\");\n+    println!(\"{message}\");\n }\n```\n\nThe diff shows adding a name variable and updating the print statement.";

        let chunks = split_text(diff, 35);
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
        for chunk in chunks {
            steps.push(text_delta(0, pace(200), &chunk));
        }
        steps.push((
            pace(180),
            StreamEvent::ContentBlockStop {
                index: 0,
            },
        ));
        steps.push(message_delta(pace(140), StopReason::EndTurn, 320, 180));
        steps
    }

    /// Text stream ending with StopReason::PauseTurn.
    fn pause_turn_demo(&self) -> Vec<(u64, StreamEvent)> {
        vec![
            (
                pace(240),
                StreamEvent::ContentBlockStart {
                    index: 0,
                    content_block: ContentBlock::Text {
                        text: String::new(),
                        cache_control: None,
                    },
                },
            ),
            text_delta(0, pace(260), "This response "),
            text_delta(0, pace(260), "was paused "),
            text_delta(0, pace(260), "mid-turn by the model. "),
            text_delta(0, pace(260), "The status bar should show "),
            text_delta(0, pace(260), "\"stop: pause_turn\"."),
            (
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: 0,
                },
            ),
            message_delta(pace(140), StopReason::PauseTurn, 280, 96),
        ]
    }

    /// Text stream ending with StopReason::StopSequence.
    fn stop_sequence_demo(&self) -> Vec<(u64, StreamEvent)> {
        vec![
            (
                pace(240),
                StreamEvent::ContentBlockStart {
                    index: 0,
                    content_block: ContentBlock::Text {
                        text: String::new(),
                        cache_control: None,
                    },
                },
            ),
            text_delta(0, pace(260), "This response "),
            text_delta(0, pace(260), "hit a custom "),
            text_delta(0, pace(260), "stop sequence. "),
            text_delta(0, pace(260), "The status bar should show "),
            text_delta(0, pace(260), "\"stop: stop_sequence\"."),
            (
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: 0,
                },
            ),
            message_delta(pace(140), StopReason::StopSequence, 220, 84),
        ]
    }

    /// 3 ToolUse blocks (bash, read, edit) to exercise render_tool_spinners().
    fn tool_spinners_demo(&self) -> Vec<(u64, StreamEvent)> {
        let tools: Vec<(&str, serde_json::Value)> = vec![
            ("bash", serde_json::json!({"command": "cargo test"})),
            ("read", serde_json::json!({"path": self.read_path})),
            (
                "edit",
                serde_json::json!({"file": "src/main.rs", "old": "fn main()", "new": "pub fn main()"}),
            ),
        ];
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
        steps.push(text_delta(0, pace(260), "Running multiple tools in parallel..."));
        steps.push((
            pace(180),
            StreamEvent::ContentBlockStop {
                index: 0,
            },
        ));

        for (i, (name, input)) in tools.iter().enumerate() {
            let idx = (i + 1) as u32;
            let id = format!("toolu_spinner_{i}");
            let input_json = serde_json::to_string(input).expect("serialize");
            let chunks = split_json_into_chunks(&input_json, 2);
            steps.push((
                pace(260),
                StreamEvent::ContentBlockStart {
                    index: idx,
                    content_block: ContentBlock::ToolUse {
                        id: id.clone(),
                        name: name.to_string(),
                        input: input.clone(),
                        cache_control: None,
                    },
                },
            ));
            for chunk in &chunks {
                steps.push(input_json_delta(idx, pace(200), chunk));
            }
            steps.push((
                pace(180),
                StreamEvent::ContentBlockStop {
                    index: idx,
                },
            ));
        }
        steps.push(message_delta(pace(140), StopReason::ToolUse, 480, 320));
        steps
    }

    /// ToolUse block targeting "echo" with diff content as message.
    fn diff_tool_result_demo(&self) -> Vec<(u64, StreamEvent)> {
        let input = serde_json::json!({
            "message": "--- a/src/main.rs\n+++ b/src/main.rs\n@@ -10,6 +10,8 @@\n fn main() {\n     let greeting = \"hello\";\n+    let name = \"world\";\n+    let message = format!(\"{greeting}, {name}!\");\n-    println!(\"{greeting}\");\n+    println!(\"{message}\");\n }\n"
        });
        let input_json = serde_json::to_string(&input).expect("serialize");
        let chunks = split_json_into_chunks(&input_json, 3);
        vec![
            (
                pace(240),
                StreamEvent::ContentBlockStart {
                    index: 0,
                    content_block: ContentBlock::Text {
                        text: String::new(),
                        cache_control: None,
                    },
                },
            ),
            text_delta(0, pace(260), "Applying a patch with diff output..."),
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
                        id: "toolu_diff_demo".to_string(),
                        name: "echo".to_string(),
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
            message_delta(pace(140), StopReason::ToolUse, 380, 180),
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
    let last_user_msg = request.messages.iter().rev().find(|m| matches!(m.role, Role::User))?;
    last_user_msg.content.iter().find_map(|block| match block {
        ContentBlock::ToolResult {
            tool_use_id,
            is_error,
            ..
        } => Some((tool_use_id.clone(), is_error.unwrap_or(false))),
        _ => None,
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

    // Preserve original whitespace by splitting into whitespace-inclusive tokens.
    for token in text.split_inclusive(char::is_whitespace) {
        if current.len() + token.len() > chunk_len && !current.is_empty() {
            out.push(current);
            current = String::new();
        }
        current.push_str(token);
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

async fn run_showcase(
    action_tx: tokio::sync::mpsc::Sender<UiAction>,
    key_inject_tx: tokio::sync::mpsc::Sender<CrosstermEvent>,
) {
    // --- Streaming demos ---
    let stream_demos = [
        (pace(900), "show thinking demo"),
        (pace(7_500), "show read demo"),
        (pace(8_500), "show error demo"),
        (pace(8_500), "show redacted demo"),
        (pace(8_500), "show markdown demo"),
        (pace(6_000), "show shimmer demo"),
        (pace(8_000), "show stall demo"),
        (pace(10_000), "show token demo"),
        (pace(5_000), "show markdown enhanced"),
        (pace(8_000), "show thinking duration"),
        (pace(10_000), "show batch tools demo"),
        (pace(6_000), "show mcp demo"),
        (pace(6_000), "show max tokens demo"),
        (pace(5_000), "show refusal demo"),
        (pace(6_000), "show long text demo"),
        (pace(6_000), "show context demo"),
        // Extended demos
        (pace(6_000), "show diff demo"),
        (pace(5_000), "show pause turn demo"),
        (pace(5_000), "show stop sequence demo"),
        // Widget demos
        (pace(8_000), "show tool spinners demo"),
        (pace(8_000), "show diff tool demo"),
    ];

    for (delay_ms, prompt) in stream_demos {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        let _ = action_tx.send(UiAction::SendMessage(prompt.to_string())).await;
    }

    // --- Slash command phases ---
    tokio::time::sleep(Duration::from_millis(pace(5_000))).await;
    let _ = action_tx
        .send(UiAction::SlashCommand {
            command: "help".to_string(),
            args: String::new(),
        })
        .await;

    tokio::time::sleep(Duration::from_millis(pace(2_500))).await;
    let _ = action_tx
        .send(UiAction::SlashCommand {
            command: "unknown_cmd".to_string(),
            args: String::new(),
        })
        .await;

    tokio::time::sleep(Duration::from_millis(pace(2_500))).await;
    let _ = action_tx
        .send(UiAction::SlashCommand {
            command: "theme".to_string(),
            args: "light".to_string(),
        })
        .await;
    tokio::time::sleep(Duration::from_millis(pace(2_000))).await;
    let _ = action_tx
        .send(UiAction::SlashCommand {
            command: "theme".to_string(),
            args: "dark".to_string(),
        })
        .await;

    // --- Cancel flow ---
    tokio::time::sleep(Duration::from_millis(pace(2_000))).await;
    let _ = action_tx.send(UiAction::SendMessage("show cancel demo".to_string())).await;
    tokio::time::sleep(Duration::from_millis(1_500)).await;
    let _ = action_tx.send(UiAction::CancelTurn).await;

    // --- Transcript mode ---
    tokio::time::sleep(Duration::from_millis(pace(2_000))).await;
    let _ = action_tx.send(UiAction::SetScreen(Screen::Transcript)).await;
    tokio::time::sleep(Duration::from_millis(pace(3_000))).await;
    let _ = action_tx.send(UiAction::SetScreen(Screen::Prompt)).await;

    // --- Key injection: search demo ---
    tokio::time::sleep(Duration::from_millis(pace(2_000))).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)))
        .await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    for c in "think".chars() {
        let _ = key_inject_tx
            .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)))
            .await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)))
        .await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)))
        .await;
    tokio::time::sleep(Duration::from_millis(800)).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)))
        .await;

    // --- Key injection: multi-line input ---
    tokio::time::sleep(Duration::from_millis(pace(1_500))).await;
    for c in "line one".chars() {
        let _ = key_inject_tx
            .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)))
            .await;
        tokio::time::sleep(Duration::from_millis(60)).await;
    }
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT)))
        .await;
    for c in "line two".chars() {
        let _ = key_inject_tx
            .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)))
            .await;
        tokio::time::sleep(Duration::from_millis(60)).await;
    }
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)))
        .await;

    // Clear text_area: 30 backspaces to remove leftover "line one\nline two\n"
    for _ in 0..30 {
        let _ = key_inject_tx
            .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)))
            .await;
        tokio::time::sleep(Duration::from_millis(30)).await;
    }

    // --- Key injection: Tab collapse ---
    tokio::time::sleep(Duration::from_millis(pace(2_000))).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)))
        .await;
    tokio::time::sleep(Duration::from_millis(pace(1_500))).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)))
        .await;

    // --- Key injection: autocomplete demo ---
    tokio::time::sleep(Duration::from_millis(pace(1_500))).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)))
        .await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    for c in "he".chars() {
        let _ = key_inject_tx
            .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)))
            .await;
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    tokio::time::sleep(Duration::from_millis(800)).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)))
        .await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)))
        .await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    // Dismiss autocomplete with Escape
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)))
        .await;
    // Clear input
    tokio::time::sleep(Duration::from_millis(300)).await;
    for _ in 0..5 {
        let _ = key_inject_tx
            .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)))
            .await;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // --- Permission prompt demo (accept) ---
    tokio::time::sleep(Duration::from_millis(pace(2_000))).await;
    let _ = action_tx
        .send(UiAction::SlashCommand {
            command: "permission".to_string(),
            args: String::new(),
        })
        .await;
    tokio::time::sleep(Duration::from_millis(pace(1_500))).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE)))
        .await;

    // --- Permission prompt demo (deny) ---
    tokio::time::sleep(Duration::from_millis(pace(1_500))).await;
    let _ = action_tx
        .send(UiAction::SlashCommand {
            command: "permission".to_string(),
            args: String::new(),
        })
        .await;
    tokio::time::sleep(Duration::from_millis(pace(1_500))).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)))
        .await;

    // --- Notification toast demo ---
    tokio::time::sleep(Duration::from_millis(pace(2_000))).await;
    let _ = action_tx
        .send(UiAction::SlashCommand {
            command: "notify".to_string(),
            args: "Hello from kloud!".to_string(),
        })
        .await;

    // --- Search mode full cycle ---
    tokio::time::sleep(Duration::from_millis(pace(3_000))).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)))
        .await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    for c in "thinking".chars() {
        let _ = key_inject_tx
            .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)))
            .await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)))
        .await;
    tokio::time::sleep(Duration::from_millis(pace(1_000))).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)))
        .await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Char('N'), KeyModifiers::SHIFT)))
        .await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let _ = key_inject_tx
        .send(CrosstermEvent::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)))
        .await;

    tokio::time::sleep(Duration::from_millis(pace(4_000))).await;
    let _ = action_tx.send(UiAction::Exit).await;
}

fn pace(ms: u64) -> u64 {
    ms * SHOWCASE_PACE_MULTIPLIER
}
