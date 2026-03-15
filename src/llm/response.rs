//! Response types for the Anthropic Messages API.

use serde::{Deserialize, Serialize};

use crate::llm::error::ApiError;

use super::types::ContentBlock;

// ---------------------------------------------------------------------------
// Stop reason
// ---------------------------------------------------------------------------

/// Why the model stopped generating.
///
/// The Agent Loop branches on this: `ToolUse` → execute tools and continue,
/// `EndTurn` → present result to user, `MaxTokens` → may need to continue.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
	EndTurn,
	MaxTokens,
	StopSequence,
	ToolUse,
	PauseTurn,
	Refusal,
	ModelContextWindowExceeded,
}

// ---------------------------------------------------------------------------
// Usage & metadata
// ---------------------------------------------------------------------------

/// Token usage statistics.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
	pub input_tokens: u32,
	pub output_tokens: u32,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cache_creation_input_tokens: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cache_read_input_tokens: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cache_creation: Option<CacheCreation>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub server_tool_use: Option<ServerToolUsage>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub service_tier: Option<ServiceTier>,
}

/// Cache creation token breakdown.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheCreation {
	pub ephemeral_1h_input_tokens: u32,
	pub ephemeral_5m_input_tokens: u32,
}

/// Server-side tool usage counters.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToolUsage {
	pub web_fetch_requests: u32,
	pub web_search_requests: u32,
}

/// API service tier for the request.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceTier {
	Standard,
	Priority,
	Batch,
}

/// Server-side context edits (e.g. automatic truncation).
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManagement {
	pub applied_edits: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Chat response
// ---------------------------------------------------------------------------

/// Response body from the Messages API.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
	pub id: String,
	pub role: String,
	pub model: String,
	pub content: Vec<ContentBlock>,
	pub stop_reason: Option<StopReason>,
	pub stop_sequence: Option<String>,
	pub usage: Usage,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub context_management: Option<ContextManagement>,
}

// ---------------------------------------------------------------------------
// Stream event
// ---------------------------------------------------------------------------

/// A Server-Sent Event emitted during a streaming response.
///
/// Events arrive in order: `MessageStart` → (`ContentBlockStart` →
/// `ContentBlockDelta`* → `ContentBlockStop`)* → `MessageDelta` → `MessageStop`.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
	MessageStart {
		message: ChatResponse,
	},
	ContentBlockStart {
		index: u32,
		content_block: ContentBlock,
	},
	ContentBlockDelta {
		index: u32,
		delta: Delta,
	},
	ContentBlockStop {
		index: u32,
	},
	MessageDelta {
		delta: MessageDelta,
		usage: Usage,
	},
	MessageStop,
	Ping,
	Error {
		error: ApiError,
	},
}

/// Incremental content within a `ContentBlockDelta` event.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Delta {
	TextDelta {
		text: String,
	},
	InputJsonDelta {
		partial_json: String,
	},
	ThinkingDelta {
		thinking: String,
	},
	SignatureDelta {
		signature: String,
	},
}

/// Message-level delta (stop reason update) emitted near the end of a stream.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelta {
	pub stop_reason: Option<StopReason>,
	pub stop_sequence: Option<String>,
}
