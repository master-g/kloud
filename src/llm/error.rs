//! Error types for the LLM API.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error returned by the Anthropic API in the response body.
#[allow(missing_docs)]
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
	#[error("API error: {0}")]
	Message(String),
}
