//! Trait definition for tools. This defines the common interface that all tools must implement, and allows us to execute them in a consistent way.

use crate::error::Result;
use crate::tools::{ToolCall, ToolResult};

/// Tool trait that all tools must implement. This allows us to have a common interface for all tools, and to execute them in a consistent way.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
	/// Returns the name of the tool, e.g. "read", "write", "bash", etc.
	fn name(&self) -> &str;

	/// Returns a description of the tool, which can be used for documentation or help messages.
	fn description(&self) -> &str;

	/// Returns the input schema for the tool, which can be used for validation or documentation.
	fn input_schema(&self) -> serde_json::Value;

	/// Executes the tool with the given arguments and returns the output as a string.
	/// The arguments are provided as a JSON value for flexibility.
	async fn execute(&self, call: &ToolCall) -> Result<ToolResult>;
}
