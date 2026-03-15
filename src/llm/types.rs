//! Shared types for the Anthropic Messages API.
//!
//! These are the building blocks used by both requests and responses:
//! roles, content blocks, messages, and cache control.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared primitives
// ---------------------------------------------------------------------------

/// Conversation role. Anthropic uses only `user` and `assistant`;
/// system instructions live in a separate top-level field.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
	#[serde(rename = "user")]
	User,
	#[serde(rename = "assistant")]
	Assistant,
}

/// Cache control hint attached to content blocks and system prompts.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
	#[serde(rename = "type")]
	pub type_: CacheControlType,
}

/// Cache control type. Currently only `Ephemeral` is supported.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheControlType {
	Ephemeral,
}

// ---------------------------------------------------------------------------
// Content blocks
// ---------------------------------------------------------------------------

/// A single block inside a message's `content` array.
///
/// The `type` field is the JSON discriminator (handled by `#[serde(tag = "type")]`).
/// Both request and response messages use the same enum — `ToolResult` is sent
/// by the client, while `ToolUse` / `Thinking` / `RedactedThinking` come from
/// the assistant.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
	Text {
		text: String,
		#[serde(skip_serializing_if = "Option::is_none")]
		cache_control: Option<CacheControl>,
	},
	/// Extended thinking (chain-of-thought visible to the caller).
	Thinking {
		thinking: String,
		/// Opaque signature for verification.
		signature: String,
	},
	/// Redacted thinking — the model thought but the content was filtered.
	RedactedThinking {
		data: String,
	},
	/// The assistant wants to call a tool.
	ToolUse {
		/// Unique id for this call; echoed back in the matching `ToolResult`.
		id: String,
		name: String,
		input: serde_json::Value,
		#[serde(skip_serializing_if = "Option::is_none")]
		cache_control: Option<CacheControl>,
	},
	/// Result of a tool execution, sent by the client.
	ToolResult {
		/// Must match the `id` from the corresponding `ToolUse`.
		tool_use_id: String,
		content: String,
		#[serde(skip_serializing_if = "Option::is_none")]
		is_error: Option<bool>,
	},
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// A single message in the conversation history.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMessage {
	pub role: Role,
	pub content: Vec<ContentBlock>,
}
