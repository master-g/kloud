//! Test basic chat functionality

use kloud::{
	config::Config,
	env::load_env,
	llm::{anthropic::AnthropicClient, client::LlmClient},
};

use test_log::test;
use tracing::{debug, info};

#[test(tokio::test)]
async fn test_echo() {
	load_env();

	let mut config = Config::default();
	config.load_from_env();

	debug!("cfg: {:?}", config);

	let client = AnthropicClient::new_builder()
		.with_base_url(config.llm.api_base_url)
		.with_api_key(config.llm.api_key.unwrap())
		.with_model(config.llm.model)
		.build()
		.unwrap();

	let system_prompt = "You are a helpful assistant.";
	let prompt = "Tell me about yourself.";
	let request = client.make_simple_chat(system_prompt, prompt);

	let response = client.chat(request).await.unwrap();

	info!("{:?}", response);
}
