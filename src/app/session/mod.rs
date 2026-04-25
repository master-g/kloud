//! Session — drives the conversation loop between user and LLM.

mod run;
mod stream;
mod tools;
mod types;

use crate::agent::SessionEvent;
use crate::agent::SessionStore;
use crate::llm::client::LlmClient;
use crate::llm::types::InputMessage;
use crate::tools::ToolRegistry;
use crate::ui::backend::UiHandle;
use crate::ui::events::AppEvent;

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
/// [`types::TurnState`] is intentionally separate: that enum only tracks the
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
    /// Registered tools available to the session for declaration and dispatch.
    tool_registry: ToolRegistry,
    /// Internal transcript and query state store.
    store: SessionStore,
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
            store: SessionStore::new(),
        }
    }

    pub(super) async fn publish_view(&self) {
        let _ = self.handle.event_tx.send(AppEvent::View(Box::new(self.store.view()))).await;
    }

    pub(super) async fn apply_event(&mut self, event: SessionEvent) {
        self.store.apply(event);
        self.publish_view().await;
    }
}
