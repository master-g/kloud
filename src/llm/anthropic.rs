//! Anthropic compatible LLM client.

use std::pin::Pin;

use futures_util::{Stream, TryStreamExt};
use tokio_util::codec::FramedRead;
use tokio_util::io::StreamReader;
use tracing::trace;
use url::Url;

use crate::{
	error::LlmError,
	llm::{
		client::{LlmClient, ModelInfo},
		error::ClientError,
		request::{ChatRequest, SystemPrompt},
		response::{ChatResponse, StreamEvent},
		sse::SseDecoder,
		types::{CacheControl, CacheControlType, ContentBlock, InputMessage, Role},
	},
};

// constants
const SSE_EVENT_DATA_PREFIX: &str = "data: ";

/// Anthropic compatible LLM client.
#[derive(Debug, Clone)]
pub struct AnthropicClient {
	api_key: String,
	message_url: Url,
	model: String,
	max_context_length: u32,
	max_response_tokens: u32,
	reqwest_client: reqwest::Client,
}

/// Builder for [`AnthropicClient`].
#[derive(Debug, Default)]
pub struct Builder {
	api_key: Option<String>,
	base_url: Option<String>,
	model: Option<String>,
	max_context_length: Option<u32>,
	max_response_tokens: Option<u32>,
	reqwest_client: Option<reqwest::Client>,
}

#[allow(missing_docs)]
impl Builder {
	pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
		self.api_key = Some(api_key.into());
		self
	}

	pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
		self.base_url = Some(base_url.into());
		self
	}

	pub fn with_model(mut self, model: impl Into<String>) -> Self {
		self.model = Some(model.into());
		self
	}

	pub fn with_max_context_length(mut self, max_context_length: u32) -> Self {
		self.max_context_length = Some(max_context_length);
		self
	}

	pub fn with_max_response_tokens(mut self, max_response_tokens: u32) -> Self {
		self.max_response_tokens = Some(max_response_tokens);
		self
	}

	pub fn with_custom_reqwest_client(mut self, reqwest_client: reqwest::Client) -> Self {
		self.reqwest_client = Some(reqwest_client);
		self
	}

	pub fn build(self) -> Result<AnthropicClient, ClientError> {
		let api_key =
			self.api_key.ok_or(ClientError::BadArgument("api_key not provided".to_string()))?;
		let base_url_raw =
			self.base_url.ok_or(ClientError::BadArgument("base_url not provided".to_string()))?;
		let model = self.model.ok_or(ClientError::BadArgument("model not provided".to_string()))?;
		let max_context_length = self.max_context_length.unwrap_or(200 * 1024);
		let max_response_tokens = self.max_response_tokens.unwrap_or_default();
		let reqwest_client = if let Some(client) = self.reqwest_client {
			client
		} else {
			reqwest::Client::new()
		};

		let base_url = Url::parse(&base_url_raw)?;

		let mut message_url = base_url.clone();
		message_url.path_segments_mut().unwrap().pop_if_empty().extend(["v1", "messages"]);

		Ok(AnthropicClient {
			api_key,
			message_url,
			model,
			max_context_length,
			max_response_tokens,
			reqwest_client,
		})
	}
}

impl AnthropicClient {
	/// Returns a new [`Builder`] for configuring an [`AnthropicClient`].
	pub fn new_builder() -> Builder {
		Builder::default()
	}

	/// Make a simple chat request with a system prompt and a prompt.
	pub fn make_simple_chat(
		&self,
		system_prompt: impl Into<String>,
		prompt: impl Into<String>,
		stream: bool,
	) -> ChatRequest {
		ChatRequest {
			model: self.model.clone(),
			messages: vec![InputMessage {
				role: Role::User,
				content: vec![ContentBlock::Text {
					text: prompt.into(),
					cache_control: Some(CacheControl {
						type_: CacheControlType::Ephemeral,
					}),
				}],
			}],
			system: SystemPrompt::Single(system_prompt.into()),
			max_tokens: Some(1024),
			stream,
			temperature: None,
			top_p: None,
			tool_choice: None,
			tools: None,
			thinking: None,
		}
	}
}

