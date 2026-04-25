//! LLM client trait — the abstraction boundary between the Agent and the API.

use std::pin::Pin;

use futures::Stream;

use crate::error::LlmError;
use crate::llm::response::StreamEvent;

use super::request::ChatRequest;
use super::response::ChatResponse;

// ---------------------------------------------------------------------------
// Model metadata
// ---------------------------------------------------------------------------

/// Static information about a model (context window size, etc.).
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub max_context_length: u32,
    pub max_response_tokens: u32,
}

// ---------------------------------------------------------------------------
// Client trait
// ---------------------------------------------------------------------------

/// Abstract LLM client. Implement this for real API calls (`AnthropicClient`)
/// and for testing (`MockClient`).
#[async_trait::async_trait]
pub trait LlmClient {
    /// Send a chat completion request and return the full response.
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmError>;

    /// Send a chat completion request and return a stream of partial responses.
    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>, LlmError>;

    /// Return metadata about the underlying model.
    fn model_info(&self) -> ModelInfo;
}
