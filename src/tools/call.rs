//! Tool call and result representations for `Kloud` tools.

use serde::{Deserialize, Serialize};

/// Tool call representation
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    /// Name of the tool to call, e.g. "read", "write", "bash", etc.
    pub name: String,

    /// Arguments for the tool call, represented as a JSON object for flexibility
    pub args: serde_json::Value,
}

/// Outcome of a tool execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum ToolResultKind {
    Success,
    Error,
    Canceled,
    Rejected,
}

/// Tool call result representation
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    /// Name of the tool that was called
    pub name: String,

    /// Outcome classification.
    pub kind: ToolResultKind,

    /// Output text (success payload or error message).
    pub output: String,
}