#[async_trait::async_trait]
impl LlmClient for AnthropicClient {
	async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmError> {
		let response = self
			.reqwest_client
			.post(self.message_url.as_ref())
			.header("x-api-key", self.api_key.to_string())
			.header("anthropic-version", "2023-06-01")
			.json(&request)
			.send()
			.await
			.map_err(|e| LlmError::RequestFailed(e.to_string()))?;

		let status = response.status();
		match status {
			reqwest::StatusCode::OK => {}
			reqwest::StatusCode::UNAUTHORIZED => {
				return Err(LlmError::AuthFailed);
			}
			reqwest::StatusCode::TOO_MANY_REQUESTS => {
				return Err(LlmError::RateLimited);
			}
			_ => {
				return Err(LlmError::RequestFailed(response.text().await?));
			}
		}

		let response_body = response.text().await?;
		Ok(serde_json::from_str(&response_body)?)
	}

	async fn chat_stream(
		&self,
		request: ChatRequest,
	) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>, LlmError> {
		let response = self
			.reqwest_client
			.post(self.message_url.as_ref())
			.header("x-api-key", self.api_key.to_string())
			.header("anthropic-version", "2023-06-01")
			.json(&request)
			.send()
			.await
			.map_err(|e| LlmError::RequestFailed(e.to_string()))?;

		let status = response.status();
		match status {
			reqwest::StatusCode::OK => {}
			reqwest::StatusCode::UNAUTHORIZED => {
				return Err(LlmError::AuthFailed);
			}
			reqwest::StatusCode::TOO_MANY_REQUESTS => {
				return Err(LlmError::RateLimited);
			}
			_ => {
				return Err(LlmError::RequestFailed(response.text().await?));
			}
		}

		// bytes_stream → map error to io::Error → StreamReader (AsyncRead)
		// → FramedRead with SseDecoder → parse SSE events
		let byte_stream = response.bytes_stream().map_err(std::io::Error::other);
		let stream_reader = StreamReader::new(byte_stream);
		let stream = FramedRead::new(stream_reader, SseDecoder)
			.map_err(|e| LlmError::StreamError(e.to_string()))
			.try_filter_map(|message: String| async move {
				trace!("SSE from server: {message}");
				// Each SSE frame from Anthropic contains exactly one `data:` line.
				// We scan all lines defensively but return the first `data:` match.
				for line in message.lines() {
					if let Some(data) = line.strip_prefix(SSE_EVENT_DATA_PREFIX) {
						if data == "[DONE]" {
							return Ok(Some(StreamEvent::MessageStop));
						}
						let event =
							serde_json::from_str::<StreamEvent>(data).map_err(LlmError::Serde)?;
						return Ok(Some(event));
					}
				}
				// Frame contained no `data:` line (e.g. comment-only or empty) — skip it.
				Ok(None)
			});

		Ok(Box::pin(stream))
	}

	fn model_info(&self) -> ModelInfo {
		ModelInfo {
			name: self.model.clone(),
			max_context_length: self.max_context_length,
			max_response_tokens: self.max_response_tokens,
		}
	}
}

#[cfg(test)]
mod tests {
	use url::Url;

	#[test]
	fn test_url_join() {
		let a = vec![
			"http://localhost:8080",
			"http://localhost:8080/",
			"http://localhost:8080/anthropic",
			"http://localhost:8080/anthropic/",
		];
		let b = vec![
			"http://localhost:8080/v1/messages",
			"http://localhost:8080/v1/messages",
			"http://localhost:8080/anthropic/v1/messages",
			"http://localhost:8080/anthropic/v1/messages",
		];

		a.into_iter().zip(b).for_each(|(a, b)| {
			let mut base_url = Url::parse(a).unwrap();
			base_url
				.path_segments_mut()
				.map_err(|_| url::ParseError::RelativeUrlWithoutBase)
				.unwrap()
				.pop_if_empty()
				.extend(["v1", "messages"]);

			assert_eq!(base_url.to_string(), b);
		});
	}
}
