use crate::agent::SessionEvent;
use crate::agent::message::MessageLevel;
use crate::agent::view::PendingPermissionView;
use futures_util::StreamExt;
use tokio::select;

use crate::ui::UiAction;
use crate::ui::events::AppEvent;

use super::types::{StreamingActionOutcome, TurnState};
use super::{SLASH_COMMANDS, Session};

impl Session {
    /// Run the conversation loop until the user exits.
    pub async fn run(mut self) -> crate::Result<()> {
        self.publish_view().await;

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
                        UiAction::SetScreen(screen) => {
                            self.apply_event(SessionEvent::ScreenChanged {
                                screen,
                            })
                            .await;
                            TurnState::Idle
                        }
                        UiAction::SetTranscriptShowAll(show_all) => {
                            self.apply_event(SessionEvent::TranscriptShowAllChanged {
                                show_all,
                            })
                            .await;
                            TurnState::Idle
                        }
                        UiAction::CancelTurn
                        | UiAction::ChangeTheme(_)
                        | UiAction::ToggleToolCollapse(_)
                        | UiAction::SearchActivate
                        | UiAction::SearchSubmit(_)
                        | UiAction::SearchNext
                        | UiAction::SearchPrev
                        | UiAction::SearchExit => TurnState::Idle,
                        UiAction::PermissionResponse {
                            allowed,
                        } => {
                            self.apply_event(SessionEvent::PendingPermissionChanged {
                                pending_permission: None,
                            })
                            .await;
                            let (content, level) = if allowed {
                                ("Permission granted".to_string(), MessageLevel::Info)
                            } else {
                                ("Permission denied".to_string(), MessageLevel::Warning)
                            };
                            self.apply_event(SessionEvent::SystemMessageAdded {
                                content,
                                level,
                            })
                            .await;
                            TurnState::Idle
                        }
                    }
                }
                TurnState::Streaming {
                    mut stream,
                    stop_reason,
                    tool_names_by_id,
                    server_names_by_id,
                    completed_assistant_blocks,
                    pending_blocks_by_index,
                } => {
                    // While a model turn is active we react to two event sources:
                    // 1. the next streamed event from the LLM
                    // 2. a cancellation/exit action from the UI
                    select! {
                        maybe_event = stream.next() => {
                            self
                                    .handle_stream_poll(
                                        maybe_event,
                                        stream,
                                        stop_reason,
                                        tool_names_by_id,
                                        server_names_by_id,
                                        completed_assistant_blocks,
                                        pending_blocks_by_index,
                                )
                                .await?
                        }
                        maybe_action = self.handle.action_rx.recv() => {
                            match self
                                    .handle_stream_action(
                                        maybe_action,
                                        stream,
                                        stop_reason,
                                        tool_names_by_id,
                                        server_names_by_id,
                                        completed_assistant_blocks,
                                        pending_blocks_by_index,
                                )
                                .await?
                            {
                                StreamingActionOutcome::Next(next_state) => next_state,
                                StreamingActionOutcome::Exit => break,
                            }
                        }
                    }
                }
            };
        }

        Ok(())
    }

    /// Returns `true` to continue the loop, `false` to exit.
    async fn handle_slash_command(&mut self, command: &str, args: &str) -> bool {
        match command {
            "help" => {
                let mut help_lines = vec![String::from("Available commands:")];
                for entry in SLASH_COMMANDS {
                    help_lines.push(format!("  /{} — {}", entry.name, entry.summary));
                }
                self.apply_event(SessionEvent::SystemMessageAdded {
                    content: help_lines.join("\n"),
                    level: MessageLevel::Info,
                })
                .await;
            }
            "exit" | "quit" => {
                let _ = self.handle.event_tx.send(AppEvent::Shutdown).await;
                return false;
            }
            "theme" => {
                let theme_name = if args.is_empty() {
                    "dark"
                } else {
                    args.trim()
                };
                match theme_name.parse::<crate::ui::tui::theme::ColorScheme>() {
                    Ok(_) => {
                        let _ = self
                            .handle
                            .event_tx
                            .send(AppEvent::ThemeChanged(theme_name.to_string()))
                            .await;
                        self.apply_event(SessionEvent::SystemMessageAdded {
                            content: format!("Theme changed to {theme_name}"),
                            level: MessageLevel::Info,
                        })
                        .await;
                    }
                    Err(e) => {
                        self.apply_event(SessionEvent::SystemMessageAdded {
                            content: e,
                            level: MessageLevel::Error,
                        })
                        .await;
                    }
                }
            }
            "permission" => {
                self.apply_event(SessionEvent::PendingPermissionChanged {
                    pending_permission: Some(PendingPermissionView {
                        label: "File write access".to_string(),
                        description: "Allow writing to src/main.rs".to_string(),
                    }),
                })
                .await;
            }
            "notify" => {
                let msg = if args.is_empty() {
                    "Demo notification".to_string()
                } else {
                    args.trim().to_string()
                };
                self.apply_event(SessionEvent::SystemMessageAdded {
                    content: format!("[toast] {msg}"),
                    level: MessageLevel::Info,
                })
                .await;
            }
            _ => {
                let known = SLASH_COMMANDS
                    .iter()
                    .map(|entry| format!("/{}", entry.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let msg = format!("Unknown command: /{command}. Try /help. Available: {known}");
                self.apply_event(SessionEvent::SystemMessageAdded {
                    content: msg,
                    level: MessageLevel::Error,
                })
                .await;
            }
        }
        true
    }
}
