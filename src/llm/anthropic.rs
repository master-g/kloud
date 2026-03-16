//! Anthropic compatible LLM client.

use std::pin::Pin;

use futures::Stream;
use url::Url;

use crate::{
	error::LlmError,
	llm::{
		client::{LlmClient, ModelInfo},
		error::ClientError,
		request::ChatRequest,
		response::{ChatResponse, StreamEvent},
	},
};

/// Anthropic compatible LLM client.
#[derive(Debug, Clone)]
pub struct AnthropicClient {
	api_key: String,
	base_url: Url,
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

		Ok(AnthropicClient {
			api_key,
			base_url,
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

	/// Returns the URL for sending messages to the Anthropic API.
	pub fn message_url(&self) -> Result<Url, url::ParseError> {
		self.base_url.join("/v1/messages")
	}
}

#[async_trait::async_trait]
impl LlmClient for AnthropicClient {
	async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmError> {
		let response = self
			.reqwest_client
			.post(self.message_url()?)
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
		_request: ChatRequest,
	) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>, LlmError> {
		todo!()
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
	use crate::llm::anthropic::AnthropicClient;

	fn client() -> AnthropicClient {
		AnthropicClient::new_builder()
			.with_api_key("sk-cccccccccccccccccccccccccccccccc")
			.with_base_url("http://localhost:8080/")
			.with_model("kcloud-opus-4.6")
			.build()
			.unwrap()
	}

	#[test]
	fn test_url() {
		let client = client();
		assert_eq!(client.message_url().unwrap().to_string(), "http://localhost:8080/v1/messages");
	}
}
