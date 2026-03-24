//! Stdio-based UI backend (println + stdin).
//!
//! A minimal line-oriented UI for non-interactive / fallback use.
//! Reads lines from stdin on a blocking task, prints streaming text
//! to stdout as it arrives.

use tokio::io::AsyncBufReadExt;

use super::backend::{UiBackend, UiChannels};
use super::events::{AppEvent, UiAction};

/// A simple line-oriented UI that uses stdout/stdin directly.
pub struct StdioBackend;

#[async_trait::async_trait]
impl UiBackend for StdioBackend {
	async fn run(self, mut channels: UiChannels) -> crate::Result<()> {
		let action_tx = channels.action_tx.clone();

		// Spawn a task to read stdin lines
		let input_handle = tokio::spawn(async move {
			let stdin = tokio::io::stdin();
			let reader = tokio::io::BufReader::new(stdin);
			let mut lines = reader.lines();

			loop {
				eprint!("> ");
				let Ok(Some(line)) = lines.next_line().await else {
					break;
				};

				let trimmed = line.trim().to_string();
				if trimmed.is_empty() {
					continue;
				}

				let action = if let Some(rest) = trimmed.strip_prefix('/') {
					let mut parts = rest.splitn(2, ' ');
					let command = parts.next().unwrap_or("").to_string();
					let args = parts.next().unwrap_or("").to_string();
					UiAction::SlashCommand {
						command,
						args,
					}
				} else {
					UiAction::SendMessage(trimmed)
				};

				if action_tx.send(action).await.is_err() {
					break;
				}
			}
		});

		// Event display loop
		while let Some(event) = channels.event_rx.recv().await {
			match event {
				AppEvent::AssistantTurnStart => {
					eprintln!();
				}
				AppEvent::TextDelta(text) => {
					eprint!("{text}");
				}
				AppEvent::ThinkingDelta(text) => {
					eprint!("[thinking] {text}");
				}
				AppEvent::RedactedThinking(text) => {
					if text.is_empty() {
						eprint!("[thinking redacted]");
					} else {
						eprint!("[thinking redacted] {text}");
					}
				}
				AppEvent::AssistantTurnEnd {
					..
				} => {
					eprintln!("\n");
				}
				AppEvent::Error(msg) => {
					eprintln!("[error] {msg}");
				}
				AppEvent::UsageReport {
					input_tokens,
					output_tokens,
				} => {
					eprintln!("[tokens] in: {input_tokens}, out: {output_tokens}");
				}
				AppEvent::ToolUseStart {
					name,
					input_preview,
					..
				} => {
					eprintln!("[tool] {name} {input_preview}");
				}
				AppEvent::ToolResult {
					name,
					output,
					is_error,
					..
				} => {
					if is_error {
						eprintln!("[tool result][error] {name}: {output}");
					} else {
						eprintln!("[tool result] {name}: {output}");
					}
				}
				AppEvent::Shutdown => break,
				AppEvent::BlockComplete {
					..
				} => {}
			}
		}

		input_handle.abort();
		Ok(())
	}
}
