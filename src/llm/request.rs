//! Request types for the Anthropic Messages API.

use serde::{Deserialize, Serialize};

use super::types::{CacheControl, InputMessage};

// ---------------------------------------------------------------------------
// System prompt
// ---------------------------------------------------------------------------

/// A block inside the structured system prompt array.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub text: String,
    #[serde(rename = "type")]
    pub type_: TextType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<serde_json::Value>,
}

/// Prompt block type discriminator (currently always `Text`).
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextType {
    Text,
}

/// System prompt — either a plain string or structured blocks with cache control.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    Single(String),
    Multiple(Vec<Prompt>),
}

// ---------------------------------------------------------------------------
// Request configuration enums
// ---------------------------------------------------------------------------

/// Controls whether extended thinking is enabled and its token budget.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Thinking {
    Enabled {
        budget_tokens: u32,
    },
    Disabled,
}

/// Controls how the model selects tools.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    /// Model decides whether to use tools.
    Auto {
        disable_parallel_tool_use: bool,
    },
    /// Model must use at least one tool.
    Any {
        disable_parallel_tool_use: bool,
    },
    /// Model must use this specific tool.
    Tool {
        name: String,
        disable_parallel_tool_use: bool,
    },
    /// Model must not use any tools.
    None,
}

// ---------------------------------------------------------------------------
// Chat request
// ---------------------------------------------------------------------------

/// Request body for the Messages API.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<InputMessage>,
    pub system: SystemPrompt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Tool definitions (JSON Schema). Each element should have `name`,
    /// `description`, and `input_schema` fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<Thinking>,
}
