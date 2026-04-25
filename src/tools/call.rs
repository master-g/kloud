//! Tool call and result representations for `KCloud` tools.

use serde::{Deserialize, Serialize};

/// Tool call representation
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    /// Name of the tool to call, e.g. "read", "write", "bash", etc.
    pub name: String,

    /// Arguments for the tool call, represented as a JSON object for flexibility
    pub args: serde_json::Value,
}

/// Tool call result representation
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    /// Name of the tool that was called
    pub name: String,

    /// Output from the tool call - Ok for success, Err for failure
    pub output: Result<String, String>,
}
