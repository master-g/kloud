//! Error handling module

use thiserror::Error;

/// Kloud error type
#[derive(Debug, Error)]
pub enum Error {
    /// Configuration error
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Tool execution error
    #[error("tool error: {0}")]
    Tool(#[from] ToolError),

    /// Agent error
    #[error("agent error: {0}")]
    Agent(#[from] AgentError),

    /// LLM API error
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),

    /// UI error
    #[error("UI error: {0}")]
    Ui(#[from] UiError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Other error using anyhow
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// Configuration error
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to read the configuration file from disk
    #[error("failed to read config file: {0}")]
    ReadError(String),

    /// Failed to parse the configuration file content
    #[error("failed to parse config: {0}")]
    ParseError(String),

    /// Required configuration field is missing
    #[error("missing required config: {0}")]
    MissingField(String),

    /// Configuration value is invalid
    #[error("invalid config value: {0}")]
    InvalidValue(String),

    /// Config file not found at specified path
    #[error("config file not found: {0}")]
    FileNotFound(String),

    /// Failed to create config directory
    #[error("failed to create config directory: {0}")]
    CreateDirError(String),

    /// Failed to write config file
    #[error("failed to write config file: {0}")]
    WriteError(String),

    /// Failed to serialize config to TOML
    #[error("failed to serialize config: {0}")]
    SerializeError(String),
}

/// Tool execution error
#[derive(Debug, Error)]
pub enum ToolError {
    /// Requested tool does not exist
    #[error("tool not found: {0}")]
    NotFound(String),

    /// Tool execution failed with an error
    #[error("tool execution failed: {0}")]
    ExecutionFailed(String),

    /// Path traversal attempt detected (security violation)
    #[error("path security violation: {0}")]
    PathSecurity(String),

    /// Command is not in the allowed list
    #[error("command not allowed: {0}")]
    CommandNotAllowed(String),

    /// Tool execution exceeded time limit
    #[error("timeout: {0}")]
    Timeout(String),
}

/// Agent error
#[derive(Debug, Error)]
pub enum AgentError {
    /// Agent is in an invalid state for the requested operation
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// Failed to handle or process a message
    #[error("message handling failed: {0}")]
    MessageFailed(String),

    /// Failed to delegate task to sub-agent
    #[error("delegation failed: {0}")]
    DelegationFailed(String),
}

/// LLM API error
#[derive(Debug, Error)]
pub enum LlmError {
    /// HTTP request to LLM API failed
    #[error("API request failed: {0}")]
    RequestFailed(String),

    /// LLM API returned an invalid or unexpected response
    #[error("API response invalid: {0}")]
    InvalidResponse(String),

    /// Serialization/deserialization error
    #[error("Serialization/deserialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Reqwest error
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// URL error
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Rate limit exceeded, too many requests
    #[error("rate limit exceeded")]
    RateLimited,

    /// Authentication failed, invalid API key
    #[error("authentication failed")]
    AuthFailed,

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// SSE stream error
    #[error("SSE stream error: {0}")]
    StreamError(String),
}

/// UI error
#[derive(Debug, Error)]
pub enum UiError {
    /// Terminal setup or teardown failed
    #[error("terminal error: {0}")]
    Terminal(String),

    /// Channel communication error
    #[error("channel error: {0}")]
    Channel(String),

    /// Rendering error
    #[error("render error: {0}")]
    Render(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
