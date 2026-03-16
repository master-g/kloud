//! Example: non-streaming chat request.
//!
//! ```bash
//! cargo run --example echo
//! ```

use kloud::{
	config::Config,
	env::load_env,
	llm::{anthropic::AnthropicClient, client::LlmClient},
};

use tracing::info;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();
	load_env();

	let mut config = Config::default();
	config.load_from_env();

	let client = AnthropicClient::new_builder()
		.with_base_url(config.llm.api_base_url)
		.with_api_key(config.llm.api_key.expect("KLOUD_API_KEY must be set"))
		.with_model(config.llm.model)
		.build()
		.expect("failed to build AnthropicClient");

	let request =
		client.make_simple_chat("You are a helpful assistant.", "Tell me about yourself.", false);

	let response = client.chat(request).await.expect("chat request failed");

	info!("{:?}", response);
}
