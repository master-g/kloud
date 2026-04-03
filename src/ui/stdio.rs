//! Stdio-based UI backend (println + stdin).

use tokio::io::AsyncBufReadExt;

use crate::agent::{DisplayBlock, MessageType, SessionView};

use super::backend::{UiBackend, UiChannels};
use super::events::{AppEvent, UiAction};

/// A simple line-oriented UI that uses stdout/stdin directly.
pub struct StdioBackend;

#[async_trait::async_trait]
impl UiBackend for StdioBackend {
	async fn run(self, mut channels: UiChannels) -> crate::Result<()> {
		let action_tx = channels.action_tx.clone();

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
					UiAction::SlashCommand {
						command: parts.next().unwrap_or("").to_string(),
						args: parts.next().unwrap_or("").to_string(),
					}
				} else {
					UiAction::SendMessage(trimmed)
				};

				if action_tx.send(action).await.is_err() {
					break;
				}
			}
		});

		let mut last_view = SessionView::default();
		while let Some(event) = channels.event_rx.recv().await {
			match event {
				AppEvent::View(view) => {
					render_incremental_view(&last_view, view.as_ref());
					last_view = *view;
				}
				AppEvent::Shutdown => break,
			}
		}

		input_handle.abort();
		Ok(())
	}
}

fn render_incremental_view(previous: &SessionView, next: &SessionView) {
	let previous_len = previous.messages.len();
	let next_len = next.messages.len();

	if next_len > previous_len {
		for message in &next.messages[previous_len..] {
			render_message(message);
		}
		return;
	}

	let Some(previous_last) = previous.messages.last() else {
		return;
	};
	let Some(next_last) = next.messages.last() else {
		return;
	};
	if previous_last.id != next_last.id {
		render_message(next_last);
		return;
	}

	if next_last.blocks.len() <= previous_last.blocks.len() {
		return;
	}

	for block in &next_last.blocks[previous_last.blocks.len()..] {
		render_block(&next_last.message_type, block);
	}
}

fn render_message(message: &crate::agent::DisplayMessage) {
	for block in &message.blocks {
		render_block(&message.message_type, block);
	}
	eprintln!();
}

fn render_block(message_type: &MessageType, block: &DisplayBlock) {
	match (message_type, block) {
		(MessageType::User, DisplayBlock::Text(text)) => eprintln!("> {text}"),
		(MessageType::Assistant, DisplayBlock::Text(text)) => eprint!("{text}"),
		(MessageType::Assistant, DisplayBlock::Thinking(text)) => eprint!("[thinking] {text}"),
		(MessageType::Assistant, DisplayBlock::RedactedThinking(_)) => {
			eprint!("[thinking redacted]");
		}
		(
			MessageType::Assistant,
			DisplayBlock::ToolUse {
				name,
				input_preview,
				..
			},
		) => eprintln!("[tool] {name} {input_preview}"),
		(
			MessageType::Assistant | MessageType::User,
			DisplayBlock::ToolResult {
				name,
				output,
				is_error,
				..
			},
		) => {
			if *is_error {
				eprintln!("[tool result][error] {name}: {output}");
			} else {
				eprintln!("[tool result] {name}: {output}");
			}
		}
		(
			MessageType::System {
				level,
			},
			DisplayBlock::Text(text),
		) => {
			eprintln!("[{level:?}] {text}");
		}
		(
			MessageType::Progress {
				..
			},
			DisplayBlock::Text(text),
		) => {
			eprintln!("[progress] {text}");
		}
		_ => {}
	}
}
