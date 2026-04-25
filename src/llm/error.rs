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

/// Error returned by the LLM client.
#[allow(missing_docs)]
#[derive(Error, Debug, Clone)]
pub enum ClientError {
    #[error("bad argument: {0}")]
    BadArgument(String),

    /// Invalid URL provided to the client.
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),
}
